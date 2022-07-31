use crate::{
    events, get_config, ExecutionStatusView, IndexerExecutionOutcomeWithReceipt, MarketEventKind,
};
use actix_web::web;
use anyhow::{anyhow, Context};
use chrono::Utc;
use rust_decimal::{Decimal, MathematicalOps};
use std::str::FromStr;

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
        events::handle_request_error(response).await?;
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
            let price = Decimal::from_str(&sale.price)
                .context("Failed to parse price into `Decimal` from `String`")?;

            let price = price / Decimal::new(10, 0).powu(24);

            let json = serde_json::json!({
                "prev_owner": sale.prev_owner,
                "curr_owner": sale.curr_owner,
                "token_id": sale.token_id,
                "price": price,
                "date": Utc::now(),
            });
            let base_url = config.rest.base_url();
            let request = client
                .post(format!("{base_url}/sales"))
                .header("Content-Type", "application/json")
                .basic_auth(config.rest.username(), Some(config.rest.password()))
                .json(&json);

            Ok(request)
        }
        _ => Err(anyhow!("The event is not implemented, {:?}", event)),
    }
}
