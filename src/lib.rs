use self::config::get_config;
use crate::models::{NftEvent, NftEventKind};
use actix_web::web;
use anyhow::{anyhow, Context};
use consts::EVENT_PREFIX;
use futures::try_join;
use near_lake_framework::near_indexer_primitives::views::ExecutionStatusView;
use near_lake_framework::near_indexer_primitives::{
    IndexerExecutionOutcomeWithReceipt, IndexerShard, StreamerMessage,
};
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use token_metadata_ext::{TokenExt, TokenMetadata};

pub mod config;
pub mod consts;
pub mod models;
pub mod startup;
pub mod telemetry;

#[tracing::instrument(name = "Handling streamer message", skip(streamer_message, client))]
async fn handle_message(
    streamer_message: StreamerMessage,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<()> {
    let nft_events = async {
        for shard in &streamer_message.shards {
            collect_and_store_contracts_events(
                shard,
                // &streamer_message.block.header.height,
                client.clone(),
            )
            .await?;
        }

        Ok::<(), anyhow::Error>(())
    };

    try_join!(nft_events)?;

    Ok(())
}

#[tracing::instrument(
    name = "Collecting nft events and store it in database",
    skip(shard, client)
)]
async fn collect_and_store_contracts_events(
    shard: &IndexerShard,
    // block_height: &u64,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<()> {
    // let mut index_in_shard: i32 = 0;
    let (_, nft, market) = get_config().await.contracts.ids();
    for outcome in &shard.receipt_execution_outcomes {
        match outcome.receipt.receiver_id.as_ref() {
            id if id == nft.as_ref() => {
                let nft_events = collect_nft_events(outcome);
                handle_nft_events(outcome, nft_events, client.clone()).await?;
            }
            // id if id == market.as_ref() => {
            // let market_events = collect_market_events(
            //     outcome,
            //     block_height,
            //     &shard.shard_id,
            //     &mut index_in_shard,
            // );
            // if !market_events.is_empty() {
            //     insert_market_events(outcome, market_events, &db, &rpc_client).await?;
            // }
            // todo!()
            // }
            _ => continue,
        }
    }

    Ok(())
}

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
    skip(event, outcome_result, client)
)]
pub async fn build_nft_request(
    NftEvent { event, .. }: NftEvent,
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
    name = "Sending request to the rest service to store new events to the database",
    skip(outcome, events, client)
)]
pub async fn handle_nft_events(
    outcome: &IndexerExecutionOutcomeWithReceipt,
    events: Vec<NftEvent>,
    client: web::Data<reqwest::Client>,
) -> anyhow::Result<()> {
    for event in events {
        let outcome_result = &outcome.execution_outcome.outcome.status;
        let request = build_nft_request(event, outcome_result, client.clone()).await?;
        let response = request.send().await?;
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

            tracing::error!("Failed to store nft event. Error: {error_message}");
            panic!();
        }

        tracing::info!("Successfully stored nft event");
    }
    Ok(())
}

