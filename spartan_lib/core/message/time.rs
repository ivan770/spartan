use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Timeout {
    max: u32,
    obtained_at: Option<DateTime<FixedOffset>>,
}

impl Timeout {
    pub(super) fn new(max: u32) -> Timeout {
        Timeout {
            max,
            obtained_at: None,
        }
    }

    pub(super) fn obtain(&mut self, current_time: DateTime<FixedOffset>) {
        self.obtained_at = Some(current_time);
    }

    pub(super) fn expired(&self, current_time: DateTime<FixedOffset>) -> bool {
        self.obtained_at.map_or(false, |obtained_at| {
            (obtained_at + Duration::seconds(i64::from(self.max))) < current_time
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub(crate) struct Offset(i32);

impl Offset {
    pub(crate) fn new(offset: i32) -> Option<Self> {
        if -86_400 < offset && offset < 86_400 {
            Some(Offset(offset))
        } else {
            None
        }
    }

    pub(crate) fn get(self) -> i32 {
        self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Time {
    offset: Offset,
    dispatched_at: DateTime<FixedOffset>,
    delay: Option<DateTime<FixedOffset>>,
    timeout: Timeout,
}

impl Time {
    pub(crate) fn new(offset: Offset, delay: Option<u32>, timeout: u32) -> Time {
        let dispatched_at = Self::get_datetime_with_offset(offset.get());

        Time {
            offset,
            dispatched_at,
            delay: Self::convert_delay(delay.map(i64::from), dispatched_at),
            timeout: Timeout::new(timeout),
        }
    }

    pub(crate) fn check_delay(&self) -> bool {
        self.delay
            .map_or(true, |delay| delay <= self.get_datetime())
    }

    pub(crate) fn get_raw_delay(&self) -> Option<i64> {
        self.delay.as_ref().map(DateTime::timestamp)
    }

    pub(crate) fn obtain(&mut self) {
        self.timeout.obtain(self.get_datetime());
    }

    pub(crate) fn expired(&self) -> bool {
        self.timeout.expired(self.get_datetime())
    }

    fn convert_delay(
        seconds: Option<i64>,
        dispatched_at: DateTime<FixedOffset>,
    ) -> Option<DateTime<FixedOffset>> {
        Some(dispatched_at + Duration::seconds(seconds?))
    }

    fn get_datetime(&self) -> DateTime<FixedOffset> {
        Self::get_datetime_with_offset(self.offset.get())
    }

    fn get_datetime_with_offset(offset: i32) -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&FixedOffset::east(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::{DateTime, Duration as ChronoDuration, FixedOffset, Offset, Time, Timeout, Utc};
    use std::thread::sleep;
    use std::time::Duration;

    fn get_timestamp() -> DateTime<FixedOffset> {
        Utc::now().into()
    }

    #[test]
    fn test_timeout() {
        let timestamp = get_timestamp();
        let mut timeout = Timeout::new(3);
        timeout.obtain(timestamp);
        assert!(timeout.obtained_at.is_some());
        assert!(!timeout.expired(timestamp));
        assert!(timeout.expired(timestamp + ChronoDuration::seconds(4)));
    }

    #[test]
    fn delay_test() {
        let time = Time::new(Offset::new(0).unwrap(), Some(2), 1);
        assert!(!time.check_delay());
        sleep(Duration::from_secs(3));
        assert!(time.check_delay());
    }

    // This test covers 'fast index lookup' bug, that came in version 0.6
    #[test]
    fn test_delay_compare() {
        let time1 = Time::new(Offset::new(0).unwrap(), Some(10), 0);
        let time2 = Time::new(Offset::new(10).unwrap(), Some(2), 0);

        assert!(time1.get_raw_delay() > time2.get_raw_delay());
    }
}
