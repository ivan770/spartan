use serde::Serialize;

#[derive(Serialize, new)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct SizeResponse {
    pub size: usize,
}
