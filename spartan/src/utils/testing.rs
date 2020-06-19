#[macro_export]
macro_rules! init_application {
    ($config:expr) => {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(
                crate::node::manager::Manager::new($config),
            ))
            .configure(crate::routing::attach_routes)
    };
}
