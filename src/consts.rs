use std::str::FromStr;

use near_indexer::near_primitives::types::AccountId;
use tokio::sync::OnceCell;

static MAIN_ACC: OnceCell<AccountId> = OnceCell::const_new();

pub async fn get_main_acc() -> &'static AccountId {
    MAIN_ACC
        .get_or_init(|| async {
            let config = crate::config::get_config::<crate::config::RunSettings>()
                .expect("Couldn't run indexer");
            AccountId::from_str(&config.main_account).expect("Account from config isn't valid")
        })
        .await
}
