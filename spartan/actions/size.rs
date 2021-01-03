use std::sync::Arc;

use spartan_lib::core::dispatcher::SimpleDispatcher;
use warp::reply::{json, Json};

use crate::{actions::Result, http::query::size::SizeResponse, node::Manager};

/// Get queue size.
///
/// Doesn't require any input, returns queue size.
pub async fn size(manager: Arc<Manager<'_>>, name: String) -> Result<Json> {
    let queue = manager.queue(&name)?.database().await;

    Ok(json(&SizeResponse::from(queue.size())))
}

#[cfg(test)]
mod tests {
    use crate::{
        http::query::{push::PushRequest, size::SizeResponse},
        init_application, test_json_request, test_request,
        utils::testing::CONFIG,
    };

    #[tokio::test]
    async fn test_size() {
        let app = init_application!(&CONFIG);

        let size: SizeResponse = test_json_request!(app, "GET", "/test/size");

        assert_eq!(size.size, 0);

        test_request!(
            app,
            "POST",
            "/test",
            &PushRequest {
                body: String::from("Hello, world").into_boxed_str(),
                ..Default::default()
            }
        )
        .await;

        let size: SizeResponse = test_json_request!(app, "GET", "/test/size");

        assert_eq!(size.size, 1);
    }
}
