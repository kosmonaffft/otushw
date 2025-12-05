use crate::errors::MyError;
use crate::types::Claims;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

const SECRET: &'static [u8] = b"mySuper_SECRETTT!!!1111456";

pub fn hash_password(password: &String) -> Result<String, MyError> {
    let salt = SaltString::generate();
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(MyError::ArgonError)?
        .to_string();
    Ok(password_hash)
}

pub fn check_password(password: &String, hash_str: &String) -> Result<(), MyError> {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(&hash_str).map_err(MyError::ArgonError)?;
    argon2
        .verify_password(&password.as_bytes(), &hash)
        .map_err(MyError::ArgonError)?;
    Ok(())
}

pub fn generate_token(id: Uuid) -> Result<String, MyError> {
    let exp = (Utc::now() + Duration::minutes(15)).naive_utc();
    let claims = Claims { id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .map_err(MyError::JWTError)?;
    Ok(token)
}

pub fn validate_token(auth: &BearerAuth) -> Result<(), MyError> {
    let validation = Validation::default();
    decode::<Claims>(auth.token(), &DecodingKey::from_secret(SECRET), &validation)
        .map_err(MyError::JWTError)?;
    Ok(())
}
