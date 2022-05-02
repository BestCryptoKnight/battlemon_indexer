use battlemon_near_json_rpc_client_wrapper::{JsonRpcWrapper, NEAR_TESTNET_ARCHIVAL_RPC_URL};
use sqlx::postgres::PgPoolOptions;

use battlemon_indexer::consts::get_config;
use battlemon_indexer::{startup, telemetry, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("battlemon_indexer".into(), "info".into());
    telemetry::init_subscriber(subscriber);
    let config = get_config().await;
    tracing::info!("Config was loaded");
    tracing::info!(
        "Root contract account is {}",
        config.contracts.top_contract_id
    );
    tracing::info!("Get connection config for database");
    let pool_conn = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());
    let rpc_client = JsonRpcWrapper::connect(
        NEAR_TESTNET_ARCHIVAL_RPC_URL,
        config.near_credentials.clone().into(),
    );
    tracing::info!("Loading config for NEAR Lake Framework");
    let lake_config = config.aws.lake_config(&rpc_client).await?;
    let stream = near_lake_framework::streamer(lake_config);
    // todo: add to config testnet or mainnet setting for rpc

    startup::run_indexer(stream, pool_conn, rpc_client)
        .await
        .expect("Couldn't run indexer");
    Ok(())
}
