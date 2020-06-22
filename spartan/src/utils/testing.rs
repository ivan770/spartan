#[macro_export]
macro_rules! init_application_from_data {
    ($data:expr) => {
        actix_web::App::new()
            .app_data($data)
            .configure(crate::routing::attach_routes)
    };
}

#[macro_export]
macro_rules! init_application {
    ($config:expr) => {
        crate::init_application_from_data!(actix_web::web::Data::new(
            crate::node::manager::Manager::new($config)
        ))
    };
}

#[macro_export]
macro_rules! test_request {
    ($method:ident, $uri:expr) => {
        actix_web::test::TestRequest::$method()
            .uri($uri)
            .to_request()
    };

    ($method:ident, $uri:expr, $body:expr) => {
        actix_web::test::TestRequest::$method()
            .set_json($body)
            .uri($uri)
            .to_request()
    };
}
