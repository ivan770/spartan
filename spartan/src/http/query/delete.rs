use serde::{Deserialize, Serialize};
use spartan_lib::{
    core::message::Message,
    uuid::Uuid
};

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct DeleteRequest {
    pub id: Uuid,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct DeleteResponse {
    pub message: Message,
}

impl DeleteResponse {
    pub fn new(message: Message) -> Self {
        DeleteResponse {
            message
        }
    }
}