use actix_web::{web, HttpResponse};
use handlebars::Handlebars;
use log::error;

use crate::wkd_result::WkdResult;

pub fn render(
    hb: web::Data<Handlebars<'_>>,
    page_path: &str,
    data: &Option<WkdResult>,
) -> HttpResponse {
    match hb.render(page_path, data) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => {
            error!("Template rendering error: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
