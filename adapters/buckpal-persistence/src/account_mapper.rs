use crate::account_entity::AccountEntity;
use crate::activity_entity::ActivityEntity;
use buckpal_application::domain::account::{Account, AccountId};
use buckpal_application::domain::activity::{Activity, ActivityId};
use buckpal_application::domain::activity_window::ActivityWindow;
use rusty_money::{money, Money};

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct AccountMapper {}

// Note: we don't use & here in some places because we don't need it

impl AccountMapper {
    pub fn map_to_domain_entity(
        &self,
        account: AccountEntity,
        activities: Vec<ActivityEntity>,
        withdrawal_blance: i64,
        deposit_balance: i64,
    ) -> Account {
        let baseline_balance = money!(deposit_balance, "AUD") - money!(withdrawal_blance, "AUD");

        Account::new_with_id(
            AccountId(account.id),
            baseline_balance,
            self.map_to_activity_window(activities),
        )
    }

    pub fn map_to_activity(&self, activity: &ActivityEntity) -> Activity {
        Activity::new_with_id(
            activity.id.map(ActivityId),
            AccountId(activity.owner_account_id),
            AccountId(activity.source_account_id),
            AccountId(activity.target_account_id),
            activity.timestamp,
            money!(activity.amount, "AUD"),
        )
    }

    pub fn map_to_activity_window(&self, activities: Vec<ActivityEntity>) -> ActivityWindow {
        let mapped_activities: Vec<Activity> = activities
            .iter()
            .map(|activity: &ActivityEntity| self.map_to_activity(activity))
            .collect();

        ActivityWindow::new(mapped_activities)
    }

    pub fn map_to_entity(&self, activity: Activity) -> ActivityEntity {
        use rust_decimal::prelude::*;

        ActivityEntity::new(
            activity.id.map(|id| id.0),
            activity.timestamp,
            activity.owner_account_id.0,
            activity.source_account_id.0,
            activity.target_account_id.0,
            // here we want to explode, no way to recover
            activity.money.amount().to_i64().unwrap(),
        )
    }
}
