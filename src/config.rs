use std::path::PathBuf;

use near_indexer::{AwaitForNodeSyncedEnum, IndexerConfig, InitConfigArgs, SyncModeEnum};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RunSettings {
    pub main_account: String,
    pub database: DatabaseSettings,
    #[serde(with = "IndexerConfigDef")]
    pub indexer: IndexerConfig,
}

#[derive(Deserialize, Debug)]
pub struct InitSettings {
    #[serde(with = "InitConfigArgsDef")]
    pub indexer: InitConfigArgs,
    pub home_dir: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(remote = "InitConfigArgs")]
struct InitConfigArgsDef {
    pub chain_id: Option<String>,
    pub account_id: Option<String>,
    pub test_seed: Option<String>,
    pub num_shards: u64,
    pub fast: bool,
    pub genesis: Option<String>,
    pub download_genesis: bool,
    pub download_genesis_url: Option<String>,
    pub download_config: bool,
    pub download_config_url: Option<String>,
    pub boot_nodes: Option<String>,
    pub max_gas_burnt_view: Option<near_indexer::near_primitives::types::Gas>,
}

#[derive(Deserialize)]
#[serde(remote = "IndexerConfig")]
pub struct IndexerConfigDef {
    pub home_dir: PathBuf,
    #[serde(with = "SyncModeEnumDef")]
    pub sync_mode: SyncModeEnum,
    #[serde(with = "AwaitForNodeSyncedEnumDef")]
    pub await_for_node_synced: AwaitForNodeSyncedEnum,
}

#[derive(Deserialize)]
#[serde(remote = "SyncModeEnum", rename_all = "snake_case")]
pub enum SyncModeEnumDef {
    LatestSynced,
    FromInterruption,
    BlockHeight(u64),
}

#[derive(Deserialize)]
#[serde(remote = "AwaitForNodeSyncedEnum", rename_all = "snake_case")]
pub enum AwaitForNodeSyncedEnumDef {
    WaitForFullSync,
    StreamWhileSyncing,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }
}

#[tracing::instrument(
    name = "Loading configuration from file `config.yaml`",
    fields(id = %Uuid::new_v4())
)]
pub fn get_config<'de, T>() -> Result<T, config::ConfigError>
where
    T: Deserialize<'de>,
{
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config"))?;
    settings.try_into()
}
