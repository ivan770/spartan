use serde::Serialize;
use std::convert::TryFrom;
use tide::{Body, Response, StatusCode};

/// Custom "try" macro implementation, that returns Ok response in case of Err variant
#[macro_export]
macro_rules! respond {
    ($expr:expr) => {
        match $expr {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                return std::result::Result::Ok(err.respond());
            }
        }
    };
}

/// Serializable errors
#[derive(Serialize)]
pub struct Error<'a> {
    /// Error status code
    status: u16,

    /// Error message
    message: &'a str,
}

impl<'a> Error<'a> {
    /// Create new Error instance
    pub fn new(status: StatusCode, message: &'a str) -> Self {
        Error {
            status: status.into(),
            message,
        }
    }

    /// Alias for "Bad Request" error
    pub fn bad_request() -> Self {
        Error {
            status: StatusCode::BadRequest.into(),
            message: StatusCode::BadRequest.canonical_reason(),
        }
    }

    /// Convert error instance into Tide response
    pub fn respond(&self) -> Response {
        let mut response =
            Response::new(StatusCode::try_from(self.status).expect("Unknown error status code"));
        response.set_body(Body::from_json(self).expect("Cannot convert error to JSON response"));
        response
    }
}
