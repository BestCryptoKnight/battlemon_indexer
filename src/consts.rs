use crate::config::AppSettings;
use near_lake_framework::near_indexer_primitives::types::AccountId;
use std::str::FromStr;
use tokio::sync::OnceCell;

static CONFIG: OnceCell<AppSettings> = OnceCell::const_new();
pub async fn get_config() -> &'static AppSettings {
    CONFIG
        .get_or_init(|| async { crate::config::load_config().expect("Couldn't run indexer") })
        .await
}
