use crate::{Testbench, TestbenchTransactionResult};
use solana_program::system_instruction;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signer::Signer;

/// A mock user with a signer [`Keypair`].
pub struct TestUser {
    pub keypair: Keypair,
}

impl TestUser {
    /// Creates a new user and sends an airdrop to its address.
    pub async fn new(testbench: &mut Testbench) -> TestbenchTransactionResult<Self> {
        let keypair = Keypair::new();

        // send lamports to user
        let instruction = system_instruction::transfer(
            &testbench.payer().pubkey(),
            &keypair.pubkey(),
            150_000_000,
        );

        let payer = testbench.clone_payer();

        testbench
            .process_transaction(&[instruction], &payer, None)
            .await
            .map(|transaction_result| transaction_result.map(|_| Self { keypair } ))
    }
}
