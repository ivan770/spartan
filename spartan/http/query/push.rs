use serde::Deserialize;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Default, serde::Serialize))]
pub struct PushRequest {
    pub body: Box<str>,
    pub offset: Option<i32>,
    pub max_tries: Option<u32>,
    pub timeout: Option<u32>,
    pub delay: Option<u32>,
}
