use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::http::header::CACHE_CONTROL;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use handlebars::DirectorySourceOptions;
use handlebars::Handlebars;
mod render;
mod wkd_result;

use render::render;
use serde::Deserialize;
use serde_json::json;

#[get("/api/{user_id}")]
async fn api(user_id: web::Path<String>) -> impl Responder {
    let wkd_result = wkd_result::get_wkd(&user_id).await;

    serde_json::to_string_pretty(&wkd_result).unwrap()
}

#[get("/")]
async fn index_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let mut response = render(hb, "index", &json!({}));
    response
        .headers_mut()
        .insert(CACHE_CONTROL, "public, max-age=604800".parse().unwrap());
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
    render(hb, "index", &json!(vec![wkd_result]))
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
            .wrap(Logger::default())
            .wrap(Governor::new(&governor_conf))
    })
    .bind((host, port))?
    .run()
    .await
}
