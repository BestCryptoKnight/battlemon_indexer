use crate::models::NftEvent;
use crate::{
    events, get_config, ExecutionStatusView, IndexerExecutionOutcomeWithReceipt, NftEventKind,
    TokenMetadata,
};
use actix_web::web;
use anyhow::{anyhow, Context};
use serde_json::json;
use token_metadata_ext::TokenExt;

#[tracing::instrument(
    name = "Deserialize outcome result into nft model",
    skip(outcome_result)
)]
pub fn deserialize_outcome_result_into_token(
    outcome_result: &ExecutionStatusView,
) -> anyhow::Result<TokenExt> {
    let token = match outcome_result {
        ExecutionStatusView::SuccessValue(v) => {
            let bytes = base64::decode(v)?;
            serde_json::from_slice::<TokenExt>(bytes.as_slice())?
        }
        _ => unreachable!(),
    };

    Ok(token)
}

#[tracing::instrument(
    name = "Building request for saving nft contract's event",
    skip(outcome_result, client)
)]
pub async fn build_nft_request(
    event: NftEventKind,
    outcome_result: &ExecutionStatusView,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<reqwest::RequestBuilder> {
    let config = get_config().await;
    // todo:
    //  - extract from rest and indexer models for db and json to separate crate.
    //  - implement conversion from contracts models to them
    match event {
        NftEventKind::NftMint => {
            let TokenExt {
                token_id,
                owner_id,
                metadata,
                model,
                ..
            } = deserialize_outcome_result_into_token(outcome_result)
                .context("Failed to deserialize nft token")?;

            let TokenMetadata {
                title,
                description,
                media,
                copies,
                issued_at,
                expires_at,
                ..
            } = metadata.unwrap();

            let nft_token = json!({
                "owner_id": owner_id,
                "token_id": token_id,
                "title": title,
                "description": description,
                "media": media,
                "media_hash": null,
                "copies": copies,
                "issued_at": issued_at,
                "expires_at": expires_at,
                "model": model,
            });

            let base_url = config.rest.base_url();
            let request = client
                .post(format!("{base_url}/nft_tokens"))
                .header("Content-Type", "application/json")
                .basic_auth(config.rest.username(), Some(config.rest.password()))
                .json(&nft_token);

            Ok(request)
        }
        _ => Err(anyhow!("The event is not implemented, {:?}", event)),
    }
}

#[tracing::instrument(
    name = "Sending request to the rest service to store new nft events to the database",
    skip(outcome, client)
)]
pub async fn handle_nft_events(
    outcome: &IndexerExecutionOutcomeWithReceipt,
    events: Vec<NftEvent>,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<()> {
    for event in events {
        let outcome_result = &outcome.execution_outcome.outcome.status;
        let request = build_nft_request(event.event, outcome_result, client.clone()).await?;
        let response = request.send().await?;
        events::handle_request_error(response).await?;
        tracing::info!("Successfully stored nft event");
    }
    Ok(())
}
