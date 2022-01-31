use near_indexer::near_primitives::types::AccountId;
use tokio::sync::OnceCell;

static MAIN_ACC: OnceCell<AccountId> = OnceCell::const_new();

pub async fn get_main_acc() -> &'static AccountId {
    MAIN_ACC
        .get_or_init(|| async { "test.near".parse().expect("Couldn't parse provided name") })
        .await
}

// if !outcome.receipt.receiver_id.is_sub_account_of(&main_acc) {
//     return Ok(());
// }
