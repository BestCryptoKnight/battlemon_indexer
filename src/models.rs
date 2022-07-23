use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum ContractEventEnum {
//     MarketSale(MarketSale),
//     NftEvent(NftEvent),
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardKind {
    #[serde(rename = "nep171")]
    Nep171,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionKind {
    #[serde(rename = "1.0.0")]
    V1_0_0,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NftEventLogKind {
    NftMintLog {
        owner_id: String,
        token_ids: Vec<String>,
        memo: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftEvent {
    pub standard: StandardKind,
    pub version: VersionKind,
    pub event: NftEventKind,
    pub data: Vec<NftEventLogKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NftEventKind {
    NftMint,
    NftBurn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSale {
    pub prev_owner: String,
    pub curr_owner: String,
    pub token_id: String,
    pub price: String,
}

#[derive(Deserialize)]
pub struct IpfsHash {
    pub hash: String,
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
                standard: StandardKind::Nep171,
                version: VersionKind::V1_0_0,
                event: NftEventKind::NftMint,
                data,
            }) => {
                let nft_event_log = data.get(0).expect("must be at least one event");
                assert!(matches!(nft_event_log, NftEventLogKind::NftMintLog { .. }))
            }
            _ => panic!("deserialized struct is wrong."),
        }
    }
}
