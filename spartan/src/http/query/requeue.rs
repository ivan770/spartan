use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct RequeueRequest {
    pub id: Uuid,
}
