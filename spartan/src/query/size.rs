use serde::Serialize;

#[derive(Serialize, new)]
pub struct SizeResponse {
    size: usize,
}
