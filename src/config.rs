use near_lake_framework::LakeConfig;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AppSettings {
    pub contract_acc: String,
    pub database: DatabaseSettings,
    pub aws: AwsSettings,
}

#[derive(Deserialize)]
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
pub fn get_config() -> Result<AppSettings, config::ConfigError> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config"))?;
    settings.try_into()
}
