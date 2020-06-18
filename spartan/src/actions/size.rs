use crate::{node::Manager, query::size::SizeResponse};
use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;

/// Get queue size.
///
/// Doesn't require any input, returns queue size.
pub async fn size(manager: Data<Manager<'_>>, queue: Path<(String,)>) -> Result<HttpResponse> {
    let queue = manager.queue(&queue.0).await?;
    Ok(HttpResponse::Ok().json(SizeResponse::new(queue.size())))
}
