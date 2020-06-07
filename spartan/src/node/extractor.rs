use super::DB;
use crate::{query::Error, Request};
use async_std::sync::MutexGuard;
use tide::StatusCode;

const QUEUE_NOT_FOUND: &str = "Queue not found";

pub struct QueueExtractor<'a>(&'a Request);

impl<'a> QueueExtractor<'a> {
    pub fn new(request: &'a Request) -> Self {
        QueueExtractor(request)
    }

    pub async fn extract(self) -> Result<MutexGuard<'a, DB>, Error<'static>> {
        self.0
            .state()
            .node()
            .get(self.0.param::<String>("queue").unwrap())
            .await
            .ok_or_else(|| Error::new(StatusCode::NotFound, QUEUE_NOT_FOUND))
    }
}
