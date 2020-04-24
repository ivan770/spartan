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
        if let Some(obtained_at) = self.obtained_at {
            (obtained_at + self.max as i64) < timestamp
        } else {
            false
        }
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
    pub(crate) fn new(offset: i32, delay: Option<i64>, timeout: u32) -> Time {
        Time {
            offset,
            timestamp: Self::get_offset_timestamp(offset),
            delay,
            timeout: Timeout::new(timeout),
        }
    }

    pub(crate) fn check_delay(&self) -> bool {
        if let Some(delay) = self.delay {
            delay <= self.get_timestamp()
        } else {
            true
        }
    }

    pub(crate) fn get_raw_delay(&self) -> Option<i64> {
        self.delay
    }

    pub(crate) fn obtain(&mut self) {
        self.timeout.obtain(self.get_timestamp());
    }

    pub(crate) fn expired(&self) -> bool {
        self.timeout.expired(self.get_timestamp())
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
    use super::{Time, Timeout, Utc};
    use std::thread::sleep;
    use std::time::Duration;

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
        let time = Time::new(0, Some(get_timestamp() + 2), 1);
        assert!(!time.check_delay());
        sleep(Duration::from_secs(3));
        assert!(time.check_delay());
    }
}
