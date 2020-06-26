use rusty_money::{money, Money};

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct MoneyTransferProperties {}

impl MoneyTransferProperties {
    pub fn maximum_transfer_threshold(&self) -> Money {
        money!(1_000_000, "AUD")
    }
}
