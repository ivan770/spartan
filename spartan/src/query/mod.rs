use serde::Serialize;
use tide::{Response as TideResponse, StatusCode};

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
        TideResponse::new(status)
            .body_json(self)
            .unwrap_or_else(|_| {
                Error::new(
                    StatusCode::InternalServerError,
                    "Unable to serialize response to JSON",
                )
                .respond()
            })
    }
}
