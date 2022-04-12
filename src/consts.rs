use std::str::FromStr;
use near_lake_framework::near_indexer_primitives::types::AccountId;
use tokio::sync::OnceCell;

static MAIN_ACC: OnceCell<AccountId> = OnceCell::const_new();

pub async fn get_contract_acc() -> &'static AccountId {
    MAIN_ACC
        .get_or_init(|| async {
            let config = crate::config::get_config::<crate::config::RunSettings>()
               .expect("Couldn't run indexer");
            AccountId::from_str(&config.contract_acc).expect("Account from config isn't valid")
        })
        .await
}