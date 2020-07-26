use serde::Deserialize;
use spartan_lib::uuid::Uuid;

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct RequeueRequest {
    pub id: Uuid,
}
