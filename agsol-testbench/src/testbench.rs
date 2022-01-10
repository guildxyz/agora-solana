use crate::TestbenchProgram;
use borsh::BorshDeserialize;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::clock::UnixTimestamp;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program_test::{BanksClient, ProgramTest, ProgramTestContext};
use solana_sdk::instruction::Instruction;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::{Transaction, TransactionError};

use spl_token::instruction as token_instruction;
use spl_token::state::{Account as TokenAccount, Mint};

/// Testbench wrapper around a [`ProgramTestContext`].
pub struct Testbench {
    pub context: ProgramTestContext,
    pub rent: Rent,
}

impl Testbench {
    /// Create new `Testbench` by loading [`TestbenchProgram`]s into a
    /// [`ProgramTest`] context.
    pub async fn new(programs: &[TestbenchProgram<'_>]) -> Self {
        let mut program_test = ProgramTest::default();

        for program in programs {
            program_test.add_program(program.name, program.id, program.process_instruction)
        }

        let mut context = program_test.start_with_context().await;
        let rent = context.banks_client.get_rent().await.unwrap();

        Self { context, rent }
    }

    pub fn client(&mut self) -> &mut BanksClient {
        &mut self.context.banks_client
    }

    pub fn payer(&self) -> &Keypair {
        &self.context.payer
    }

    pub fn clone_payer(&self) -> Keypair {
        Keypair::from_bytes(self.context.payer.to_bytes().as_ref()).unwrap()
    }

    // TODO make this nicer?
    pub async fn block_time(&mut self) -> UnixTimestamp {
        let clock_sysvar = self
            .client()
            .get_account(solana_program::sysvar::clock::id())
            .await
            .unwrap() // result
            .unwrap(); // option
        solana_sdk::account::from_account::<solana_program::clock::Clock, _>(&clock_sysvar)
            .unwrap()
            .unix_timestamp
    }

    pub fn warp_to_slot(&mut self, slot: u64) {
        self.context.warp_to_slot(slot).unwrap()
    }

    pub async fn get_current_slot(&mut self) -> u64 {
        self.context.banks_client.get_root_slot().await.unwrap()
    }

    pub async fn warp_n_slots(&mut self, n: u64) {
        let current_slot = self.get_current_slot().await;
        self.warp_to_slot(current_slot + n);
    }

    pub async fn warp_n_seconds(&mut self, n: i64) {
        let clock_start = self.block_time().await;
        let mut clock_curr = self.block_time().await;
        while clock_curr < clock_start + n {
            self.warp_n_slots(n as u64 * 5).await;
            clock_curr = self.block_time().await;
        }
    }

    pub async fn warp_to_finalize(&mut self) {
        self.warp_n_slots(2).await;
    }

    pub async fn process_transaction(
        &mut self,
        instructions: &[Instruction],
        payer: &Keypair,
        signers: Option<&[&Keypair]>,
    ) -> Result<(), TransactionError> {
        let latest_blockhash = self
            .context
            .banks_client
            .get_latest_blockhash()
            .await
            .unwrap();

        let mut transaction = Transaction::new_with_payer(instructions, Some(&payer.pubkey()));
        let mut all_signers = vec![payer];
        if let Some(signers) = signers {
            all_signers.extend_from_slice(signers);
        }

        transaction.sign(&all_signers, latest_blockhash);

        // TransportError has an unwrap method that turns it into a
        // TransactionError
        self.context
            .banks_client
            .process_transaction(transaction)
            .await
            .map_err(|e| e.unwrap())?;

        self.warp_to_finalize().await;

        Ok(())
    }

    pub async fn create_mint(&mut self, decimals: u8, mint_authority: &Pubkey) -> Pubkey {
        let mint_keypair = Keypair::new();
        let mint_rent = self.rent.minimum_balance(Mint::LEN);
        let instructions = [
            system_instruction::create_account(
                &self.context.payer.pubkey(),
                &mint_keypair.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            token_instruction::initialize_mint(
                &spl_token::id(),
                &mint_keypair.pubkey(),
                mint_authority,
                None,
                decimals,
            )
            .unwrap(),
        ];

        let payer = self.clone_payer();
        self.process_transaction(&instructions, &payer, Some(&[&mint_keypair]))
            .await
            .unwrap();

        mint_keypair.pubkey()
    }

    pub async fn create_token_holding_account(&mut self, owner: &Keypair, mint: &Pubkey) -> Pubkey {
        let account_keypair = Keypair::new();
        let mint_rent = self.rent.minimum_balance(TokenAccount::LEN);
        let instructions = [
            system_instruction::create_account(
                &owner.pubkey(),
                &account_keypair.pubkey(),
                mint_rent,
                TokenAccount::LEN as u64,
                &spl_token::id(),
            ),
            token_instruction::initialize_account(
                &spl_token::id(),
                &account_keypair.pubkey(),
                mint,
                &owner.pubkey(),
            )
            .unwrap(),
        ];

        let payer = self.clone_payer();
        self.process_transaction(&instructions, &payer, Some(&[owner, &account_keypair]))
            .await
            .unwrap();

        account_keypair.pubkey()
    }

    pub async fn mint_to_account(&mut self, mint: &Pubkey, account: &Pubkey, amount: u64) {
        let instruction = token_instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &self.payer().pubkey(), // mint authority
            &[&self.payer().pubkey()],
            amount,
        )
        .unwrap();

        let signer = self.clone_payer();
        self.process_transaction(&[instruction], &signer, Some(&[&signer]))
            .await
            .unwrap();
    }

    pub async fn token_balance(&mut self, token_account: &Pubkey) -> u64 {
        let data: TokenAccount = self
            .client()
            .get_packed_account_data(*token_account)
            .await
            .unwrap();

        data.amount
    }

    pub async fn total_supply(&mut self, mint_account: &Pubkey) -> u64 {
        let data: Mint = self
            .client()
            .get_packed_account_data(*mint_account)
            .await
            .unwrap();

        data.supply
    }

    pub async fn get_account_lamports(&mut self, account_pubkey: &Pubkey) -> u64 {
        self.client()
            .get_account(*account_pubkey)
            .await
            .unwrap()
            .unwrap()
            .lamports
    }

    pub async fn get_account_data(&mut self, account_pubkey: &Pubkey) -> Vec<u8> {
        self.client()
            .get_account(*account_pubkey)
            .await
            .unwrap()
            .unwrap()
            .data
    }

    pub async fn get_and_deserialize_account_data<T: BorshDeserialize>(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> T {
        let account_data = self.get_account_data(account_pubkey).await;
        try_from_slice_unchecked(account_data.as_slice()).unwrap()
    }

    pub async fn get_token_account(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> Result<TokenAccount, String> {
        self.client()
            .get_packed_account_data(*account_pubkey)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_mint_account(&mut self, account_pubkey: &Pubkey) -> Result<Mint, String> {
        self.client()
            .get_packed_account_data(*account_pubkey)
            .await
            .map_err(|e| e.to_string())
    }
}
