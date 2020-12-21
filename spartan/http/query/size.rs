use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct SizeResponse {
    pub size: usize,
}

impl From<usize> for SizeResponse {
    fn from(size: usize) -> Self {
        SizeResponse { size }
    }
}
