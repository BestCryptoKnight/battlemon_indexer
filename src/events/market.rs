use crate::{events, get_config, ExecutionStatusView, IndexerExecutionOutcomeWithReceipt};
use actix_web::web;
use battlemon_models::{
    market::ask::AskForRest, market::bid::BidForRest, market::events::MarketEventKind,
    market::sale::SaleForRest,
};

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
    skip(_outcome_result, client)
)]
pub async fn build_market_request(
    event: MarketEventKind,
    _outcome_result: &ExecutionStatusView,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<reqwest::RequestBuilder> {
    use MarketEventKind::*;

    let config = get_config().await;
    let base_url = config.rest.base_url();

    let request_builder = match event {
        Sale(sale) => {
            let json: SaleForRest = sale.into();
            client.post(format!("{base_url}/sales")).json(&json)
        }
        AddBid(bid) => {
            let json: BidForRest = bid.into();
            client.post(format!("{base_url}/bids")).json(&json)
        }
        AddAsk(ask) => {
            let json: AskForRest = ask.into();
            client.post(format!("{base_url}/asks")).json(&json)
        }
        RemoveBid(bid) => {
            let json: BidForRest = bid.into();
            client.delete(format!("{base_url}/bids")).json(&json)
        }
        RemoveAsk(ask) => {
            let json: AskForRest = ask.into();
            client.delete(format!("{base_url}/asks")).json(&json)
        }
    };

    let ret = request_builder
        .header("Content-Type", "application/json")
        .basic_auth(config.rest.username(), Some(config.rest.password()));

    Ok(ret)
}
