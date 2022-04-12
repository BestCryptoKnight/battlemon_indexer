use near_lake_framework::LakeConfig;
use sqlx::postgres::PgPoolOptions;

use battlemon_indexer::config::{get_config, RunSettings};
use battlemon_indexer::{startup, telemetry, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("battlemon_indexer".into(), "info".into());
    telemetry::init_subscriber(subscriber);
    let config = get_config::<RunSettings>().expect("Couldn't run indexer");
    tracing::info!("Config was loaded");
    tracing::info!("NFT Contract account is {}", config.contract_acc);
    tracing::info!("Get connection config for database");
    let pool_conn = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());
    let stream = near_lake_framework::streamer(LakeConfig {
        s3_endpoint: None,
        s3_bucket_name: "near-lake-data-testnet".to_string(),
        s3_region_name: "eu-central-1".to_string(),
        start_block_height: config.block,
    });
    startup::run_indexer(stream, pool_conn)
        .await
        .expect("Couldn't run indexer");
    Ok(())
}
