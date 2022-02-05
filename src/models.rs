use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Sale(Sale),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub prev_owner: String,
    pub curr_owner: String,
    pub token_id: String,
    pub price: String,
}
