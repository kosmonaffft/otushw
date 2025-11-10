mod errors;
mod handlers;
mod types;

use crate::handlers::{login, register};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::info;
use refinery::embed_migrations;
use serde::Deserialize;
use std::str::FromStr;
use tokio_postgres::NoTls;

embed_migrations!("migrations");

#[derive(Deserialize, Debug)]
struct Config {
    connection_string: String,
}

#[derive(Clone)]
struct AppData {
    pool: Pool<PostgresConnectionManager<NoTls>>,
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
        let async_config = AsyncConfig::from_str(app_config.connection_string.as_str()).unwrap();
        let manager = PostgresConnectionManager::new(async_config, NoTls);
        let pool = Pool::builder().build(manager).await.unwrap();

        let app_data = AppData { pool };

        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(app_data.clone()))
                .service(login)
                .service(register)
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
    })
}

fn migrate_db(app_config: &Config) {
    let sync_config = SyncConfig::from_str(app_config.connection_string.as_str()).unwrap();
    let mut sync_postgres = sync_config.connect(NoTls).unwrap();
    migrations::runner().run(&mut sync_postgres).unwrap();
    sync_postgres.close().unwrap();
}
