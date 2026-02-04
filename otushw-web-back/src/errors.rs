use actix_web::HttpResponse;
use actix_web::body::BoxBody;
use argon2::password_hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("bb8 pg pool error")]
    PgPoolError(#[from] bb8::RunError<tokio_postgres::Error>),

    #[error("bb8 redis pool error")]
    RedisPoolError(#[from] bb8::RunError<redis::RedisError>),

    #[error("tokio postgres error")]
    TokioPostgresError(#[from] tokio_postgres::Error),

    #[error("password hash error")]
    ArgonError(#[from] password_hash::Error),

    #[error("password hash phc error")]
    ArgonPhcError(#[from] password_hash::phc::Error),

    #[error("jwt error")]
    JWTError(#[from] jsonwebtoken::errors::Error),

    #[error("redis error")]
    RedisError(#[from] redis::RedisError),

    #[error("serde error")]
    SerdeError(#[from] serde_json::error::Error),
}

impl actix_web::ResponseError for MyError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            MyError::PgPoolError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::RedisPoolError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::TokioPostgresError(e) => {
                HttpResponse::InternalServerError().json(e.to_string())
            }
            MyError::ArgonError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::JWTError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::ArgonPhcError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::RedisError(e) => HttpResponse::InternalServerError().json(e.to_string()),
            MyError::SerdeError(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }
}
