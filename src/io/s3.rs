use super::json::read_json_array_or_jsonl_bytes;
use crate::args::S3InputArgs;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_types::region::Region;
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};

const S3_LIST_PAGE_SIZE: i32 = 1_000;
const S3_SCAN_LIMIT: usize = 100_000;

pub(super) async fn read_s3_records<T: serde::de::DeserializeOwned>(
    s3: &S3InputArgs,
    latest_read_limit: Option<usize>,
) -> AppResult<(Vec<T>, usize)> {
    let store = connect_store(s3).await?;
    let mut records = Vec::new();
    let mut keys_read = 0;
    let keys = latest_payload_keys(s3, latest_read_limit.unwrap_or(s3.max_keys)).await?;
    for key in keys {
        let bytes = store.get_bytes(&key).await?;
        records.extend(read_json_array_or_jsonl_bytes::<T>(
            &format!("s3://{}/{}", s3.bucket, key),
            &bytes,
        )?);
        keys_read += 1;
    }
    Ok((records, keys_read))
}

pub(super) async fn connect_store(s3: &S3InputArgs) -> AppResult<ObjectStore> {
    ObjectStore::connect(ObjectStoreConfig {
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await
}

pub(super) async fn latest_payload_keys(s3: &S3InputArgs, limit: usize) -> AppResult<Vec<String>> {
    Ok(select_latest_payload_keys(
        list_payload_objects(s3).await?,
        limit,
    ))
}

async fn list_payload_objects(s3: &S3InputArgs) -> AppResult<Vec<ListedPayloadObject>> {
    let client = connect_s3_client(s3).await?;
    let mut objects = Vec::new();
    let mut continuation_token = None;
    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(&s3.bucket)
            .prefix(&s3.prefix)
            .max_keys(S3_LIST_PAGE_SIZE);
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }
        let output = request.send().await.map_err(|error| {
            AppError::aws(format!(
                "list_objects_v2 bucket={} prefix={} error={error}",
                s3.bucket, s3.prefix
            ))
        })?;
        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            if !is_json_payload_key(key) {
                continue;
            }
            objects.push(ListedPayloadObject {
                key: key.to_owned(),
                last_modified_ms: object
                    .last_modified()
                    .and_then(|date_time| date_time.to_millis().ok())
                    .unwrap_or(0),
            });
            if objects.len() >= S3_SCAN_LIMIT {
                return Err(AppError::validation(format!(
                    "s3 prefix scan limit exceeded bucket={} prefix={} limit={S3_SCAN_LIMIT}; narrow the prefix",
                    s3.bucket, s3.prefix
                )));
            }
        }
        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }
    Ok(objects)
}

async fn connect_s3_client(s3: &S3InputArgs) -> AppResult<Client> {
    let mut loader =
        aws_config::defaults(BehaviorVersion::latest()).region(Region::new(s3.region.clone()));
    if let Some(profile) = s3.profile.as_ref() {
        loader = loader.profile_name(profile);
    }
    let sdk_config = loader.load().await;
    Ok(Client::new(&sdk_config))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListedPayloadObject {
    pub(crate) key: String,
    pub(crate) last_modified_ms: i64,
}

pub(crate) fn select_latest_payload_keys(
    mut objects: Vec<ListedPayloadObject>,
    limit: usize,
) -> Vec<String> {
    objects.sort_unstable_by(|left, right| {
        right
            .last_modified_ms
            .cmp(&left.last_modified_ms)
            .then_with(|| right.key.cmp(&left.key))
    });
    objects
        .into_iter()
        .take(limit)
        .map(|object| object.key)
        .collect()
}

fn is_json_payload_key(key: &str) -> bool {
    key.ends_with(".json") || key.ends_with(".jsonl")
}
