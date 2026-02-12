use crate::AppData;
use crate::errors::MyError;
use crate::security::validate_token;
use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, Responder, post};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime, Utc};
use redis::{AsyncTypedCommands, RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct CreatePostRequest {
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePostResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ts: DateTime<Utc>,
    pub body: String,
}

impl ToRedisArgs for CreatePostResponse {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(serde_json::to_string(&self).unwrap().as_bytes());
    }
}

#[post("/posts")]
async fn add_post(
    body: Json<CreatePostRequest>,
    auth: BearerAuth,
    app_data: Data<AppData>,
) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;
    let connection = app_data.pg_pool.get().await.map_err(MyError::from)?;
    let post_id = Uuid::new_v4();
    let now = Utc::now();
    let post_insert_query: String = "
        INSERT INTO posts (id, user_id, ts, content)
        VALUES ($1, $2, $3, $4);
        "
    .into();
    connection
        .execute(
            &post_insert_query,
            &[&post_id, &my_id, &now.naive_utc(), &body.body],
        )
        .await
        .map_err(MyError::from)?;
    let response = CreatePostResponse {
        id: post_id,
        user_id: my_id,
        ts: now,
        body: body.0.body,
    };

    if app_data.use_redis {
        let get_friends_query: String =
            "SELECT fr.to_id FROM friend_relations AS fr WHERE fr.from_id = $1".into();

        let my_friends_ids: Vec<Uuid> = connection
            .query(&get_friends_query, &[&my_id])
            .await
            .map_err(MyError::from)?
            .iter()
            .map(|row| row.get(0))
            .collect();

        let mut rd_connection = app_data.redis_pool.get().await.map_err(MyError::from)?;
        for id in my_friends_ids {
            let redis_key = format!("feed_for:{}", id);
            let exists: bool = rd_connection
                .exists(&redis_key)
                .await
                .map_err(MyError::from)?;

            if exists {
                rd_connection
                    .lpush(&redis_key, &response)
                    .await
                    .map_err(MyError::from)?;

                rd_connection
                    .ltrim(&redis_key, 0, app_data.feed_limit as isize)
                    .await
                    .map_err(MyError::from)?;
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}
