use actix_web::web;
use tokio::sync::mpsc;

use crate::{handle_message, StreamerMessage};

#[tracing::instrument(name = "Run indexer", skip(stream, client))]
pub async fn run_indexer(
    mut stream: mpsc::Receiver<StreamerMessage>,
    client: reqwest::Client,
) -> anyhow::Result<()> {
    let client = web::Data::new(client);
    while let Some(stream_message) = stream.recv().await {
        handle_message(stream_message, client.clone()).await?
    }

    Ok::<_, anyhow::Error>(())
}
