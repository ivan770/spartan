use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RequeueRequest {
    pub id: Uuid,
}
