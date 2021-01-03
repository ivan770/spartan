// use crate::{config::Config, http::middleware::access::Access};
use std::{convert::Infallible, sync::Arc};

use warp::{any, body::json, delete, get, path, post, wrap_fn, Filter, Rejection, Reply};

use super::middleware::access::{access, AccessError};
use crate::{actions::ResponseError, node::Manager};

macro_rules! route {
    ($name:ident) => {
        crate::actions::$name::$name
    };
}

fn with_manager(
    manager: Arc<Manager<'static>>,
) -> impl Filter<Extract = (Arc<Manager<'static>>,), Error = Infallible> + Clone {
    any().map(move || manager.clone())
}

/// Attach routes to Actix service config
pub fn attach_routes(
    manager: Arc<Manager<'static>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let pop = with_manager(manager.clone())
        .and(get())
        .and(path!(String))
        .with(wrap_fn(access))
        .map_async(route!(pop));

    let push = with_manager(manager.clone())
        .and(post())
        .and(path!(String))
        .with(wrap_fn(access))
        .and(json())
        .map_async(route!(push));

    let delete = with_manager(manager.clone())
        .and(delete())
        .and(path!(String))
        .with(wrap_fn(access))
        .and(json())
        .map_async(route!(delete));

    let requeue = with_manager(manager.clone())
        .and(post())
        .and(path!(String / "requeue"))
        .with(wrap_fn(access))
        .and(json())
        .map_async(route!(requeue));

    let clear = with_manager(manager.clone())
        .and(post())
        .and(path!(String / "clear"))
        .with(wrap_fn(access))
        .map_async(route!(clear));

    let size = with_manager(manager)
        .and(get())
        .and(path!(String / "size"))
        .with(wrap_fn(access))
        .map_async(route!(size));

    size.or(clear)
        .or(requeue)
        .or(pop)
        .or(push)
        .or(delete)
        .recover(handle_rejections)
}

#[cfg(test)]
/// Attach test routes to Actix service config
pub fn attach_test_routes(
    manager: Arc<Manager<'static>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    attach_routes(manager)
}

pub async fn handle_rejections(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = rejection.find::<AccessError>() {
        Ok(ResponseError::from(*error).into_response())
    } else {
        Err(rejection)
    }
}
