use actix_web::HttpResponse;
use actix_web::body::BoxBody;
use argon2::password_hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("bb8 pool error")]
    PoolError(#[from] bb8::RunError<tokio_postgres::Error>),

    #[error("tokio postgres error")]
    TokioPostgresError(#[from] tokio_postgres::Error),

    #[error("password hash error")]
    ArgonError(#[from] password_hash::Error),

    #[error("jwt error")]
    JWTError(#[from] jsonwebtoken::errors::Error),
}

impl actix_web::ResponseError for MyError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            MyError::PoolError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::TokioPostgresError(e) => {
                HttpResponse::InternalServerError().json(e.to_string())
            }
            MyError::ArgonError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::JWTError(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }
}
