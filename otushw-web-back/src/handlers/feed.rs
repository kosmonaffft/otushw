use crate::AppData;
use crate::errors::MyError;
use crate::security::validate_token;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder, get};
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[get("/feed")]
async fn get_feed(auth: BearerAuth, app_data: Data<AppData>) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;

    let query: String = "
        SELECT p.content
        FROM friend_relations fr
        LEFT JOIN posts p ON p.user_id = fr.to_id
        WHERE fr.from_id = $1
        ORDER BY p.ts
        "
    .into();

    let connection = app_data.pg_pool.get().await.map_err(MyError::PgPoolError)?;
    let rows = connection
        .query(&query, &[&my_id])
        .await
        .map_err(MyError::TokioPostgresError)?;
    Ok(HttpResponse::Ok())
}
