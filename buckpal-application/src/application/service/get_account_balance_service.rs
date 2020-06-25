use crate::application::port::incoming::get_account_balance_query::GetAccountBalanceQuery;
use crate::application::port::outgoing::load_account_port::LoadAccountPort;
use crate::domain::account::AccountId;
use rusty_money::Money;

pub struct GetAccountBalanceService<'a> {
    load_account_port: Box<dyn LoadAccountPort + 'a>,
}

impl<'a> GetAccountBalanceQuery for GetAccountBalanceService<'a> {
    fn get_account_balance(&self, account_id: &AccountId) -> Money {
        use chrono::Utc;

        self.load_account_port
            .load_account(account_id, &Utc::now())
            .calculate_balance()
    }
}
