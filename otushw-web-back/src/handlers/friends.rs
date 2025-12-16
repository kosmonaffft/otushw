use crate::AppData;
use crate::errors::MyError;
use crate::security::validate_token;
use actix_web::web::{Data, Path};
use actix_web::{HttpResponse, Responder, delete, post};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct AddFriendResponse {
    from_id: Uuid,
    to_id: Uuid,
}

#[post("/friends/{id}")]
async fn add_friend(
    id: Path<Uuid>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;

    let query: String = "
        INSERT INTO friend_relations (from_id, to_id)
        VALUES ($1, $2)
        ON CONFLICT (from_id, to_id) DO NOTHING;
        "
    .into();

    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let to_id = id.into_inner();
    connection
        .execute(&query, &[&to_id, &my_id])
        .await
        .map_err(MyError::TokioPostgresError)?;
    let response = AddFriendResponse {
        from_id: my_id,
        to_id,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/friends/{id}")]
async fn delete_friend(
    id: Path<Uuid>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;

    let query: String = "
        DELETE FROM friend_relations
        WHERE from_id = $1 AND to_id = $2;
        "
    .into();

    let connection = app_data.pool.get().await.map_err(MyError::PoolError)?;
    let to_id = id.into_inner();
    connection
        .execute(&query, &[&to_id, &my_id])
        .await
        .map_err(MyError::TokioPostgresError)?;
    Ok(HttpResponse::NoContent())
}
