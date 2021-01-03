use once_cell::sync::Lazy;

use crate::config::Config;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::default);

#[macro_export]
macro_rules! init_application_from_data {
    ($data:expr) => {
        crate::http::routing::attach_test_routes($data)
    };
}

#[macro_export]
macro_rules! init_application {
    ($config:expr) => {
        crate::init_application_from_data!(::std::sync::Arc::new(
            crate::node::manager::Manager::new($config)
        ))
    };
}

#[macro_export]
macro_rules! test_request {
    ($filter:ident, $method:expr, $uri:expr) => {
        ::warp::test::request()
            .method($method)
            .path($uri)
            .reply(&$filter)
    };

    ($filter:ident, $method:expr, $uri:expr, $body:expr) => {
        ::warp::test::request()
            .method($method)
            .json($body)
            .path($uri)
            .reply(&$filter)
    };
}

#[macro_export]
macro_rules! test_json_request {
    ($filter:ident, $method:expr, $uri:expr) => {
        ::serde_json::from_slice(test_request!($filter, $method, $uri).await.body()).unwrap()
    };

    ($filter:ident, $method:expr, $uri:expr, $body:expr) => {
        ::serde_json::from_slice(test_request!($filter, $method, $uri, $body).await.body()).unwrap()
    };
}
