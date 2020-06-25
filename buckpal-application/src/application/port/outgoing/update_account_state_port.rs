use crate::domain::account::Account;

pub trait UpdateAccountStatePort {
    fn update_activities(account: Account);
}
