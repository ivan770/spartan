use super::Query;
use serde::Serialize;

#[derive(Serialize, new)]
pub struct ClearResponse {}

impl Query for ClearResponse {}
