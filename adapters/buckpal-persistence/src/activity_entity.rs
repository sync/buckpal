use chrono::{DateTime, Utc};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ActivityEntity {
    pub id: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub owner_account_id: i32,
    pub source_account_id: i32,
    pub target_account_id: i32,
    pub amount: i64,
}

impl ActivityEntity {
    pub fn new(
        id: Option<i32>,
        timestamp: DateTime<Utc>,
        owner_account_id: i32,
        source_account_id: i32,
        target_account_id: i32,
        amount: i64,
    ) -> Self {
        Self {
            id,
            timestamp,
            owner_account_id,
            source_account_id,
            target_account_id,
            amount,
        }
    }
}
