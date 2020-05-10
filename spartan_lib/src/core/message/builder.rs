use crate::core::message::Message;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("no body provided for builder")]
    BodyNotProvided,
}

/// Message builder
///
/// ```
/// use spartan_lib::chrono::{Utc, FixedOffset};
/// use spartan_lib::core::message::builder::MessageBuilder;
///
/// let message = MessageBuilder::default()
///     .body(b"Hello, world")
///     .offset(9 * 3600)
///     .max_tries(5)
///     .timeout(60)
///     .delay(|tz| (Utc::today().and_hms(2, 0, 0) + FixedOffset::east(tz)).timestamp())
///     .compose()
///     .unwrap();
/// ```
pub struct MessageBuilder<'a> {
    body: Option<&'a [u8]>,
    offset: i32,
    max_tries: u32,
    timeout: u32,
    delay: Option<i64>,
}

impl Default for MessageBuilder<'_> {
    fn default() -> Self {
        MessageBuilder {
            body: None,
            offset: 0,
            max_tries: 1,
            timeout: 30,
            delay: None,
        }
    }
}

impl<'a> MessageBuilder<'a> {
    /// Message body.
    #[must_use]
    pub fn body(mut self, body: &'a [u8]) -> Self {
        self.body = Some(body);
        self
    }

    /// Timezone offset in seconds.
    #[must_use]
    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = offset;
        self
    }

    /// Max tries for message to be reserved.
    #[must_use]
    pub fn max_tries(mut self, max_tries: u32) -> Self {
        self.max_tries = max_tries;
        self
    }

    /// Message timeout. Used by GC to collect messages, that execute for too long.
    #[must_use]
    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set message delay.
    #[must_use]
    pub fn delay<F>(mut self, delay: F) -> Self
    where
        F: FnOnce(i32) -> i64,
    {
        self.delay = Some(delay(self.offset));
        self
    }

    /// Compose message. Returns Err, if body was not provided.
    pub fn compose(self) -> Result<Message, BuilderError> {
        if let Some(body) = self.body {
            Ok(Message::new(
                body,
                self.delay,
                self.offset,
                self.max_tries,
                self.timeout,
            ))
        } else {
            Err(BuilderError::BodyNotProvided)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MessageBuilder;
    use chrono::Utc;
    use rand::random;

    #[test]
    fn creates_message() {
        MessageBuilder::default()
            .body(&random::<[u8; 16]>())
            .max_tries(3)
            .offset(100)
            .delay(|_| Utc::now().timestamp())
            .timeout(40)
            .compose()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_with_empty_body() {
        MessageBuilder::default()
            .max_tries(3)
            .offset(100)
            .delay(|_| Utc::now().timestamp())
            .timeout(40)
            .compose()
            .unwrap();
    }
}
