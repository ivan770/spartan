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
    let mut queue = manager.queue(&queue.0).await?;
    queue.clear();
    Ok(HttpResponse::Ok().json(()))
}

#[cfg(test)]
mod tests {
    use crate::{
        init_application,
        query::{push::PushRequest, size::SizeResponse},
        server::Config,
    };
    use actix_web::{
        test::{init_service, read_response, read_response_json, TestRequest},
        web::Bytes,
    };
    use once_cell::sync::Lazy;
    use std::path::PathBuf;

    static CONFIG: Lazy<Config> = Lazy::new(|| Config {
        gc_timer: 10,
        persistence_timer: 30,
        path: PathBuf::from("./db"),
        queues: vec![String::from("test")],
    });

    #[actix_rt::test]
    async fn test_clear() {
        let mut app = init_service(init_application!(&CONFIG)).await;
        let req = TestRequest::post().uri("/test/clear").to_request();
        let resp = read_response(&mut app, req).await;
        assert_eq!(resp, Bytes::from_static(b"null"));
    }

    #[actix_rt::test]
    async fn test_clear_fullchain() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        {
            let post_req = TestRequest::post()
                .set_json(&PushRequest {
                    body: String::from("Hello, world"),
                    ..Default::default()
                })
                .uri("/test")
                .to_request();
            read_response(&mut app, post_req).await;

            let size_req = TestRequest::get().uri("/test/size").to_request();
            let size_res: SizeResponse = read_response_json(&mut app, size_req).await;
            assert_eq!(size_res.size, 1);
        }

        let req = TestRequest::post().uri("/test/clear").to_request();
        read_response(&mut app, req).await;

        let size_req = TestRequest::get().uri("/test/size").to_request();
        let size_res: SizeResponse = read_response_json(&mut app, size_req).await;
        assert_eq!(size_res.size, 0);
    }
}
