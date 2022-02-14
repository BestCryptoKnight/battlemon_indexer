use std::path::PathBuf;

use near_indexer::{indexer_init_configs, Indexer};
use uuid::Uuid;

use crate::config::{get_config, InitSettings, RunSettings};
use crate::listen_blocks;

#[tracing::instrument(
    name = "Initialize indexer",
    skip(home_dir),
    fields(id = %Uuid::new_v4())
)]
pub fn init_indexer(home_dir: PathBuf) -> crate::Result<()> {
    let config = get_config::<InitSettings>()?;
    tracing::info!("Configuration was loaded");
    indexer_init_configs(&home_dir, config.indexer)?;
    tracing::info!("Indexer was initialized");
    Ok(())
}

#[tracing::instrument(
    name = "Run indexer",
    skip(home_dir),
    fields(id = %Uuid::new_v4())
)]
pub fn run_indexer(home_dir: PathBuf) -> crate::Result<()> {
    let mut config = get_config::<RunSettings>().expect("Couldn't run indexer");
    tracing::info!("Config was loaded");
    config.indexer.home_dir = home_dir;
    let system = actix::System::new();
    system.block_on(async move {
        tracing::info!("Main account is {}", config.main_account);
        tracing::info!("Get connection config for database");
        let address = config.database.connection_string();
        tracing::info!("Using postgres database at: {}", &address);
        let conn = sqlx::PgPool::connect(&address)
            .await
            .expect("Failed to connect to Postgres");
        let conn = actix_web::web::Data::new(conn);

        tracing::info!("Create indexer with configuration: {:?}", config.indexer);
        let indexer = Indexer::new(config.indexer);
        let stream = indexer.streamer();
        actix::spawn(async move {
            if let Err(e) = listen_blocks(stream, conn.clone()).await {
                tracing::error!("`listen_blocks` is terminated with error: {:#}", e)
            }
        });
    });
    system.run()?;

    Ok(())
}
