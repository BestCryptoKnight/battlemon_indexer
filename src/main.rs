use battlemon_indexer::config::get_config;
use battlemon_indexer::{startup, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = telemetry::get_subscriber("battlemon_indexer".into(), "info".into());
    telemetry::init_subscriber(subscriber);
    let config = get_config().await;
    tracing::info!("Loading configuration for NEAR Lake Framework");
    let lake_config = config.near_lake.near_lake_config().await?;
    let client = reqwest::Client::new();
    tracing::info!("Starting up NEAR Lake Framework");
    let stream = near_lake_framework::streamer(lake_config).1;

    startup::run_indexer(stream, client)
        .await
        .expect("Couldn't run indexer");
    Ok(())
}
