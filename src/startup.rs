use crate::config::{get_config, InitSettings, RunSettings};
use crate::listen_blocks;
use near_indexer::{indexer_init_configs, Indexer};
use std::path::PathBuf;
use uuid::Uuid;

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
    // let db_config = get_configuration();
    // let address = format!("127.0.0.1:{}", db_config.)
    let system = actix::System::new();
    tracing::info!("Indexer was created and configured");
    system.block_on(async move {
        let indexer = Indexer::new(config.indexer);
        let stream = indexer.streamer();
        actix::spawn(async {
            if let Err(e) = listen_blocks(stream).await {
                tracing::error!("`listen_blocks` is terminated with error: {:#}", e)
            }
        });
    });
    system.run()?;

    Ok(())
}
