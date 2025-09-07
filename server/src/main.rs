use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, middleware, web};
use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
use serde::Deserialize;

mod footer;
mod render;
mod wkd_result;

use render::render;

#[get("/api/{user_id}")]
async fn api(user_id: web::Path<String>) -> Result<impl Responder> {
    let wkd_result = wkd_result::get_wkd(&user_id).await;
    Ok(web::Json(wkd_result))
}

#[derive(Deserialize)]
struct FormData {
    email: Option<String>,
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
            .wrap(middleware::Logger::default())
            .wrap(Governor::new(&governor_conf))
            .wrap(middleware::Compress::default())
    })
    .bind((host, port))?
    .run()
    .await
}
