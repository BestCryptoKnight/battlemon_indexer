use self::config::get_config;
use crate::models::{MarketEventKind, NftEventKind, T};
use actix_web::web;
use anyhow::{anyhow, Context};
use consts::EVENT_PREFIX;
use events::{market, nft};
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
pub mod events;
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
    name = "Collecting contracts events and store it in the database",
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
                let nft_events = events::collect_contract_events(outcome);
                nft::handle_nft_events(outcome, nft_events, client.clone()).await?;
            }
            id if id == market.as_ref() => {
                let market_events = events::collect_contract_events(outcome);
                market::handle_market_events(outcome, market_events, client.clone()).await?;
            }
            _ => continue,
        }
    }

    Ok(())
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
