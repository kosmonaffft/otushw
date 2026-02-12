use crate::AppData;
use crate::errors::MyError;
use crate::handlers::posts::CreatePostResponse;
use crate::security::validate_token;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder, get};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime, NaiveDateTime, Utc};
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use tokio_postgres::GenericClient;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    pub posts: Vec<CreatePostResponse>,
}

#[get("/feed")]
async fn get_feed(auth: BearerAuth, app_data: Data<AppData>) -> actix_web::Result<impl Responder> {
    let my_id = validate_token(&auth)?;
    if app_data.use_redis {
        let mut rd_connection = app_data.redis_pool.get().await.map_err(MyError::from)?;
        let redis_key = format!("feed_for:{}", my_id);
        let cache_exists: bool = rd_connection
            .exists(&redis_key)
            .await
            .map_err(MyError::from)?;

        if cache_exists {
            let posts: Vec<String> = rd_connection
                .lrange(&redis_key, 0, 99)
                .await
                .map_err(MyError::from)?;
            let posts_str = posts.join(",");
            let result = format!("{{\"posts\":[{}]\\}}", posts_str);
            Ok(HttpResponse::Ok().body(result))
        } else {
            let pg_connection = app_data.pg_pool.get().await.map_err(MyError::from)?;

            let posts: Vec<CreatePostResponse> =
                get_feed_posts(&my_id, app_data.feed_limit, pg_connection.client()).await?;
            for post in &posts {
                rd_connection
                    .rpush(&redis_key, &post)
                    .await
                    .map_err(MyError::from)?;
            }
            let feed = Feed { posts };
            Ok(HttpResponse::Ok().json(feed))
        }
    } else {
        let pg_connection = app_data.pg_pool.get().await.map_err(MyError::from)?;

        let posts: Vec<CreatePostResponse> =
            get_feed_posts(&my_id, app_data.feed_limit, pg_connection.client()).await?;
        let feed = Feed { posts };
        Ok(HttpResponse::Ok().json(feed))
    }
}

async fn get_feed_posts<C: GenericClient>(
    my_id: &Uuid,
    feed_limit: i32,
    pg_connection: &C,
) -> Result<Vec<CreatePostResponse>, MyError> {
    let query = format!(
        "SELECT p.id, p.user_id, p.ts, p.content
         FROM posts p
         LEFT JOIN friend_relations fr ON p.user_id = fr.to_id
         WHERE fr.from_id = $1
         ORDER BY p.ts DESC
         LIMIT {feed_limit}"
    );
    let posts: Vec<CreatePostResponse> = pg_connection
        .query(&query, &[&my_id])
        .await
        .map_err(MyError::from)?
        .iter()
        .map(|row| {
            let naive_ts: NaiveDateTime = row.get(2);
            CreatePostResponse {
                id: row.get(0),
                user_id: row.get(1),
                ts: DateTime::from_naive_utc_and_offset(naive_ts, Utc),
                body: row.get(3),
            }
        })
        .collect();
    Ok(posts)
}
