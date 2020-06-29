#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AccountEntity {
    pub id: i32,
}

impl AccountEntity {
    #[allow(dead_code)]
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}
