use std::sync::Arc;

use spartan_lib::core::dispatcher::SimpleDispatcher;
use warp::reply::{json, Json};

use crate::{
    actions::Result,
    node::{event::Event, Manager},
};

/// Clear queue.
///
/// Doesn't require any input, returns empty response.
pub async fn clear(manager: Arc<Manager<'_>>, name: String) -> Result<Json> {
    let queue = manager.queue(&name)?;

    queue.log_event(&name, &manager, Event::Clear).await?;

    queue.database().await.clear();
    Ok(json(&()))
}

#[cfg(test)]
mod tests {
    use crate::{
        http::query::{push::PushRequest, size::SizeResponse},
        init_application, test_json_request, test_request,
        utils::testing::CONFIG,
    };

    #[tokio::test]
    async fn test_clear() {
        let app = init_application!(&CONFIG);
        let resp = test_request!(app, "POST", "/test/clear").await;
        assert_eq!(resp.body(), "null");
    }

    #[tokio::test]
    async fn test_clear_fullchain() {
        let app = init_application!(&CONFIG);

        {
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

            let size_res: SizeResponse = test_json_request!(app, "GET", "/test/size");
            assert_eq!(size_res.size, 1);
        }

        test_request!(app, "POST", "/test/clear").await;

        let size_res: SizeResponse = test_json_request!(app, "GET", "/test/size");

        assert_eq!(size_res.size, 0);
    }
}
