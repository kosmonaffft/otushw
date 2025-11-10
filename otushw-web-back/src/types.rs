use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub id: Uuid,
    pub exp: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub id: Uuid,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub first_name: String,
    pub second_name: String,
    pub birthdate: NaiveDate,
    pub biography: String,
    pub city: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub first_name: String,
    pub second_name: String,
    pub birthdate: NaiveDate,
    pub biography: String,
    pub city: String,
}
