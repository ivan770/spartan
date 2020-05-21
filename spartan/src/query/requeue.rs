use super::Query;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RequeueRequest {
    pub id: Uuid,
}

#[derive(Serialize, new)]
pub struct RequeueResponse {}

impl Query for RequeueRequest {}
impl Query for RequeueResponse {}
