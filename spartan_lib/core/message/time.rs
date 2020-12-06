use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Timeout {
    max: u32,
    obtained_at: Option<DateTime<Utc>>,
}

impl Timeout {
    pub(super) fn new(max: u32) -> Timeout {
        Timeout {
            max,
            obtained_at: None,
        }
    }

    pub(super) fn obtain(&mut self, current_time: DateTime<Utc>) {
        self.obtained_at = Some(current_time);
    }

    pub(super) fn expired(&self, current_time: DateTime<Utc>) -> bool {
        self.obtained_at.map_or(false, |obtained_at| {
            (obtained_at + Duration::seconds(i64::from(self.max))) < current_time
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Time {
    offset: i32,
    timestamp: DateTime<Utc>,
    delay: Option<DateTime<Utc>>,
    timeout: Timeout,
}

impl Time {
    pub(crate) fn new(offset: i32, delay: Option<u32>, timeout: u32) -> Time {
        let timestamp = Self::get_datetime();

        Time {
            offset,
            timestamp,
            delay: Self::convert_delay(delay.map(i64::from), timestamp),
            timeout: Timeout::new(timeout),
        }
    }

    pub(crate) fn check_delay(&self) -> bool {
        self.delay
            .map_or(true, |delay| delay <= Self::get_datetime())
    }

    pub(crate) fn get_raw_delay(&self) -> Option<DateTime<Utc>> {
        self.delay
    }

    pub(crate) fn obtain(&mut self) {
        self.timeout.obtain(Self::get_datetime());
    }

    pub(crate) fn expired(&self) -> bool {
        self.timeout.expired(Self::get_datetime())
    }

    fn convert_delay(seconds: Option<i64>, timestamp: DateTime<Utc>) -> Option<DateTime<Utc>> {
        Some(timestamp + Duration::seconds(seconds?))
    }

    fn get_datetime() -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::{DateTime, Duration as ChronoDuration, Time, Timeout, Utc};
    use std::thread::sleep;
    use std::time::Duration;

    fn get_timestamp() -> DateTime<Utc> {
        Utc::now()
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
        let time = Time::new(0, Some(2), 1);
        assert!(!time.check_delay());
        sleep(Duration::from_secs(3));
        assert!(time.check_delay());
    }

    // This test covers 'fast index lookup' bug, that came in version 0.6
    #[test]
    fn test_delay_compare() {
        let time1 = Time::new(0, Some(10), 0);
        let time2 = Time::new(10, Some(2), 0);

        assert!(time1.get_raw_delay() > time2.get_raw_delay());
    }
}
