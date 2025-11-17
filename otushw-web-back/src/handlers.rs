use crate::errors::MyError;
use crate::types::{Claims, LoginRequest, RegisterRequest, RegisterResponse};
use crate::AppData;
use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

const SECRET: &'static [u8] = b"mySuper_SECRETTT!!!1111456";

#[get("/get/{id}")]
async fn get(
    id: web::Path<Uuid>,
    auth: BearerAuth,
    app_data: web::Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let validation = Validation::default();
    decode::<Claims>(auth.token(), &DecodingKey::from_secret(SECRET), &validation)
        .map_err(MyError::JWTError)?;
    let uuid: Uuid = id.into_inner();
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let row = connection
        .query_one(
            "SELECT u.first_name, u.second_name, u.is_male, u.birthdate, u.biography, u.city FROM users u WHERE u.id = $1",
            &[&uuid],
        )
        .await
        .map_err(MyError::TokioPostgresError)?;
    let result = RegisterResponse {
        id: uuid,
        first_name: row.try_get(0).map_err(MyError::TokioPostgresError)?,
        second_name: row.try_get(1).map_err(MyError::TokioPostgresError)?,
        is_male: row.try_get(2).map_err(MyError::TokioPostgresError)?,
        birthdate: row.try_get(3).map_err(MyError::TokioPostgresError)?,
        biography: row.try_get(4).map_err(MyError::TokioPostgresError)?,
        city: row.try_get(5).map_err(MyError::TokioPostgresError)?,
    };
    Ok(HttpResponse::Ok().json(result))
}

#[post("/login")]
async fn login(
    request: web::Json<LoginRequest>,
    app_data: web::Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let row = connection
        .query_one(
            "SELECT u.id, u.password_hash FROM users u WHERE u.id = $1",
            &[&request.id],
        )
        .await
        .map_err(MyError::TokioPostgresError)?;
    let hash_str: String = row.try_get(1).map_err(MyError::TokioPostgresError)?;
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(&hash_str).map_err(MyError::ArgonError)?;
    argon2
        .verify_password(&request.password.as_bytes(), &hash)
        .map_err(MyError::ArgonError)?;
    let now = Utc::now();
    let expiration = now + Duration::minutes(15);
    let claims = Claims {
        id: request.id,
        exp: expiration.naive_utc(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .map_err(MyError::JWTError)?;
    Ok(HttpResponse::Ok().json(token))
}

#[post("/register")]
async fn register(
    request: web::Json<RegisterRequest>,
    app_data: web::Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let salt = SaltString::generate();
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(MyError::ArgonError)?
        .to_string();
    let id = Uuid::new_v4();
    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let row = connection
        .query_one(
            "INSERT INTO users
                      (id, password_hash, first_name, second_name, is_male, birthdate, biography, city)
                      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                      RETURNING id, first_name, second_name, is_male, birthdate, biography, city",
            &[
                &id,
                &password_hash,
                &request.first_name,
                &request.second_name,
                &request.is_male,
                &request.birthdate,
                &request.biography,
                &request.city,
            ],
        )
        .await
        .map_err(MyError::TokioPostgresError)?;
    let response = RegisterResponse {
        id: row.try_get(0).map_err(MyError::TokioPostgresError)?,
        first_name: row.try_get(1).map_err(MyError::TokioPostgresError)?,
        second_name: row.try_get(2).map_err(MyError::TokioPostgresError)?,
        is_male: row.try_get(3).map_err(MyError::TokioPostgresError)?,
        birthdate: row.try_get(4).map_err(MyError::TokioPostgresError)?,
        biography: row.try_get(5).map_err(MyError::TokioPostgresError)?,
        city: row.try_get(6).map_err(MyError::TokioPostgresError)?,
    };
    Ok(HttpResponse::Ok().json(response))
}
