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
///     .body("Hello, world")
///     .offset(9 * 3600)
///     .max_tries(5)
///     .timeout(60)
///     .delay(10)
///     .compose()
///     .unwrap();
/// ```
pub struct MessageBuilder {
    body: Option<String>,
    offset: i32,
    max_tries: u32,
    timeout: u32,
    delay: Option<u32>,
}

impl Default for MessageBuilder {
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

impl MessageBuilder {
    /// Message body.
    #[must_use]
    pub fn body<T>(mut self, body: T) -> Self
    where
        T: Into<String>,
    {
        self.body = Some(body.into());
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

    /// Message timeout. Used by GC to collect messages that execute for too long.
    #[must_use]
    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set message delay in seconds.
    #[must_use]
    pub fn delay(mut self, delay: u32) -> Self {
        self.delay = Some(delay);
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

    #[test]
    fn creates_message() {
        MessageBuilder::default()
            .body("Hello, world")
            .max_tries(3)
            .offset(100)
            .delay(1)
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
            .delay(1)
            .timeout(40)
            .compose()
            .unwrap();
    }
}
