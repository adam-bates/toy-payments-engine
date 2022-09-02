pub mod events;
pub mod ids;
pub mod input;
pub mod models;
pub mod services;

mod account_report;
mod money;
mod result;

pub use account_report::AccountReport;
pub use money::Money;
pub use result::Result;

/// Convenience function to build a transaction service
pub fn build_transaction_service() -> services::TransactionService {
    let account_service = services::AccountService::new();
    let transaction_service = services::TransactionService::new(account_service);

    return transaction_service;
}
