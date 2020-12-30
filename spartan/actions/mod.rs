use std::{fmt::Display, result::Result as StdResult};

use spartan_lib::core::message::builder::BuilderError;
use thiserror::Error as ThisError;
use warp::{
    http::response::Builder,
    hyper::{Body, StatusCode},
    reply::Response,
    Reply,
};

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

pub type Result<T> = StdResult<T, ResponseError>;

pub struct ResponseError {
    status: StatusCode,
    error: String,
}

pub trait RespondableError: Display {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<E> From<E> for ResponseError
where
    E: RespondableError,
{
    fn from(error: E) -> Self {
        ResponseError {
            status: error.status_code(),
            error: error.to_string(),
        }
    }
}

impl Reply for ResponseError {
    fn into_response(self) -> Response {
        Builder::default()
            .status(self.status)
            .body(Body::from(self.error))
            .unwrap()
    }
}

#[derive(ThisError, Debug)]
pub enum QueueError {
    #[error("No message available")]
    NoMessageAvailable,
    #[error("Message not found")]
    MessageNotFound,
    #[error("Unable to compose message")]
    MessageCompose(#[from] BuilderError),
}

impl RespondableError for QueueError {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}
