use crate::config::AppSettings;
use anyhow::Context;
use tokio::sync::OnceCell;

pub const EVENT_PREFIX: &str = "EVENT_JSON:";

pub static CONFIG: OnceCell<AppSettings> = OnceCell::const_new();
