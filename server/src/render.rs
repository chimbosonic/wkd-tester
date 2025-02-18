use actix_web::{web, HttpResponse};
use handlebars::Handlebars;
use log::error;
use serde_json::Value;

pub fn render(hb: web::Data<Handlebars<'_>>, page_path: &str, data: &Value) -> HttpResponse {
    match hb.render(page_path, data) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => {
            error!("Template rendering error: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
