use serde::Serialize;
use tide::{Body, Response as TideResponse, StatusCode};

pub mod clear;
pub mod delete;
pub mod error;
pub mod pop;
pub mod push;
pub mod requeue;
pub mod size;

pub use error::Error;

/// Trait for HTTP requests and responses
pub trait Query {}

/// Trait for converting custom types into Tide responses
pub trait Response {
    /// Convert custom type into Tide response with 200 status code
    fn respond(&self) -> TideResponse;

    /// Convert custom type into Tide response with custom status code
    fn respond_with_status(&self, status: StatusCode) -> TideResponse;
}

impl<T> Response for T
where
    T: Serialize + Query,
{
    fn respond(&self) -> TideResponse {
        self.respond_with_status(StatusCode::Ok)
    }

    fn respond_with_status(&self, status: StatusCode) -> TideResponse {
        if let Ok(body) = Body::from_json(self) {
            let mut response = TideResponse::new(status);
            response.set_body(body);
            response
        } else {
            Error::new(
                StatusCode::InternalServerError,
                "Unable to serialize response to JSON",
            )
            .respond()
        }
    }
}
