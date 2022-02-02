use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Events {
    Sale(Sale),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub prev: String,
    pub curr: String,
    pub token_id: String,
    pub price: String,
}
