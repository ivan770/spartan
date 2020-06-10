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

pub trait Query {}

pub trait Response {
    fn respond(&self) -> TideResponse;
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
