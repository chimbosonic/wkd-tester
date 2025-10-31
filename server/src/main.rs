use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::error::ErrorBadRequest;
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, middleware, web};
use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
use serde::Deserialize;

mod footer;
mod render;
mod wkd_result;

use render::render;

#[derive(Deserialize)]
struct FormData {
    email: Option<String>,
}

#[get("/api/lookup")]
async fn api(form: web::Query<FormData>) -> Result<impl Responder> {
    let email = match &form.email {
        Some(email) => email,
        None => return Err(ErrorBadRequest("Missing email parameter")),
    };

    let result = wkd_result::get_wkd(email).await;
    let result = web::Json(result)
        .customize()
        .insert_header((CACHE_CONTROL, "no-store"));

    Ok(result)
}

#[get("/")]
async fn lookup(form: web::Query<FormData>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let wkd_result = match &form.email {
        Some(email) => Some(wkd_result::get_wkd(email).await),
        None => None,
    };

    let control_header = match &form.email {
        Some(_) => "no-store",
        None => "public, max-age=604800",
    };

    let mut response = render(hb, "index", &wkd_result);

    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(control_header));

    response
}

fn setup_handlebars() -> web::Data<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory("./static/", DirectorySourceOptions::default())
        .unwrap();
    web::Data::new(handlebars)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = "0.0.0.0";
    let port = 7070;

    let governor_conf = GovernorConfigBuilder::default().finish().unwrap();

    let handlebars_ref = setup_handlebars();

    println!("Starting server on http://{host}:{port}");
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .service(lookup)
            .service(api)
            .wrap(middleware::Logger::new(
                "%a \"%U\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
            ))
            .wrap(Governor::new(&governor_conf))
            .wrap(middleware::Compress::default())
    })
    .bind((host, port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test};

    use super::*;

    #[actix_web::test]
    async fn test_lookup_not_index() {
        let handlebars_ref = setup_handlebars();
        let app =
            test::init_service(App::new().app_data(handlebars_ref.clone()).service(lookup)).await;

        let req = test::TestRequest::get().uri("/not_found").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND)
    }

    #[actix_web::test]
    async fn test_lookup_no_email() {
        let handlebars_ref = setup_handlebars();
        let app =
            test::init_service(App::new().app_data(handlebars_ref.clone()).service(lookup)).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get(CACHE_CONTROL).unwrap(),
            "public, max-age=604800"
        );
    }

    #[actix_web::test]
    async fn test_lookup_email() {
        let handlebars_ref = setup_handlebars();
        let app =
            test::init_service(App::new().app_data(handlebars_ref.clone()).service(lookup)).await;

        let req = test::TestRequest::get()
            .uri("/?email=something")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        let body = test::read_body(res).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("<title>Web Key Directory - Tester</title>"));
        assert!(body_str.contains("InvalidEmailError"));
    }

    #[actix_web::test]
    async fn test_api_not_found() {
        let app = test::init_service(App::new().service(api)).await;

        let req = test::TestRequest::get().uri("/not_found").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND)
    }

    #[actix_web::test]
    async fn test_api_no_email() {
        let app = test::init_service(App::new().service(api)).await;

        let req = test::TestRequest::get().uri("/api/lookup").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = test::read_body(res).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("Missing email parameter"));
    }

    #[actix_web::test]
    async fn test_api_email() {
        let app = test::init_service(App::new().service(api)).await;

        let req = test::TestRequest::get()
            .uri("/api/lookup?email=something")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = test::read_body(res).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("API Response Body: {}", body_str);
        assert_eq!(
            body_str,
            r#"{"user_id":"something","methods":[{"uri":"","key":null,"errors":[{"name":"InvalidEmailError","message":"User ID must be in the format '{local_part}@{domain_part}'"}],"method_type":"Direct","successes":[]},{"uri":"","key":null,"errors":[{"name":"InvalidEmailError","message":"User ID must be in the format '{local_part}@{domain_part}'"}],"method_type":"Advanced","successes":[]}]}"#
        );
    }
}
