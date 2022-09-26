use crate::{IndexerExecutionOutcomeWithReceipt, EVENT_PREFIX};
use anyhow::Context;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod market;
pub mod nft;

#[tracing::instrument(name = "Handle request error", skip(response))]
pub async fn handle_response_for_error(response: Response) -> anyhow::Result<()> {
    if !response.status().is_success() {
        let error_json = response
            .json::<Value>()
            .await
            .context("Failed to deserialize error from response")?;
        let error_message = error_json
            .get("error")
            .context("Failed to get an error from response")?
            .as_str()
            .unwrap();

        tracing::error!("Failed to store event. Error: {error_message}");
    } else {
        tracing::info!("Successfully stored nft event");
    }

    Ok(())
}

#[tracing::instrument(name = "Collection contracts events from logs", skip(outcome))]
pub fn collect_contract_events<'a, T>(
    outcome: &'a IndexerExecutionOutcomeWithReceipt,
    // _block_timestamp: &u64,
    // _shard_id: &ShardId,
    // _index_in_shard: &mut i32,
) -> Vec<T>
where
    T: Serialize + Deserialize<'a>,
{
    outcome
        .execution_outcome
        .outcome
        .logs
        .iter()
        .filter_map(|log| log.trim().strip_prefix(EVENT_PREFIX))
        .filter_map(|v| {
            serde_json::from_str(v).unwrap_or_else(|e| {
                tracing::error!("Couldn't parse: {}", e);
                None
            })
        })
        .collect()
}
