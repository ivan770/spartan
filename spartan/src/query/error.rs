use serde::Serialize;
use std::convert::TryFrom;
use tide::{Response, StatusCode};

#[macro_export]
macro_rules! respond {
    ($expr:expr) => {
        match $expr {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(err) => {
                return std::result::Result::Ok(err.respond());
            }
        }
    };
}

#[derive(Serialize)]
pub struct Error<'a> {
    status: u16,
    message: &'a str,
}

impl<'a> Error<'a> {
    pub fn new(status: StatusCode, message: &'a str) -> Self {
        Error {
            status: status.into(),
            message,
        }
    }

    pub fn bad_request() -> Self {
        Error {
            status: StatusCode::BadRequest.into(),
            message: StatusCode::BadRequest.canonical_reason(),
        }
    }

    pub fn respond(&self) -> Response {
        Response::new(StatusCode::try_from(self.status).expect("Unknown error status code"))
            .body_json(self)
            .expect("Cannot convert error to JSON response")
    }
}
