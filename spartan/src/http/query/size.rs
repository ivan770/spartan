use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct SizeResponse {
    pub size: usize,
}

impl SizeResponse {
    pub fn new(size: usize) -> Self {
        SizeResponse {
            size
        }
    }
}
