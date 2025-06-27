use actix_web::{HttpResponse, web};
use handlebars::Handlebars;
use log::error;
use serde::Serialize;

use crate::wkd_result::WkdResult;

use crate::footer::{FOOTER_DATA, FooterData};

#[derive(Serialize)]
struct RenderData<'a, T> {
    footer: FooterData,
    data: &'a T,
}

pub fn render(
    hb: web::Data<Handlebars<'_>>,
    page_path: &str,
    data: &Option<WkdResult>,
) -> HttpResponse {
    let render_data = RenderData {
        footer: FOOTER_DATA.clone(),
        data,
    };

    match hb.render(page_path, &render_data) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => {
            error!("Template rendering error: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
