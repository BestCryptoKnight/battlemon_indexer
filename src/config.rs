use crate::consts::CONFIG;
use anyhow::Context;
use aws_sdk_s3::Region;
use battlemon_near_json_rpc_client_wrapper::AccountId;
use battlemon_near_json_rpc_client_wrapper::JsonRpcWrapper;
use near_crypto::{InMemorySigner, SecretKey};
use near_lake_framework::{LakeConfig, LakeConfigBuilder};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use std::str::FromStr;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct AppSettings {
    pub contracts: ContractSettings,
    pub database: DatabaseSettings,
    pub near_lake: NearLakeSettings,
}

#[derive(serde::Deserialize)]
pub struct ContractSettings {
    top_contract_id: AccountId,
    nft_contract_id: AccountId,
    market_contract_id: AccountId,
}

impl ContractSettings {
    pub fn ids(&self) -> (&AccountId, &AccountId, &AccountId) {
        (
            &self.top_contract_id,
            &self.nft_contract_id,
            &self.market_contract_id,
        )
    }

    pub fn nft_id(&self) -> &AccountId {
        &self.nft_contract_id
    }

    pub fn market_id(&self) -> &AccountId {
        &self.market_contract_id
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct NearCredentialsSettings {
    pub account_id: AccountId,
    pub private_key: Secret<String>,
}

#[derive(serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum NearNetworkKind {
    Mainnet,
    Testnet,
}

impl NearNetworkKind {
    pub fn rpc_url(&self) -> &'static str {
        match self {
            Self::Mainnet => battlemon_near_json_rpc_client_wrapper::NEAR_MAINNET_ARCHIVAL_RPC_URL,
            Self::Testnet => battlemon_near_json_rpc_client_wrapper::NEAR_TESTNET_ARCHIVAL_RPC_URL,
        }
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct NearLakeSettings {
    pub network: NearNetworkKind,
    pub start_block_height: u64,
    pub start_from_last_block: bool,
    aws_access_key_id: Secret<String>,
    aws_secret_access_key: Secret<String>,
    near_credentials: NearCredentialsSettings,
}

impl NearLakeSettings {
    pub async fn near_lake_config(&self) -> anyhow::Result<LakeConfig> {
        let aws_creds = near_lake_framework::Credentials::new(
            self.aws_access_key_id.expose_secret(),
            self.aws_secret_access_key.expose_secret(),
            None,
            None,
            "custom_credentials",
        );
        let s3_config = aws_sdk_s3::Config::builder()
            .credentials_provider(aws_creds)
            .region(Region::new("eu-central-1"))
            .build();
        let ret = LakeConfigBuilder::default().s3_config(s3_config);
        let block_height = if self.start_from_last_block {
            let secret_key =
                SecretKey::from_str(self.near_credentials.private_key.expose_secret()).unwrap();
            let signer = InMemorySigner::from_secret_key(
                self.near_credentials.account_id.clone(),
                secret_key,
            );
            let rpc_client = JsonRpcWrapper::connect(self.network.rpc_url(), signer);
            rpc_client.final_block_height().await?
        } else {
            self.start_block_height
        };

        let ret = match self.network {
            NearNetworkKind::Mainnet => ret.mainnet(),
            NearNetworkKind::Testnet => ret.testnet(),
        }
        .start_block_height(block_height)
        .build()?;

        Ok(ret)
    }
}

#[derive(Deserialize, Clone)]
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
pub fn load_config() -> anyhow::Result<AppSettings> {
    let config_path = std::env::current_dir()
        .context("Failed to determine current directory")?
        .join("configs");
    // TODO:
    //  - add testnet and mainnet configs
    //  - add env var to override config file
    let settings = config::Config::builder()
        .add_source(config::File::from(config_path.join("local_config.toml")))
        .build()?;

    settings
        .try_deserialize()
        .context("Failed to deserialize config files into `AppSettings`")
}

pub async fn get_config() -> &'static AppSettings {
    CONFIG
        .get_or_init(|| async { load_config().expect("Couldn't load config for indexer") })
        .await
}
