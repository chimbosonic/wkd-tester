use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, middleware, post, web};
use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
mod footer;
mod render;
mod wkd_result;

use render::render;
use serde::Deserialize;

#[get("/api/{user_id}")]
async fn api(user_id: web::Path<String>) -> Result<impl Responder> {
    let wkd_result = wkd_result::get_wkd(&user_id).await;
    Ok(web::Json(wkd_result))
}

#[get("/")]
async fn index_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let mut response = render(hb, "index", &None);
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );
    response
}

#[derive(Deserialize)]
struct FormData {
    email: String,
}

#[post("/")]
async fn lookup(form: web::Form<FormData>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let user_id = form.email.clone();
    let wkd_result = wkd_result::get_wkd(&user_id).await;
    render(hb, "index", &Some(wkd_result))
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
            .service(index_get)
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
