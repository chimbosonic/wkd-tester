use crate::render;
use crate::wkd_result;
use actix_web::error::ErrorBadRequest;
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE, HeaderValue};
use actix_web::{HttpResponse, Responder, Result, get, web};
use handlebars::Handlebars;
use render::render;
use serde::Deserialize;
use utoipa::OpenApi;

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
pub struct ApiDoc;

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
pub async fn api(form: web::Query<FormData>) -> Result<impl Responder> {
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
pub async fn lookup(form: web::Query<FormData>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
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
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));

    response
}

#[get("/.well-known/sitemap.xml")]
pub async fn serve_sitemap(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let mut response = render(hb, "sitemap", &None);
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    response
}
