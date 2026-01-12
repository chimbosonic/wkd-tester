use actix_web::{
    http::{StatusCode, header::CONTENT_TYPE},
    test,
};

use super::*;

use crate::{config::STATIC_CONTENT_CONFIG, wkd_result::WkdResult};

#[actix_web::test]
async fn test_lookup_not_index() {
    let handlebars_ref = setup_handlebars();
    let app = test::init_service(
        App::new()
            .app_data(handlebars_ref.clone())
            .service(lookup)
            .wrap(setup_error_handlers_middleware())
            .wrap(setup_logging_middleware())
            .wrap(setup_compression_middleware())
            .wrap(setup_default_headers_middleware()),
    )
    .await;

    let req = test::TestRequest::get().uri("/not_found").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND)
}

#[actix_web::test]
async fn test_lookup_no_email() {
    let handlebars_ref = setup_handlebars();

    let app = App::new()
        .app_data(handlebars_ref.clone())
        .service(lookup)
        .wrap(setup_error_handlers_middleware())
        .wrap(setup_logging_middleware())
        .wrap(setup_compression_middleware())
        .wrap(setup_default_headers_middleware());

    #[cfg(feature = "wkd-cache")]
    let app = {
        let cache = setup_cache();
        app.app_data(cache.clone())
    };

    let app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get(CACHE_CONTROL).unwrap(),
        "public, max-age=604800"
    );
    assert_eq!(res.headers().get(CONTENT_TYPE).unwrap(), "text/html");
    let body = test::read_body(res).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("<title>Web Key Directory - Tester</title>"));
}

#[actix_web::test]
async fn test_lookup_email() {
    let handlebars_ref = setup_handlebars();
    let app = App::new()
        .app_data(handlebars_ref.clone())
        .service(lookup)
        .wrap(setup_error_handlers_middleware())
        .wrap(setup_logging_middleware())
        .wrap(setup_compression_middleware())
        .wrap(setup_default_headers_middleware());

    #[cfg(feature = "wkd-cache")]
    let app = {
        let cache = setup_cache();
        app.app_data(cache.clone())
    };

    let app = test::init_service(app).await;

    let req = test::TestRequest::get()
        .uri("/?email=something")
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
    assert_eq!(res.headers().get(CONTENT_TYPE).unwrap(), "text/html");
    let body = test::read_body(res).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("<title>Web Key Directory - Tester</title>"));
    assert!(body_str.contains("InvalidEmailError"));
}

#[actix_web::test]
async fn test_api_not_found() {
    let app = test::init_service(
        App::new()
            .service(api)
            .wrap(setup_error_handlers_middleware())
            .wrap(setup_logging_middleware())
            .wrap(setup_compression_middleware())
            .wrap(setup_default_headers_middleware()),
    )
    .await;

    let req = test::TestRequest::get().uri("/not_found").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
}

#[actix_web::test]
async fn test_api_no_email() {
    let app = App::new()
        .service(api)
        .wrap(setup_error_handlers_middleware())
        .wrap(setup_logging_middleware())
        .wrap(setup_compression_middleware())
        .wrap(setup_default_headers_middleware());

    #[cfg(feature = "wkd-cache")]
    let app = {
        let cache = setup_cache();
        app.app_data(cache.clone())
    };

    let app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/api/lookup").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
    let body = test::read_body(res).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("Missing email parameter"));
}

#[actix_web::test]
async fn test_api_email() {
    let app = App::new()
        .service(api)
        .wrap(setup_error_handlers_middleware())
        .wrap(setup_logging_middleware())
        .wrap(setup_compression_middleware())
        .wrap(setup_default_headers_middleware());

    #[cfg(feature = "wkd-cache")]
    let app = {
        let cache = setup_cache();
        app.app_data(cache.clone())
    };

    let app = test::init_service(app).await;

    let req = test::TestRequest::get()
        .uri("/api/lookup?email=something")
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
    assert_eq!(res.headers().get(CONTENT_TYPE).unwrap(), "application/json");
    let body = test::read_body(res).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    println!("API Response Body: {}", body_str);
    assert!(serde_json::from_str::<WkdResult>(body_str).is_ok());
}

#[actix_web::test]
async fn test_sitemap() {
    let handlebars_ref = setup_handlebars();
    let app = test::init_service(
        App::new()
            .app_data(handlebars_ref.clone())
            .service(serve_sitemap)
            .wrap(setup_error_handlers_middleware())
            .wrap(setup_logging_middleware())
            .wrap(setup_compression_middleware())
            .wrap(setup_default_headers_middleware()),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/.well-known/sitemap.xml")
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get(CACHE_CONTROL).unwrap(),
        "public, max-age=604800"
    );
    assert_eq!(res.headers().get(CONTENT_TYPE).unwrap(), "application/xml");
    let body = test::read_body(res).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    println!("SitemapXML Response Body: {}", body_str);
    assert_eq!(
        body_str,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\r\n  xsi:schemaLocation=\"http://www.sitemaps.org/schemas/sitemap/0.9\r\n      http://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd\">\r\n  <url>\r\n    <loc>{base_url}{rel_path}/</loc>\r\n    <lastmod>2025-11-25T10:01:01+00:00</lastmod>\r\n    <priority>1.00</priority>\r\n  </url>\r\n  <url>\r\n    <loc>{base_url}{rel_path}/api-docs/ui/</loc>\r\n    <lastmod>2025-11-25T10:01:01+00:00</lastmod>\r\n    <priority>0.80</priority>\r\n  </url>\r\n</urlset>",
            base_url = STATIC_CONTENT_CONFIG.base_url,
            rel_path = STATIC_CONTENT_CONFIG.root_path
        )
    );
}
