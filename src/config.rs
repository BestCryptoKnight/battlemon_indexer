use near_crypto::{InMemorySigner, PublicKey, SecretKey};
use near_lake_framework::near_indexer_primitives::types::AccountId;
use near_lake_framework::LakeConfig;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AppSettings {
    pub contracts: ContractSettings,
    pub database: DatabaseSettings,
    pub aws: AwsSettings,
    pub near_credentials: NearCredsSettings,
}

#[derive(Deserialize)]
pub struct ContractSettings {
    pub top_contract_id: AccountId,
    pub nft_contract_id: AccountId,
    pub market_contract_id: AccountId,
}

#[derive(Deserialize, Clone)]
pub struct NearCredsSettings {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl From<NearCredsSettings> for InMemorySigner {
    fn from(near_creds: NearCredsSettings) -> Self {
        Self {
            account_id: near_creds.account_id,
            public_key: near_creds.public_key,
            secret_key: near_creds.private_key,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct AwsSettings {
    pub s3_endpoint: Option<String>,
    pub s3_bucket_name: String,
    pub s3_region_name: String,
    pub start_block_height: u64,
}

impl From<AwsSettings> for LakeConfig {
    fn from(aws: AwsSettings) -> Self {
        Self {
            s3_endpoint: aws.s3_endpoint,
            s3_bucket_name: aws.s3_bucket_name,
            s3_region_name: aws.s3_region_name,
            start_block_height: aws.start_block_height,
        }
    }
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
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.db_name)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
    }
}

#[tracing::instrument(
    name = "Loading configuration from file `config.yaml`",
    fields(id = %Uuid::new_v4())
)]
pub fn load_config() -> Result<AppSettings, config::ConfigError> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config"))?;
    settings.try_into()
}
