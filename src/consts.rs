use crate::config::AppConfig;
use tokio::sync::OnceCell;

pub const EVENT_PREFIX: &str = "EVENT_JSON:";

pub static CONFIG: OnceCell<AppConfig> = OnceCell::const_new();
