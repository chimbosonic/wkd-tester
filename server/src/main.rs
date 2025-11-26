mod config;
mod render;
mod routes;
mod wkd_result;

use actix_web::http::StatusCode;
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{App, HttpServer, Result, middleware, web};
use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
use routes::{ApiDoc, api, lookup, serve_sitemap};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::SERVER_CONFIG;

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
    middleware::Logger::new("%a %{r}a \"%{Host}i\" \"%U\" \"%{User-Agent}i\" %s %b %D")
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

    let host = SERVER_CONFIG.host;
    let port = SERVER_CONFIG.port;

    let handlebars_ref = setup_handlebars();

    let openapi = ApiDoc::openapi();

    println!("Starting server on http://{host}:{port}");
    println!("Swagger UI available at http://{host}:{port}/api-docs/ui/");
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .service(lookup)
            .service(api)
            .service(serve_sitemap)
            .service(
                SwaggerUi::new("/api-docs/ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
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
mod tests;
