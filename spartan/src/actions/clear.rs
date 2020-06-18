use crate::node::Manager;
use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;

/// Clear queue.
///
/// Doesn't require any input, returns empty response.
pub async fn clear(manager: Data<Manager<'_>>, queue: Path<(String,)>) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0).await?;
    queue.clear();
    Ok(HttpResponse::Ok().json(()))
}
