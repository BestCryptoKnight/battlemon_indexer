use actix_web::web;
use near_jsonrpc_client::JsonRpcClient;
use tokio::sync::mpsc;

use crate::{handle_message, PgPool, StreamerMessage};

#[tracing::instrument(name = "Run indexer", skip(stream, pool_conn))]
pub async fn run_indexer(
    mut stream: mpsc::Receiver<StreamerMessage>,
    pool_conn: PgPool,
    rpc_client: JsonRpcClient,
) -> crate::Result<()> {
    let pool_conn = web::Data::new(pool_conn);
    let rpc_client = web::Data::new(rpc_client);
    while let Some(stream_message) = stream.recv().await {
        handle_message(stream_message, pool_conn.clone(), rpc_client.clone()).await?
    }

    Ok(())
}
