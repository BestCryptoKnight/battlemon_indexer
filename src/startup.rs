use tokio::sync::mpsc;

use crate::{handle_message, PgPool, StreamerMessage};

#[tracing::instrument(name = "Run indexer", skip(stream, pool_conn))]
pub async fn run_indexer(
    mut stream: mpsc::Receiver<StreamerMessage>,
    pool_conn: PgPool,
) -> crate::Result<()> {
    let pool_conn = actix_web::web::Data::new(pool_conn);

    while let Some(stream_message) = stream.recv().await {
        handle_message(stream_message, pool_conn.clone()).await?
    }

    Ok(())
}
