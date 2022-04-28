use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContractEventEnum {
    MarketSale(MarketSale),
    NftEvent(NftEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardEnum {
    #[serde(rename = "nep171")]
    Nep171,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionEnum {
    #[serde(rename = "1.0.0")]
    V1_0_0,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NftEventLogEnum {
    NftMintLog {
        owner_id: String,
        token_ids: Vec<String>,
        memo: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftEvent {
    pub standard: StandardEnum,
    pub version: VersionEnum,
    pub event: NftEventEnum,
    pub data: Vec<NftEventLogEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NftEventEnum {
    NftMint,
    NftBurn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSale {
    pub prev_owner: String,
    pub curr_owner: String,
    pub token_id: String,
    pub price: Decimal,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nft_event_mint_deserialization() {
        let json = json!({
            "standard":"nep171",
            "version":"1.0.0",
            "event":"nft_mint",
            "data":[
                {
                    "owner_id":"battlemon.testnet",
                    "token_ids":["2"],
                    "memo": null,
                }
            ]
        })
        .to_string();
        let nft_event = serde_json::from_str(&json).expect("Couldn't deserialize json");

        match nft_event {
            ContractEventEnum::NftEvent(NftEvent {
                standard: StandardEnum::Nep171,
                version: VersionEnum::V1_0_0,
                event: NftEventEnum::NftMint,
                data,
            }) => {
                let nft_event_log = data.get(0).expect("must be at least one event");
                assert!(matches!(nft_event_log, NftEventLogEnum::NftMintLog { .. }))
            }
            _ => panic!("deserialized struct is wrong."),
        }
    }
}
