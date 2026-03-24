use crate::AppData;
use crate::errors::MyError;
use crate::handlers::users::find_user;
use crate::security::validate_token;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpResponse, Responder, get, post};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::GenericClient;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct SendMessageRequest {
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMessageResponse {
    pub id: Uuid,
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub ts: DateTime<Utc>,
    pub content: String,
}

#[post("/send_message/{id}")]
async fn send_message(
    id: Path<Uuid>,
    body: Json<SendMessageRequest>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;
    let pg_connection = app_data.pg_pool.get().await.map_err(MyError::from)?;
    let user = find_user(id.into_inner(), pg_connection.client())
        .await
        .map_err(MyError::from)?;
    let distr_id = Uuid::from_u128(my_id.as_u128() ^ user.id.as_u128());
    let citus_connection = app_data.citus_pool.get().await.map_err(MyError::from)?;
    let now = Utc::now();
    let id = Uuid::new_v4();
    let message_insert_query: String = "
        INSERT INTO dialogues (id, distr_id, from_id, to_id, ts, content)
        VALUES ($1, $2, $3, $4, $5, $6);
        "
    .into();
    citus_connection
        .execute(
            &message_insert_query,
            &[
                &id,
                &distr_id,
                &my_id,
                &user.id,
                &now.naive_utc(),
                &body.content,
            ],
        )
        .await
        .map_err(MyError::from)?;
    let response = SendMessageResponse {
        id,
        from_id: my_id,
        to_id: user.id,
        ts: now,
        content: body.content.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/dialogues/{id}")]
async fn get_dialogues(
    id: Path<Uuid>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;
    let pg_connection = app_data.pg_pool.get().await.map_err(MyError::from)?;
    let user = find_user(id.into_inner(), pg_connection.client())
        .await
        .map_err(MyError::from)?;
    let citus_connection = app_data.citus_pool.get().await.map_err(MyError::from)?;
    let query = "
        SELECT id, from_id, to_id, ts, content
        FROM dialogues
        WHERE (from_id = $1 AND to_id = $2) OR (from_id = $2 AND to_id = $1)
        ORDER BY ts ASC
    ";
    let rows = citus_connection
        .query(query, &[&my_id, &user.id])
        .await
        .map_err(MyError::from)?;
    let messages: Vec<SendMessageResponse> = rows
        .iter()
        .map(|row| SendMessageResponse {
            id: row.get(0),
            from_id: row.get(1),
            to_id: row.get(2),
            ts: row.get::<_, chrono::NaiveDateTime>(3).and_utc(),
            content: row.get(4),
        })
        .collect();
    Ok(HttpResponse::Ok().json(messages))
}
