#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TestbenchError {
    #[error(transparent)]
    TransactionError(#[from] solana_sdk::transaction::TransactionError),
    #[error("Warping error")]
    WarpingError,
    #[error("Account not found")]
    AccountNotFound,
    #[error("Deserialize error")]
    CouldNotDeserialize,
    #[error("Could not fetch latest blockhash")]
    BlockhashError,
}
