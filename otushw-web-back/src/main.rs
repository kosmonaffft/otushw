mod errors;
mod handlers;
mod security;
mod types;

use crate::handlers::feed::get_feed;
use crate::handlers::friends::{add_friend, delete_friend};
use crate::handlers::posts::add_post;
use crate::handlers::users::{get_user, login, register_user, search_user};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bb8_redis::RedisConnectionManager;
use log::info;
use serde::Deserialize;
use std::str::FromStr;
use tokio_postgres::NoTls;

mod pg_migrations {
    use refinery::embed_migrations;
    embed_migrations!("pg_migrations");
}

mod citus_migrations {
    use refinery::embed_migrations;
    embed_migrations!("citus_migrations");
}

#[derive(Deserialize, Debug)]
struct Config {
    pg_connection_string: String,
    citus_connection_string: String,
    redis_connection_string: String,
    use_redis: bool,
    feed_limit: i32,
}

#[derive(Clone)]
struct AppData {
    pg_pool: Pool<PostgresConnectionManager<NoTls>>,
    redis_pool: Pool<RedisConnectionManager>,
    use_redis: bool,
    feed_limit: i32,
}

type SyncConfig = postgres::config::Config;
type AsyncConfig = tokio_postgres::config::Config;

fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting up...");

    info!("Parsing connection string...");
    let app_config = envy::prefixed("OTHW_").from_env::<Config>().unwrap();

    info!("Migrating DB...");
    migrate_pg_db(&app_config);

    info!("Starting actix server...");
    let system = actix_web::rt::System::new();
    system.block_on(async {
        let pg_async_config =
            AsyncConfig::from_str(app_config.pg_connection_string.as_str()).unwrap();
        let pg_manager = PostgresConnectionManager::new(pg_async_config, NoTls);
        let pg_pool = Pool::builder().build(pg_manager).await.unwrap();

        let redis_connection_manager =
            RedisConnectionManager::new(app_config.redis_connection_string.as_str()).unwrap();
        let redis_pool: Pool<RedisConnectionManager> = Pool::builder()
            .build(redis_connection_manager)
            .await
            .unwrap();

        let app_data = AppData {
            pg_pool,
            redis_pool,
            use_redis: app_config.use_redis,
            feed_limit: app_config.feed_limit,
        };

        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(app_data.clone()))
                .service(login)
                .service(register_user)
                .service(get_user)
                .service(search_user)
                .service(add_friend)
                .service(delete_friend)
                .service(add_post)
                .service(get_feed)
        })
        .workers(32)
        .bind("0.0.0.0:8080")?
        .run()
        .await
    })
}

fn migrate_pg_db(app_config: &Config) {
    let sync_config = SyncConfig::from_str(app_config.pg_connection_string.as_str()).unwrap();
    let mut sync_postgres = sync_config.connect(NoTls).unwrap();
    pg_migrations::migrations::runner().run(&mut sync_postgres).unwrap();
    sync_postgres.close().unwrap();
}
