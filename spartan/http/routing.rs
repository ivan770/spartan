// use crate::{config::Config, http::middleware::access::Access};
use std::{convert::Infallible, sync::Arc};

use warp::{any, body::json, delete, get, path, post, Filter, Rejection, Reply};

use crate::node::Manager;

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
        .map_async(route!(pop));

    let push = with_manager(manager.clone())
        .and(post())
        .and(path!(String))
        .and(json())
        .map_async(route!(push));

    let delete = with_manager(manager.clone())
        .and(delete())
        .and(path!(String))
        .and(json())
        .map_async(route!(delete));

    let requeue = with_manager(manager.clone())
        .and(post())
        .and(path!(String / "requeue"))
        .and(json())
        .map_async(route!(requeue));

    let clear = with_manager(manager.clone())
        .and(post())
        .and(path!(String / "clear"))
        .map_async(route!(clear));

    let size = with_manager(manager)
        .and(get())
        .and(path!(String / "size"))
        .map_async(route!(size));

    size.or(clear).or(requeue).or(pop).or(push).or(delete)
}

#[cfg(test)]
/// Attach test routes to Actix service config
pub fn attach_test_routes(
    manager: Arc<Manager<'static>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    attach_routes(manager)
}