#[tracing::instrument(name = "Collection NFT events from logs", skip(outcome))]
fn collect_nft_events(
    outcome: &IndexerExecutionOutcomeWithReceipt,
    // _block_timestamp: &u64,
    // _shard_id: &ShardId,
    // _index_in_shard: &mut i32,
) -> Vec<NftEvent> {
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

//         let event: nft_types::NearEvent = match serde_json::from_str::<'_, nft_types::NearEvent>(
//             log[prefix.len()..].trim(),
//         ) {
//             Ok(result) => result,
//             Err(err) => {
//                 warn!(
//                     target: crate::INDEXER_FOR_EXPLORER,
//                     "NFT: provided event log does not correspond to any of formats defined in NEP. Will ignore this event. \n {:#?} \n{:#?}",
//                     err,
//                     untrimmed_log,
//                 );
//                 return None;
//             }
//         };
//
//         let nft_types::NearEvent::Nep171(nep171_event) = event;
//         Some(nep171_event)
//     });
//
//     let mut nft_events = Vec::new();
//     let contract_id = &outcome.receipt.receiver_id;
//     for log in event_logs {
//         match log.event_kind {
//             nft_types::Nep171EventKind::NftMint(mint_events) => {
//                 for mint_event in mint_events {
//                     let memo = mint_event.memo.unwrap_or_else(|| "".to_string());
//                     for token_id in mint_event.token_ids {
//                         nft_events.push(
//                             models::assets::non_fungible_token_events::NonFungibleTokenEvent {
//                                 emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
//                                 emitted_at_block_timestamp: BigDecimal::from(*block_timestamp),
//                                 emitted_in_shard_id: BigDecimal::from(*shard_id),
//                                 emitted_index_of_event_entry_in_shard: *index_in_shard,
//                                 emitted_by_contract_account_id: contract_id.to_string(),
//                                 token_id: token_id.escape_default().to_string(),
//                                 event_kind: models::enums::NftEventKind::Mint,
//                                 token_old_owner_account_id: "".to_string(),
//                                 token_new_owner_account_id: mint_event
//                                     .owner_id
//                                     .escape_default()
//                                     .to_string(),
//                                 token_authorized_account_id: "".to_string(),
//                                 event_memo: memo.escape_default().to_string(),
//                             },
//                         );
//                         *index_in_shard += 1;
//                     }
//                 }
//             }
//             nft_types::Nep171EventKind::NftTransfer(transfer_events) => {
//                 for transfer_event in transfer_events {
//                     let authorized_id = transfer_event
//                         .authorized_id
//                         .unwrap_or_else(|| "".to_string());
//                     let memo = transfer_event.memo.unwrap_or_else(|| "".to_string());
//                     for token_id in transfer_event.token_ids {
//                         nft_events.push(
//                             models::assets::non_fungible_token_events::NonFungibleTokenEvent {
//                                 emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
//                                 emitted_at_block_timestamp: BigDecimal::from(*block_timestamp),
//                                 emitted_in_shard_id: BigDecimal::from(*shard_id),
//                                 emitted_index_of_event_entry_in_shard: *index_in_shard,
//                                 emitted_by_contract_account_id: contract_id.to_string(),
//                                 token_id: token_id.escape_default().to_string(),
//                                 event_kind: models::enums::NftEventKind::Transfer,
//                                 token_old_owner_account_id: transfer_event
//                                     .old_owner_id
//                                     .escape_default()
//                                     .to_string(),
//                                 token_new_owner_account_id: transfer_event
//                                     .new_owner_id
//                                     .escape_default()
//                                     .to_string(),
//                                 token_authorized_account_id: authorized_id
//                                     .escape_default()
//                                     .to_string(),
//                                 event_memo: memo.escape_default().to_string(),
//                             },
//                         );
//                         *index_in_shard += 1;
//                     }
//                 }
//             }
//             nft_types::Nep171EventKind::NftBurn(burn_events) => {
//                 for burn_event in burn_events {
//                     let authorized_id = &burn_event.authorized_id.unwrap_or_else(|| "".to_string());
//                     let memo = burn_event.memo.unwrap_or_else(|| "".to_string());
//                     for token_id in burn_event.token_ids {
//                         nft_events.push(
//                             models::assets::non_fungible_token_events::NonFungibleTokenEvent {
//                                 emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
//                                 emitted_at_block_timestamp: BigDecimal::from(*block_timestamp),
//                                 emitted_in_shard_id: BigDecimal::from(*shard_id),
//                                 emitted_index_of_event_entry_in_shard: *index_in_shard,
//                                 emitted_by_contract_account_id: contract_id.to_string(),
//                                 token_id: token_id.escape_default().to_string(),
//                                 event_kind: models::enums::NftEventKind::Burn,
//                                 token_old_owner_account_id: burn_event
//                                     .owner_id
//                                     .escape_default()
//                                     .to_string(),
//                                 token_new_owner_account_id: "".to_string(),
//                                 token_authorized_account_id: authorized_id
//                                     .escape_default()
//                                     .to_string(),
//                                 event_memo: memo.escape_default().to_string(),
//                             },
//                         );
//                         *index_in_shard += 1;
//                     }
//                 }
//             }
//         }
//     }
//     nft_events
// }
