use chrono::{DateTime, Datelike, Timelike, Utc};

pub(crate) struct Partition {
    pub(crate) date: String,
    pub(crate) hour: u32,
}

pub(crate) fn partition(timestamp_ms: i64) -> Partition {
    let datetime =
        DateTime::<Utc>::from_timestamp_millis(timestamp_ms).unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    Partition {
        date: format!(
            "{:04}-{:02}-{:02}",
            datetime.year(),
            datetime.month(),
            datetime.day()
        ),
        hour: datetime.hour(),
    }
}

pub(crate) fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}
