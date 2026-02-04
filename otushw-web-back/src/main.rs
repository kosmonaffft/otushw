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
use log::info;
use redis::{Client};
use refinery::embed_migrations;
use serde::Deserialize;
use std::str::FromStr;
use tokio_postgres::NoTls;

embed_migrations!("migrations");

#[derive(Deserialize, Debug)]
struct Config {
    pg_connection_string: String,
    redis_connection_string: String,
}

#[derive(Clone)]
struct AppData {
    pg_pool: Pool<PostgresConnectionManager<NoTls>>,
    redis_pool: Pool<Client>,
}

type SyncConfig = postgres::config::Config;
type AsyncConfig = tokio_postgres::config::Config;

fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting up...");

    info!("Parsing connection string...");
    let app_config = envy::prefixed("OTHW_").from_env::<Config>().unwrap();

    info!("Migrating DB...");
    migrate_db(&app_config);

    info!("Starting actix server...");
    let system = actix_web::rt::System::new();
    system.block_on(async {
        let pg_async_config =
            AsyncConfig::from_str(app_config.pg_connection_string.as_str()).unwrap();
        let pg_manager = PostgresConnectionManager::new(pg_async_config, NoTls);
        let pg_pool = Pool::builder().build(pg_manager).await.unwrap();

        let redis_client = Client::open(app_config.redis_connection_string.as_str()).unwrap();
        let redis_pool = Pool::builder().build(redis_client).await.unwrap();

        let app_data = AppData {
            pg_pool,
            redis_pool,
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

fn migrate_db(app_config: &Config) {
    let sync_config = SyncConfig::from_str(app_config.pg_connection_string.as_str()).unwrap();
    let mut sync_postgres = sync_config.connect(NoTls).unwrap();
    migrations::runner().run(&mut sync_postgres).unwrap();
    sync_postgres.close().unwrap();
}
