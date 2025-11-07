use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub id: String,
    pub password: String,
}
