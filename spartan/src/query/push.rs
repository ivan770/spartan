use serde::Deserialize;

#[derive(Deserialize)]
pub struct PushRequest {
    pub body: String,
    pub offset: Option<i32>,
    pub max_tries: Option<u32>,
    pub timeout: Option<u32>,
    pub delay: Option<u32>,
}
