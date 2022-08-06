use anyhow::Context;
use battlemon_indexer::config::{get_config, AppConfig};
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
    update_or_insert_contract_ids(&config, &client).await?;
    startup::run_indexer(stream, client)
        .await
        .expect("Couldn't run indexer");
    Ok(())
}

#[tracing::instrument(
    name = "Update info about Battlemon's contracts ids",
    skip(config, http_client)
)]
async fn update_or_insert_contract_ids(
    config: &AppConfig,
    http_client: &reqwest::Client,
) -> anyhow::Result<()> {
    http_client
        .post(format!("{}/contracts", config.rest.base_url()))
        .basic_auth(config.rest.username(), Some(config.rest.password()))
        .json(&config.contracts)
        .send()
        .await
        .context(
            "Failed to make request to rest serivice for updating info about actual contract's id",
        )?;

    Ok(())
}
