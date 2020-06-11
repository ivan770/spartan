use super::Query;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use spartan_lib::core::message::Message;

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub id: Uuid,
}

#[derive(Serialize, new)]
pub struct DeleteResponse {
    pub message: Message
}

impl Query for DeleteRequest {}
impl Query for DeleteResponse {}
