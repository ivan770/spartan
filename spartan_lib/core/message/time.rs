use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Timeout {
    max: u32,
    obtained_at: Option<i64>,
}

impl Timeout {
    pub(super) fn new(max: u32) -> Timeout {
        Timeout {
            max,
            obtained_at: None,
        }
    }

    pub(super) fn obtain(&mut self, timestamp: i64) {
        self.obtained_at = Some(timestamp);
    }

    pub(super) fn expired(&self, timestamp: i64) -> bool {
        self.obtained_at.map_or(false, |obtained_at| {
            (obtained_at + i64::from(self.max)) < timestamp
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Time {
    offset: i32,
    timestamp: i64,
    delay: Option<i64>,
    timeout: Timeout,
}

impl Time {
    pub(crate) fn new(offset: i32, delay: Option<u32>, timeout: u32) -> Time {
        let timestamp = Self::get_offset_timestamp(offset);

        Time {
            offset,
            timestamp,
            delay: Self::convert_delay(delay, timestamp),
            timeout: Timeout::new(timeout),
        }
    }

    pub(crate) fn check_delay(&self) -> bool {
        self.delay
            .map_or(true, |delay| delay <= self.get_timestamp())
    }

    pub(crate) fn get_raw_delay(&self) -> Option<i64> {
        // Previous version of get_raw_delay was without this map,
        // resulting in incorrect timezone handling, and possibly even
        // damaging database indexes (such as TreeDatabase index)
        self.delay.map(|delay| delay - i64::from(self.offset))
    }

    pub(crate) fn obtain(&mut self) {
        self.timeout.obtain(self.get_timestamp());
    }

    pub(crate) fn expired(&self) -> bool {
        self.timeout.expired(self.get_timestamp())
    }

    fn convert_delay(seconds: Option<u32>, timestamp: i64) -> Option<i64> {
        Some(timestamp + i64::from(seconds?))
    }

    fn get_timestamp(&self) -> i64 {
        Self::get_offset_timestamp(self.offset)
    }

    fn get_offset_timestamp(offset: i32) -> i64 {
        (Utc::now() + FixedOffset::east(offset)).timestamp()
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::{Time, Timeout, Utc};

    fn get_timestamp() -> i64 {
        Utc::now().timestamp()
    }

    #[test]
    fn test_timeout() {
        let timestamp = get_timestamp();
        let mut timeout = Timeout::new(3);
        timeout.obtain(timestamp);
        assert!(timeout.obtained_at.is_some());
        assert!(!timeout.expired(timestamp));
        assert!(timeout.expired(timestamp + 4));
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
