use crate::domain::account::Account;

#[cfg_attr(test, mockall::automock)]
pub trait UpdateAccountStatePort {
    fn update_activities(&self, account: &Account);
}
