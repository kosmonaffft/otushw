use crate::AppData;
use crate::errors::MyError;
use crate::security::validate_token;
use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, Responder, post};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct CreatePostRequest {
    body: String,
}

#[derive(Serialize, Debug)]
struct CreatePostResponse {
    id: Uuid,
    user_id: Uuid,
    ts: DateTime<Utc>,
    body: String,
}

#[post("/posts")]
async fn add_post(
    body: Json<CreatePostRequest>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let query: String = "
        INSERT INTO posts (id, user_id, ts, content)
        VALUES ($1, $2, $3, $4);
        "
    .into();

    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    connection
        .execute(&query, &[&id, &my_id, &now, &body.0.body])
        .await
        .map_err(MyError::TokioPostgresError)?;
    let response = CreatePostResponse {
        id,
        user_id: my_id,
        ts: now,
        body: body.0.body,
    };
    Ok(HttpResponse::Ok().json(response))
}
