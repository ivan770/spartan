use std::pin::Pin;
use std::task::{Context, Poll};

use crate::config::Config;
use actix_service::{Service, Transform};
use actix_web::{
    dev::ServiceRequest, dev::ServiceResponse, http::StatusCode, Error, ResponseError,
};
use futures_util::future::{ok, Ready};
use std::future::Future;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum AccessError {
    #[error("Access denied")]
    AccessDenied,
    #[error("Authorization header not found")]
    AuthorizationHeaderNotFound,
    #[error("Incorrect key header")]
    IncorrectKeyHeader,
}

impl ResponseError for AccessError {
    fn status_code(&self) -> StatusCode {
        match self {
            AccessError::AccessDenied => StatusCode::UNAUTHORIZED,
            AccessError::AuthorizationHeaderNotFound | AccessError::IncorrectKeyHeader => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

pub struct Access {
    config: &'static Config<'static>,
}

impl Access {
    pub fn new(config: &'static Config) -> Access {
        Access { config }
    }
}

impl<S, B> Transform<S> for Access
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AccessMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AccessMiddleware {
            service,
            config: self.config,
        })
    }
}

pub struct AccessMiddleware<S> {
    service: S,
    config: &'static Config<'static>,
}

#[allow(clippy::type_complexity)]
impl<S, B> Service for AccessMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if let Err(error) = self.parse_request(&req) {
            Box::pin(async move { Err(error.into()) })
        } else {
            let fut = self.service.call(req);

            Box::pin(async move { Ok(fut.await?) })
        }
    }
}

impl<S> AccessMiddleware<S>
where
    S: Service<Request = ServiceRequest>,
{
    fn parse_request(&self, req: &<S as Service>::Request) -> Result<(), AccessError> {
        if self.has_access_keys() {
            req.match_info()
                .get("queue")
                .map(|queue| {
                    req.headers()
                        .get("Authorization")
                        .ok_or(AccessError::AuthorizationHeaderNotFound)?
                        .to_str()
                        .map_err(|_| AccessError::IncorrectKeyHeader)?
                        .strip_prefix("Bearer ")
                        .map(|token| self.check_access(token, queue))
                        .ok_or(AccessError::IncorrectKeyHeader)?
                })
                .unwrap_or_else(|| Ok(()))
        } else {
            Ok(())
        }
    }

    fn check_access(&self, key: &str, queue: &str) -> Result<(), AccessError> {
        self.config
            .access_keys
            .as_ref()
            .expect("Config doesn't have access keys")
            .get(key)
            .map(|key| {
                if key.has_queue(queue) {
                    Ok(())
                } else {
                    Err(AccessError::AccessDenied)
                }
            })
            .ok_or(AccessError::AccessDenied)?
    }

    fn has_access_keys(&self) -> bool {
        self.config.access_keys.is_some()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{key::Key, Config},
        http::routing::attach_routes,
        node::Manager,
        test_request,
    };
    use actix_service::Service;
    use actix_web::{
        http::StatusCode,
        test::{init_service, TestRequest},
        web::Data,
        App,
    };
    use once_cell::sync::Lazy;

    static CONFIG: Lazy<Config> = Lazy::new(|| Config {
        access_keys: Some(
            [
                Key {
                    key: String::from("testing").into_boxed_str(),
                    queues: [String::from("test").into_boxed_str()]
                        .iter()
                        .cloned()
                        .collect(),
                },
                Key {
                    key: String::from("wildcard").into_boxed_str(),
                    queues: [String::from("*").into_boxed_str()]
                        .iter()
                        .cloned()
                        .collect(),
                },
            ]
            .iter()
            .cloned()
            .collect(),
        ),
        ..Default::default()
    });

    macro_rules! init_application {
        ($config:expr) => {
            App::new()
                .app_data(Data::new(Manager::new($config)))
                .configure(|service_config| {
                    attach_routes($config, service_config);
                })
        };
    }

    #[actix_rt::test]
    async fn test_missing_header() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        let status = app
            .call(test_request!(get, "/test"))
            .await
            .unwrap_err()
            .as_response_error()
            .status_code();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_incorrect_key() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        let status = app
            .call(
                TestRequest::get()
                    .uri("/test")
                    .header("Authorization", "Bearer test123")
                    .to_request(),
            )
            .await
            .unwrap_err()
            .as_response_error()
            .status_code();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_queue_not_allowed() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        let status = app
            .call(
                TestRequest::get()
                    .uri("/test2")
                    .header("Authorization", "Bearer testing")
                    .to_request(),
            )
            .await
            .unwrap_err()
            .as_response_error()
            .status_code();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_queue_allowed() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        let status = app
            .call(
                TestRequest::get()
                    .uri("/test/size")
                    .header("Authorization", "Bearer testing")
                    .to_request(),
            )
            .await
            .unwrap()
            .status();

        assert_eq!(status, StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_wildcard() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        let status = app
            .call(
                TestRequest::get()
                    .uri("/test/size")
                    .header("Authorization", "Bearer wildcard")
                    .to_request(),
            )
            .await
            .unwrap()
            .status();

        assert_eq!(status, StatusCode::OK);
    }
}
