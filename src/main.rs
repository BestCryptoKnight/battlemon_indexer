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
    let stream = near_lake_framework::streamer(config.aws.clone().into());
    // todo: add to config testnet or mainnet setting for rpc
    let rpc_client = JsonRpcWrapper::connect(
        NEAR_TESTNET_ARCHIVAL_RPC_URL,
        config.near_credentials.clone().into(),
    );

    startup::run_indexer(stream, pool_conn, rpc_client)
        .await
        .expect("Couldn't run indexer");
    Ok(())
}
