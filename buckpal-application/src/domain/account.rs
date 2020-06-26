use crate::domain::activity_window::ActivityWindow;
use rusty_money::{money, Money};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AccountId(pub i32);

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Account {
    /// The unique ID of the account
    pub id: Option<AccountId>,
    /// The baseline balance of the account. This was the balance of the account before the first
    /// activity in the activityWindow.
    pub baseline_balance: Money,
    /// The window of latest activities on this account.
    pub activity_window: ActivityWindow,
}

#[cfg_attr(test, mocktopus::macros::mockable)]
impl Account {
    /// Creates an Account entity without an ID. Use to create a new entity that is not yet
    /// persisted.
    pub fn new_without_id(baseline_balance: Money, activity_window: ActivityWindow) -> Self {
        Self {
            id: None,
            baseline_balance,
            activity_window,
        }
    }

    /// Creates an Account entity with an ID. Use to reconstitute a persisted entity.
    pub fn new_with_id(
        account_id: AccountId,
        baseline_balance: Money,
        activity_window: ActivityWindow,
    ) -> Self {
        Self {
            id: Some(account_id),
            baseline_balance,
            activity_window,
        }
    }

    /// Calculates the total balance of the account by adding the activity values to the baseline
    /// balance.
    pub fn calculate_balance(&self) -> Money {
        let window_balance = self.id.clone().map_or_else(
            || money!(0, "AUD"),
            |id| self.activity_window.calculate_balance(&id),
        );
        self.baseline_balance.clone() + window_balance
    }

    /// Tries to withdraw a certain amount of money from this account.
    /// If successful, creates a new activity with a negative value.
    pub fn withdraw(&mut self, money: &Money, target_account_id: &AccountId) -> bool {
        if !self.may_withdraw(&money) {
            return false;
        }

        let id = match self.id.clone() {
            Some(id) => id,
            None => return false,
        };

        use crate::domain::activity::Activity;
        use chrono::Utc;

        let withdrawal = Activity::new(
            id.clone(),
            id,
            target_account_id.clone(),
            Utc::now(),
            money.clone(),
        );
        self.activity_window.add_activity(&withdrawal);
        true
    }

    fn may_withdraw(&self, money: &Money) -> bool {
        let balance = self.calculate_balance() - money.clone();
        balance.is_zero() || balance.is_positive()
    }

    /// Tries to deposit a certain amount of money to this account.
    /// If sucessful, creates a new activity with a positive value.
    /// return true if the deposit was successful, false if not.
    pub fn deposit(&mut self, money: &Money, source_account_id: &AccountId) -> bool {
        let id = match self.id.clone() {
            Some(id) => id,
            None => return false,
        };

        use crate::domain::activity::Activity;
        use chrono::Utc;

        let deposit = Activity::new(
            id.clone(),
            source_account_id.clone(),
            id,
            Utc::now(),
            money.clone(),
        );
        self.activity_window.add_activity(&deposit);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::account_test_data::AccountBuilder;
    use super::{AccountId, ActivityWindow};
    use crate::domain::activity::activity_test_data::ActivityBuilder;
    use rusty_money::{money, Money};

    #[test]
    fn calculates_balance() {
        let account_id = AccountId(1);
        let activity_window = ActivityWindow::new(vec![
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(999, "AUD"))
                .build(),
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(1, "AUD"))
                .build(),
        ]);
        let account = AccountBuilder::default_account()
            .with_account_id(&account_id)
            .with_baseline_balance(&money!(555, "AUD"))
            .with_activity_window(&activity_window)
            .build();

        let balance = account.calculate_balance();

        assert_eq!(balance, money!(1555, "AUD"));
    }

    #[test]
    fn withdrawal_succeeds() {
        let account_id = AccountId(1);
        let activity_window = ActivityWindow::new(vec![
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(999, "AUD"))
                .build(),
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(1, "AUD"))
                .build(),
        ]);
        let mut account = AccountBuilder::default_account()
            .with_account_id(&account_id.clone())
            .with_baseline_balance(&money!(555, "AUD"))
            .with_activity_window(&activity_window)
            .build();

        let success = account.withdraw(&money!(555, "AUD"), &AccountId(99));

        assert_eq!(success, true);
        assert_eq!(account.activity_window.activities.len(), 3);
        assert_eq!(account.calculate_balance(), money!(1000, "AUD"));
    }

    #[test]
    fn withdrawal_failure() {
        let account_id = AccountId(1);
        let activity_window = ActivityWindow::new(vec![
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(999, "AUD"))
                .build(),
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(1, "AUD"))
                .build(),
        ]);
        let mut account = AccountBuilder::default_account()
            .with_account_id(&account_id)
            .with_baseline_balance(&money!(555, "AUD"))
            .with_activity_window(&activity_window)
            .build();

        let success = account.withdraw(&money!(1556, "AUD"), &AccountId(99));

        assert_eq!(success, false);
        assert_eq!(account.activity_window.activities.len(), 2);
        assert_eq!(account.calculate_balance(), money!(1555, "AUD"));
    }

    #[test]
    fn deposit_succeeds() {
        let account_id = AccountId(1);
        let activity_window = ActivityWindow::new(vec![
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(999, "AUD"))
                .build(),
            ActivityBuilder::default_activity()
                .with_target_account(&account_id)
                .with_money(&money!(1, "AUD"))
                .build(),
        ]);
        let mut account = AccountBuilder::default_account()
            .with_account_id(&account_id.clone())
            .with_baseline_balance(&money!(555, "AUD"))
            .with_activity_window(&activity_window)
            .build();

        let success = account.deposit(&money!(445, "AUD"), &AccountId(99));

        assert_eq!(success, true);
        assert_eq!(account.activity_window.activities.len(), 3);
        assert_eq!(account.calculate_balance(), money!(2000, "AUD"));
    }
}

#[cfg(test)]
pub mod account_test_data {
    use super::{Account, AccountId};
    use crate::domain::activity::activity_test_data::ActivityBuilder;
    use crate::domain::activity_window::ActivityWindow;
    use rusty_money::{money, Money};

    pub struct AccountBuilder {
        account: Account,
    }

    impl AccountBuilder {
        pub fn default_account() -> Self {
            let activity_window = ActivityWindow::new(vec![
                ActivityBuilder::default_activity().build(),
                ActivityBuilder::default_activity().build(),
            ]);
            let account = Account::new_with_id(AccountId(42), money!(999, "AUD"), activity_window);

            Self { account }
        }

        pub fn with_account_id(&mut self, account_id: &AccountId) -> &mut Self {
            let mut account = self.account.clone();
            account.id = Some(account_id.clone());

            let mut new = self;
            new.account = account;
            new
        }

        pub fn with_baseline_balance(&mut self, baseline_balance: &Money) -> &mut Self {
            let mut account = self.account.clone();
            account.baseline_balance = baseline_balance.clone();

            let mut new = self;
            new.account = account;
            new
        }

        pub fn with_activity_window(&mut self, activity_window: &ActivityWindow) -> &mut Self {
            let mut account = self.account.clone();
            account.activity_window = activity_window.clone();

            let mut new = self;
            new.account = account;
            new
        }

        pub fn build(&self) -> Account {
            self.account.clone()
        }
    }
}
