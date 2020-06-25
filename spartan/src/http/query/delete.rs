use serde::{Deserialize, Serialize};
use spartan_lib::core::message::Message;
use uuid::Uuid;

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct DeleteRequest {
    pub id: Uuid,
}

#[derive(Serialize, new)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct DeleteResponse {
    pub message: Message,
}
