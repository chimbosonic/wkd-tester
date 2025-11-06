use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::error::ErrorBadRequest;
use actix_web::http::StatusCode;
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, middleware, web};
use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
use serde::Deserialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod footer;
mod render;
mod wkd_result;

use render::render;

#[derive(OpenApi)]
#[openapi(
    paths(api),
    components(schemas(
        wkd_result::WkdResult,
        wkd_result::WkdUriResult,
        wkd_result::WkdMethodType,
        wkd_result::WkdError,
        wkd_result::WkdKey,
        wkd_result::WkdSuccess
    )),
    info(
        title = "WKD Tester API",
        version = "0.1.0",
        description = "API for testing Web Key Directory (WKD) lookups",
        contact(name = "Alexis Lowe", url = "https://chimbosonic.com"),
        license(
            name = "MIT",
            url = "https://github.com/chimbosonic/wkd-tester/blob/master/LICENSE"
        )
    )
)]
struct ApiDoc;

#[derive(Deserialize, utoipa::IntoParams)]
struct FormData {
    /// Email address to lookup in WKD
    email: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/lookup",
    params(FormData),
    responses(
        (status = 200, description = "WKD lookup successful", body = wkd_result::WkdResult),
        (status = 400, description = "Missing email parameter")
    ),
    tag = "WKD Lookup"
)]
#[get("/api/lookup")]
async fn api(form: web::Query<FormData>) -> Result<impl Responder> {
    let email = match &form.email {
        Some(email) => email,
        None => {
            return Err(ErrorBadRequest("Missing email parameter"));
        }
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

fn add_error_header<B>(
    mut res: actix_web::dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>> {
    res.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

fn setup_logging_middleware() -> middleware::Logger {
    middleware::Logger::new("%a \"%U\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T")
}

fn setup_compression_middleware() -> middleware::Compress {
    middleware::Compress::default()
}

fn setup_default_headers_middleware() -> middleware::DefaultHeaders {
    middleware::DefaultHeaders::new().add((CACHE_CONTROL, "public, max-age=604800"))
}

fn setup_error_handlers_middleware<B: 'static>() -> middleware::ErrorHandlers<B> {
    middleware::ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, add_error_header)
        .handler(StatusCode::BAD_REQUEST, add_error_header)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = "0.0.0.0";
    let port = 7070;

    let governor_conf = GovernorConfigBuilder::default().finish().unwrap();

    let handlebars_ref = setup_handlebars();

    let openapi = ApiDoc::openapi();

    println!("Starting server on http://{host}:{port}");
    println!("Swagger UI available at http://{host}:{port}/api-docs/ui/");
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .service(lookup)
            .service(api)
            .service(
                SwaggerUi::new("/api-docs/ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            .wrap(Governor::new(&governor_conf))
            .wrap(setup_error_handlers_middleware())
            .wrap(setup_logging_middleware())
            .wrap(setup_compression_middleware())
            .wrap(setup_default_headers_middleware())
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
        let app = test::init_service(
            App::new()
                .service(api)
                .wrap(setup_error_handlers_middleware())
                .wrap(setup_logging_middleware())
                .wrap(setup_compression_middleware())
                .wrap(setup_default_headers_middleware()),
        )
        .await;

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
        let app = test::init_service(
            App::new()
                .service(api)
                .wrap(setup_error_handlers_middleware())
                .wrap(setup_logging_middleware())
                .wrap(setup_compression_middleware())
                .wrap(setup_default_headers_middleware()),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/lookup?email=something")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        let body = test::read_body(res).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("API Response Body: {}", body_str);
        assert_eq!(
            body_str,
            r#"{"user_id":"something","methods":[{"uri":"","key":null,"errors":[{"name":"InvalidEmailError","message":"User ID must be in the format '{local_part}@{domain_part}'"}],"method_type":"Direct","successes":[]},{"uri":"","key":null,"errors":[{"name":"InvalidEmailError","message":"User ID must be in the format '{local_part}@{domain_part}'"}],"method_type":"Advanced","successes":[]}]}"#
        );
    }
}
