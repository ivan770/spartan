use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;

/// Clear queue
pub mod clear;

/// Delete message from queue
pub mod delete;

/// Pop message from queue
pub mod pop;

/// Push message to queue
pub mod push;

/// Requeue message back
pub mod requeue;

/// Get queue size
pub mod size;

#[derive(Error, Debug)]
enum QueueError {
    #[error("No message available")]
    NoMessageAvailable,
    #[error("Message not found")]
    MessageNotFound
}

impl ResponseError for QueueError {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}
