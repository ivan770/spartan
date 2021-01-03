use std::sync::Arc;

use thiserror::Error as ThisError;
use warp::{
    header::optional,
    hyper::StatusCode,
    reject::{custom, Reject},
    Filter, Rejection,
};

use crate::{actions::RespondableError, config::Config, node::Manager};

#[derive(ThisError, Copy, Clone, Debug)]
pub enum AccessError {
    #[error("Access denied")]
    AccessDenied,
    #[error("Authorization header not found")]
    AuthorizationHeaderNotFound,
    #[error("Incorrect key header")]
    IncorrectKeyHeader,
}

impl RespondableError for AccessError {
    fn status_code(&self) -> StatusCode {
        match self {
            AccessError::AccessDenied => StatusCode::UNAUTHORIZED,
            AccessError::AuthorizationHeaderNotFound | AccessError::IncorrectKeyHeader => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

impl Reject for AccessError {}

pub struct AccessMiddleware {
    config: &'static Config<'static>,
}

pub fn access<T>(
    filter: T,
) -> impl Filter<Extract = T::Extract, Error = Rejection> + Clone + 'static
where
    T: Filter<Extract = (Arc<Manager<'static>>, String), Error = Rejection> + Clone + 'static,
{
    filter
        .and(optional("Authorization"))
        .and_then(
            move |manager: Arc<Manager<'static>>, queue: String, key: Option<String>| async move {
                match AccessMiddleware::new(manager.config()).parse_request(key, &queue) {
                    Ok(_) => Ok((manager, queue)),
                    Err(e) => Err(custom(e)),
                }
            },
        )
        .untuple_one()
}

impl AccessMiddleware {
    fn new(config: &'static Config<'static>) -> Self {
        AccessMiddleware { config }
    }

    fn parse_request(&self, key: Option<String>, queue: &str) -> Result<(), AccessError> {
        if self.has_access_keys() {
            key.ok_or(AccessError::AuthorizationHeaderNotFound)?
                .strip_prefix("Bearer ")
                .map(|token| self.check_access(token, queue))
                .ok_or(AccessError::IncorrectKeyHeader)?
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
    use once_cell::sync::Lazy;
    use warp::{hyper::StatusCode, test::request};

    use crate::config::{key::Key, Config};

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
            crate::http::routing::attach_routes(::std::sync::Arc::new(
                crate::node::manager::Manager::new($config),
            ))
        };
    }

    #[tokio::test]
    async fn test_missing_header() {
        let app = init_application!(&CONFIG);

        let resp = request().path("/test").reply(&app).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_incorrect_key() {
        let app = init_application!(&CONFIG);

        let resp = request()
            .path("/test2")
            .header("Authorization", "Bearer test123")
            .reply(&app)
            .await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_queue_not_allowed() {
        let app = init_application!(&CONFIG);

        let resp = request()
            .path("/test2")
            .header("Authorization", "Bearer testing")
            .reply(&app)
            .await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_queue_allowed() {
        let app = init_application!(&CONFIG);

        let resp = request()
            .path("/test/size")
            .header("Authorization", "Bearer testing")
            .reply(&app)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_wildcard() {
        let app = init_application!(&CONFIG);

        let resp = request()
            .path("/test/size")
            .header("Authorization", "Bearer wildcard")
            .reply(&app)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
