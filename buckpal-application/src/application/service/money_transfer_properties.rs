use rusty_money::{money, Money};

pub struct MoneyTransferProperties {}

impl MoneyTransferProperties {
    pub fn maximum_transfer_threshold(&self) -> Money {
        money!(1_000_000, "AUD")
    }
}
