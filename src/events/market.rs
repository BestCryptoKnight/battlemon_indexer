use crate::{
    events, get_config, ExecutionStatusView, IndexerExecutionOutcomeWithReceipt, MarketEventKind,
    EVENT_PREFIX,
};
use actix_web::web;
use anyhow::{anyhow, Context};
use reqwest::Response;
use serde_json::Value;

#[tracing::instrument(
    name = "Sending request to the rest service to store new market events to the database",
    skip(outcome, events, client)
)]
pub async fn handle_market_events(
    outcome: &IndexerExecutionOutcomeWithReceipt,
    events: Vec<MarketEventKind>,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<()> {
    for event in events {
        let outcome_result = &outcome.execution_outcome.outcome.status;
        let request = build_market_request(event, outcome_result, client.clone()).await?;
        let response = request.send().await?;

        events::handle_request_error(response).await;

        tracing::info!("Successfully stored nft event");
    }
    Ok(())
}

#[tracing::instrument(
    name = "Building request for saving market contract's event",
    skip(event, _outcome_result, client)
)]
pub async fn build_market_request(
    event: MarketEventKind,
    _outcome_result: &ExecutionStatusView,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<reqwest::RequestBuilder> {
    let config = get_config().await;
    match event {
        MarketEventKind::MarketSale(sale) => {
            let base_url = config.rest.base_url();
            let request = client
                .post(format!("{base_url}/sales"))
                .header("Content-Type", "application/json")
                .basic_auth(config.rest.username(), Some(config.rest.password()))
                .json(&sale);

            Ok(request)
        }
        _ => Err(anyhow!("The event is not implemented, {:?}", event)),
    }
}
