use crate::types::LoginRequest;
use actix_web::{HttpResponse, Responder, post, web};

const SECRET: &'static [u8] = b"mySuper_SECRETTT!!!1111456";

#[post("/login")]
async fn login(login_request: web::Json<LoginRequest>) -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok().body("OK"))
}
