use crate::{events, get_config, ExecutionStatusView, IndexerExecutionOutcomeWithReceipt};
use actix_web::web;
use anyhow::{anyhow, Context};
use battlemon_models::nft::{NftEvent, NftEventKind, NftTokenForRest, TokenExt};

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
        _ => return Err(anyhow!("Outcome result is not success value")),
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
    let base_url = config.rest.base_url();
    match event {
        NftEventKind::NftMint => {
            let token: NftTokenForRest = deserialize_outcome_result_into_token(outcome_result)
                .context("Failed to deserialize nft token")?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert TokenExt to NftTokenForRest"))?;

            let request = client
                .post(format!("{base_url}/nft_tokens"))
                .header("Content-Type", "application/json")
                .basic_auth(config.rest.username(), Some(config.rest.password()))
                .json(&token);

            Ok(request)
        }
        NftEventKind::AssembleNft | NftEventKind::DisassembleNft => {
            let token: NftTokenForRest = deserialize_outcome_result_into_token(outcome_result)
                .context("Failed to deserialize nft token")?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert TokenExt to NftTokenForRest"))?;

            let request = client
                .patch(format!("{base_url}/nft_tokens"))
                .basic_auth(config.rest.username(), Some(config.rest.password()))
                .header("Content-Type", "application/json")
                .json(&token);

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
        let request = build_nft_request(event.event, outcome_result, client.clone()).await;

        if request.is_err() {
            tracing::error!(
                "Failed to build request for saving nft event: {:?}",
                request
            );
            continue;
        }

        let response = request?.send().await?;
        events::handle_response_for_error(response).await?;
    }
    Ok(())
}
