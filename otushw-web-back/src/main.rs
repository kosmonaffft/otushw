mod handlers;
mod types;

use crate::handlers::login;
use actix_web::{App, HttpServer};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use refinery::embed_migrations;
use serde::Deserialize;
use std::str::FromStr;
use tokio_postgres::NoTls;

embed_migrations!("migrations");

#[derive(Deserialize, Debug)]
struct Config {
    connection_string: String,
}

type SyncConfig = postgres::config::Config;
type AsyncConfig = tokio_postgres::config::Config;

fn print_migrations() {
    println!("Available migrations:");
    for migration in migrations::runner().get_migrations() {
        println!("- {}: {}", migration.version(), migration.name());
    }
}

fn main() -> std::io::Result<()> {
    print_migrations();
    env_logger::init();

    let app_config = envy::prefixed("OTHW_").from_env::<Config>().unwrap();

    migrate_db(&app_config);

    let system = actix_web::rt::System::new();

    system.block_on(async {
        let async_config = AsyncConfig::from_str(app_config.connection_string.as_str()).unwrap();
        let manager = PostgresConnectionManager::new(async_config, NoTls);
        let pool = Pool::builder().build(manager).await.unwrap();

        HttpServer::new(|| App::new().service(login))
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
