#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TestbenchError {
    #[error("Could not fetch rent")]
    RentError,
    #[error("Warping error")]
    WarpingError,
    #[error("Account not found")]
    AccountNotFound,
    #[error("Deserialize error")]
    CouldNotDeserialize,
    #[error("Could not fetch latest blockhash")]
    BlockhashError,
    #[error("Solana internal error")]
    SolanaInternalError,
}
