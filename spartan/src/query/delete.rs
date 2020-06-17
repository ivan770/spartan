use serde::{Deserialize, Serialize};
use spartan_lib::core::message::Message;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub id: Uuid,
}

#[derive(Serialize, new)]
pub struct DeleteResponse {
    pub message: Message,
}
