pub mod events;
pub mod ids;
pub mod input;
pub mod models;
mod money;
mod result;
pub mod services;

pub use money::Money;
pub use result::Result;

pub fn build_transaction_service() -> services::TransactionService {
    let account_service = services::AccountService::new();
    let transaction_service = services::TransactionService::new(account_service);

    return transaction_service;
}
