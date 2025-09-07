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

    let control_header = match wkd_result {
        Some(_) => "no-store",
        None => "public, max-age=604800",
    };

    let mut response = render(hb, "index", &wkd_result);

    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(control_header));

    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = "0.0.0.0";
    let port = 7070;

    let governor_conf = GovernorConfigBuilder::default().finish().unwrap();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory("./static/", DirectorySourceOptions::default())
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

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
