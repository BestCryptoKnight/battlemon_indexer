use std::time::Duration;

use actix_web::web;
use chrono::Utc;
use futures::try_join;
use futures::StreamExt;
use near_indexer::near_primitives::types::ShardId;
use near_indexer::{IndexerExecutionOutcomeWithReceipt, IndexerShard, StreamerMessage};
use sqlx::PgPool;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use consts::get_main_acc;

use crate::models::Event;

pub type GenericError = Box<dyn std::error::Error + Sync + Send>;
pub type Result<T> = std::result::Result<T, GenericError>;

pub mod config;
pub mod consts;
pub mod models;
pub mod startup;
pub mod telemetry;

pub async fn listen_blocks(stream: Receiver<StreamerMessage>, db: web::Data<PgPool>) -> Result<()> {
    let mut handle_messages = ReceiverStream::new(stream)
        .map(|streamer_message| {
            tracing::info!("Block height: {:?}", streamer_message.block.header.height);
            handle_message(streamer_message, db.clone())
        })
        .buffer_unordered(1);

    while let Some(_handled_message) = handle_messages.next().await {}

    // Graceful shutdown
    tracing::info!("Indexer will be shutdown gracefully in 7 seconds...");
    drop(handle_messages);
    time::sleep(Duration::from_secs(7)).await;

    Ok(())
}

#[tracing::instrument(name = "Handling streamer message", skip(streamer_message, db))]
async fn handle_message(streamer_message: StreamerMessage, db: web::Data<PgPool>) -> Result<()> {
    let nft_events = async {
        for shard in &streamer_message.shards {
            collect_and_store_nft_events(
                shard,
                &streamer_message.block.header.timestamp,
                db.clone(),
            )
            .await?;
        }

        Ok::<(), GenericError>(())
    };

    try_join!(nft_events)?;

    Ok(())
}

#[tracing::instrument(name = "Collecting nft events and store it in database", skip(db))]
async fn collect_and_store_nft_events(
    shard: &IndexerShard,
    block_timestamp: &u64,
    db: web::Data<PgPool>,
) -> Result<()> {
    let mut index_in_shard: i32 = 0;
    // let main_acc = get_main_acc().await;
    for outcome in &shard.receipt_execution_outcomes {
        // if !outcome.receipt.receiver_id.is_sub_account_of(main_acc) {
        //     continue;
        // }

        let nft_events = collect_nft_events(
            outcome,
            block_timestamp,
            &shard.shard_id,
            &mut index_in_shard,
        );
        if !nft_events.is_empty() {
            insert_nft_events(nft_events, &db).await?;
        }
    }

    Ok(())
}

#[tracing::instrument(name = "Saving new event to the database", skip(events, db))]
pub async fn insert_nft_events(events: Vec<Event>, db: &PgPool) -> Result<()> {
    let mut tx = db.begin().await?;
    for event in events {
        let query = match event {
            Event::Sale(sale) => {
                sqlx::query!(
                    r#"
                    INSERT INTO sales (id, prev_owner, curr_owner, token_id, price, date)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    Uuid::new_v4(),
                    sale.prev_owner,
                    sale.curr_owner,
                    sale.token_id,
                    sale.price,
                    Utc::now()
                )
            }
        };

        query.execute(&mut tx).await.map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    }
    tx.commit().await?;

    Ok(())
}

fn collect_nft_events(
    outcome: &IndexerExecutionOutcomeWithReceipt,
    block_timestamp: &u64,
    shard_id: &ShardId,
    index_in_shard: &mut i32,
) -> Vec<models::Event> {
    let prefix = "EVENT_JSON:";

    outcome
        .execution_outcome
        .outcome
        .logs
        .iter()
        .filter_map(|log| log.trim().strip_prefix(prefix))
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
