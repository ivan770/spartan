use crate::node::Manager;
use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;

/// Clear queue.
///
/// Doesn't require any input, returns empty response.
pub async fn clear(manager: Data<Manager<'_>>, queue: Path<(String,)>) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0)?.database.lock().await;

    queue.clear();
    Ok(HttpResponse::Ok().json(()))
}

#[cfg(test)]
mod tests {
    use crate::{
        http::query::{push::PushRequest, size::SizeResponse},
        init_application, test_request,
        utils::testing::CONFIG,
    };
    use actix_web::{
        test::{init_service, read_response, read_response_json},
        web::Bytes,
    };

    #[actix_rt::test]
    async fn test_clear() {
        let mut app = init_service(init_application!(&CONFIG)).await;
        let resp = read_response(&mut app, test_request!(post, "/test/clear")).await;
        assert_eq!(resp, Bytes::from_static(b"null"));
    }

    #[actix_rt::test]
    async fn test_clear_fullchain() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        {
            read_response(
                &mut app,
                test_request!(
                    post,
                    "/test",
                    &PushRequest {
                        body: String::from("Hello, world").into_boxed_str(),
                        ..Default::default()
                    }
                ),
            )
            .await;

            let size_res: SizeResponse =
                read_response_json(&mut app, test_request!(get, "/test/size")).await;
            assert_eq!(size_res.size, 1);
        }

        read_response(&mut app, test_request!(post, "/test/clear")).await;

        let size_res: SizeResponse =
            read_response_json(&mut app, test_request!(get, "/test/size")).await;
        assert_eq!(size_res.size, 0);
    }
}
