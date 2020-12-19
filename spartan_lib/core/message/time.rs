use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// Message timeout options
///
/// Contains max timeout in seconds, and message obtain time.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timeout {
    max: u32,
    #[serde(with = "serialization::tz_local_seconds_option")]
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

    /// Get max timeout in seconds
    ///
    /// If `obtained_at + max < current_time`, then message is expired.
    pub fn max(&self) -> &u32 {
        &self.max
    }

    /// Get message obtain time
    ///
    /// [`None`] if message was never obtained before
    pub fn obtained_at(&self) -> &Option<DateTime<FixedOffset>> {
        &self.obtained_at
    }
}

/// Timezone offset type wrapper for [`i32`]
///
/// Initialization requires for offset to be in range of `(-86399..86400)`
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Offset(i32);

impl Offset {
    pub(crate) fn new(offset: i32) -> Option<Self> {
        if -86_400 < offset && offset < 86_400 {
            Some(Offset(offset))
        } else {
            None
        }
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

/// A time manager for handling message dispatch times, timeouts,
/// delays and timezones
///
/// Be aware, that all time handling itself is accessible to [`Message`] only
///
/// [`Message`]: crate::core::message::Message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Time {
    offset: Offset,

    #[serde(with = "serialization::tz_local_seconds")]
    dispatched_at: DateTime<FixedOffset>,

    #[serde(with = "serialization::tz_local_seconds_option")]
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

    /// Get message timezone offset.
    pub fn offset(&self) -> &Offset {
        &self.offset
    }

    /// Get message dispatch time with offset awareness.
    pub fn dispatched_at(&self) -> &DateTime<FixedOffset> {
        &self.dispatched_at
    }

    /// Get message availability time.
    ///
    /// If `current time > delay`, then message is available for obtaining.
    ///
    /// For implementation details you may want to check out [`Message`] implementation of [`Dispatchable`] trait.
    ///
    /// [`Message`]: crate::core::message::Message
    /// [`Dispatchable`]: crate::core::payload::Dispatchable
    pub fn delay(&self) -> &Option<DateTime<FixedOffset>> {
        &self.delay
    }

    /// Get message timeout options.
    pub fn timeout(&self) -> &Timeout {
        &self.timeout
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

mod serialization {
    use std::fmt::{Formatter, Result as FmtResult};

    use chrono::{DateTime, FixedOffset, TimeZone, Utc};
    use serde::de::{SeqAccess, Visitor};

    struct LocalSecondsTimestampVisitor;

    impl<'de> Visitor<'de> for LocalSecondsTimestampVisitor {
        type Value = DateTime<FixedOffset>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            write!(formatter, "a sequence of UTC timestamp and offset")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let timestamp = seq.next_element()?.expect("Invalid size provided in hint");
            let offset = seq.next_element()?.expect("Invalid size provided in hint");
            Ok(Utc
                .timestamp(timestamp, 0)
                .with_timezone(&FixedOffset::east(offset)))
        }
    }

    pub(crate) mod tz_local_seconds {
        use chrono::{DateTime, FixedOffset};
        use serde::{
            de::Deserializer,
            ser::{SerializeTuple, Serializer},
        };

        use super::LocalSecondsTimestampVisitor;

        pub fn serialize<S>(dt: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut tuple = serializer.serialize_tuple(2)?;

            tuple.serialize_element(&dt.timestamp())?;
            tuple.serialize_element(&dt.offset().local_minus_utc())?;

            tuple.end()
        }

        pub fn deserialize<'de, D>(d: D) -> Result<DateTime<FixedOffset>, D::Error>
        where
            D: Deserializer<'de>,
        {
            d.deserialize_tuple(2, LocalSecondsTimestampVisitor)
        }
    }

    pub(crate) mod tz_local_seconds_option {
        use std::fmt::{Formatter, Result as FmtResult};

        use chrono::{DateTime, FixedOffset};
        use serde::{
            de::{Deserializer, Error, Visitor},
            ser::Serializer,
        };

        use super::LocalSecondsTimestampVisitor;

        struct OptionLocalSecondsTimestampVisitor;

        impl<'de> Visitor<'de> for OptionLocalSecondsTimestampVisitor {
            type Value = Option<DateTime<FixedOffset>>;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                write!(formatter, "a sequence of UTC timestamp and offset or None")
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer
                    .deserialize_tuple(2, LocalSecondsTimestampVisitor)
                    .map(Some)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(None)
            }
        }

        pub fn serialize<S>(
            dt: &Option<DateTime<FixedOffset>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match dt.as_ref() {
                Some(dt) => {
                    serializer.serialize_some(&(dt.timestamp(), dt.offset().local_minus_utc()))
                }
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            d.deserialize_option(OptionLocalSecondsTimestampVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::{DateTime, Duration as ChronoDuration, FixedOffset, Offset, Time, Timeout, Utc};

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

    #[test]
    fn test_initialize_offset_with_correct_value() {
        Offset::new(0).unwrap();
        Offset::new(86399).unwrap();
        Offset::new(-86399).unwrap();
    }

    #[test]
    fn test_initialize_offset_with_incorrect_value() {
        assert!(Offset::new(86400).is_none());
        assert!(Offset::new(-86400).is_none());
    }
}
