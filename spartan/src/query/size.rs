use super::Query;
use serde::Serialize;

#[derive(Serialize, new)]
pub struct SizeResponse {
    size: usize,
}

impl Query for SizeResponse {}
