use chrono::{DateTime, Utc};
use rusty_money::Money;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AccountId(i32);

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ActivityId(i32);

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Activity {
    pub id: Option<ActivityId>,
    pub owner_account_id: AccountId,
    pub source_account_id: AccountId,
    pub target_account_id: AccountId,
    pub timestamp: DateTime<Utc>,
    pub money: Money,
}

impl Activity {
    fn new(
        owner_account_id: AccountId,
        source_account_id: AccountId,
        target_account_id: AccountId,
        timestamp: DateTime<Utc>,
        money: Money,
    ) -> Self {
        Self {
            id: None,
            owner_account_id,
            source_account_id,
            target_account_id,
            timestamp,
            money,
        }
    }
}
