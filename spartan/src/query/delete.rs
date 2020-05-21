use super::Query;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub id: Uuid,
}

#[derive(Serialize, new)]
pub struct DeleteResponse {}

impl Query for DeleteRequest {}
impl Query for DeleteResponse {}
