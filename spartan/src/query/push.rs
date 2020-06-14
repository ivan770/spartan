use super::Query;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PushRequest {
    pub body: String,
    pub offset: Option<i32>,
    pub max_tries: Option<u32>,
    pub timeout: Option<u32>,
    pub delay: Option<u32>,
}

#[derive(Serialize, new)]
pub struct PushResponse {}

impl Query for PushRequest {}
impl Query for PushResponse {}
