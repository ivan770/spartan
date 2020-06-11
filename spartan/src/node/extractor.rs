use super::DB;
use crate::{query::Error, Request};
use async_std::sync::MutexGuard;
use tide::StatusCode;

/// Error message
const QUEUE_NOT_FOUND: &str = "Queue not found";

/// Request queue extractor
pub struct QueueExtractor<'a>(&'a Request);

impl<'a> QueueExtractor<'a> {
    /// Create new extractor from request
    pub fn new(request: &'a Request) -> Self {
        QueueExtractor(request)
    }

    /// Extract queue from request
    pub async fn extract(self) -> Result<MutexGuard<'a, DB>, Error<'static>> {
        self.0
            .state()
            .node()
            .get(self.0.param::<String>("queue").unwrap())
            .await
            .ok_or_else(|| Error::new(StatusCode::NotFound, QUEUE_NOT_FOUND))
    }
}
