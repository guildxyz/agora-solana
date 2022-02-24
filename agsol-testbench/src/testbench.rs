use crate::{TestbenchError, TestbenchProgram};
use borsh::BorshDeserialize;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::clock::UnixTimestamp;
use solana_program::hash::Hash;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program_test::{BanksClient, ProgramTest, ProgramTestContext};
use solana_sdk::account::Account;
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

pub type TestbenchResult<T> = Result<T, TestbenchError>;
pub type TestbenchTransactionResult<T> = TestbenchResult<Result<T, TransactionError>>;

impl Testbench {
    /// Create new `Testbench` by loading [`TestbenchProgram`]s into a
    /// [`ProgramTest`] context.
    pub async fn new(programs: &[TestbenchProgram<'_>]) -> TestbenchResult<Self> {
        let mut program_test = ProgramTest::default();

        for program in programs {
            program_test.add_program(program.name, program.id, program.process_instruction)
        }

        let mut context = program_test.start_with_context().await;
        let rent = context
            .banks_client
            .get_rent()
            .await
            .map_err(|_| TestbenchError::RentError)?;

        Ok(Self { context, rent })
    }

    pub fn client(&mut self) -> &mut BanksClient {
        &mut self.context.banks_client
    }

    pub fn payer(&self) -> &Keypair {
        &self.context.payer
    }

    pub fn clone_payer(&self) -> Keypair {
        // unwrap is fine here because it is guaranteed to be a correct keypair
        Keypair::from_bytes(self.context.payer.to_bytes().as_ref()).unwrap()
    }

    pub async fn get_account(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> TestbenchResult<Option<Account>> {
        self.client()
            .get_account(*account_pubkey)
            .await
            .map_err(|_| TestbenchError::SolanaInternalError)
    }

    // TODO make this nicer?
    pub async fn block_time(&mut self) -> TestbenchResult<UnixTimestamp> {
        let clock_sysvar = self
            .get_account(&solana_program::sysvar::clock::id())
            .await?
            .ok_or(TestbenchError::AccountNotFound)?;
        Ok(
            solana_sdk::account::from_account::<solana_program::clock::Clock, _>(&clock_sysvar)
                .ok_or(TestbenchError::CouldNotDeserialize)?
                .unix_timestamp,
        )
    }

    pub fn warp_to_slot(&mut self, slot: u64) -> TestbenchResult<()> {
        self.context
            .warp_to_slot(slot)
            .map_err(|_| TestbenchError::WarpingError)
    }

    pub async fn get_current_slot(&mut self) -> TestbenchResult<u64> {
        Ok(self
            .context
            .banks_client
            .get_root_slot()
            .await
            .map_err(|_| TestbenchError::WarpingError)?)
    }

    pub async fn warp_n_slots(&mut self, n: u64) -> TestbenchResult<()> {
        let current_slot = self.get_current_slot().await?;
        self.warp_to_slot(current_slot + n)
    }

    pub async fn warp_n_seconds(&mut self, n: i64) -> TestbenchResult<()> {
        let clock_start = self.block_time().await?;
        let mut clock_curr = self.block_time().await?;
        while clock_curr < clock_start + n {
            self.warp_n_slots(n as u64).await?;
            clock_curr = self.block_time().await?;
        }
        Ok(())
    }

    pub async fn warp_to_finalize(&mut self) -> TestbenchResult<()> {
        self.warp_n_slots(2).await
    }

    pub async fn get_latest_blockhash(&mut self) -> TestbenchResult<Hash> {
        self.context
            .banks_client
            .get_latest_blockhash()
            .await
            .map_err(|_| TestbenchError::BlockhashError)
    }

    pub async fn process_transaction(
        &mut self,
        instructions: &[Instruction],
        payer: &Keypair,
        signers: Option<&[&Keypair]>,
    ) -> TestbenchTransactionResult<i64> {
        let latest_blockhash = self.get_latest_blockhash().await?;

        let mut transaction = Transaction::new_with_payer(instructions, Some(&payer.pubkey()));
        let mut all_signers = vec![payer];
        if let Some(signers) = signers {
            all_signers.extend_from_slice(signers);
        }

        transaction.sign(&all_signers, latest_blockhash);

        // TransportError has an unwrap method that turns it into a TransactionError

        let payer_balance_before = self.get_account_lamports(&payer.pubkey()).await?;
        let transaction_result = self
            .context
            .banks_client
            .process_transaction(transaction)
            .await
            .map_err(|e| e.unwrap());
        self.warp_to_finalize().await?;
        let payer_balance_after = self.get_account_lamports(&payer.pubkey()).await?;

        Ok(transaction_result.map(|_| payer_balance_after as i64 - payer_balance_before as i64))
    }

    pub async fn create_mint(
        &mut self,
        decimals: u8,
        mint_authority: &Pubkey,
    ) -> TestbenchTransactionResult<Pubkey> {
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
            // unwrap is fine here because initialize_mint only throws error if the token program id is incorrect
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
            .map(|transaction_result| transaction_result.map(|_| mint_keypair.pubkey()))
    }

    pub async fn create_token_holding_account(
        &mut self,
        owner: &Keypair,
        mint: &Pubkey,
    ) -> TestbenchTransactionResult<Pubkey> {
        let account_keypair = Keypair::new();
        let rent = self.rent.minimum_balance(TokenAccount::LEN);
        let instructions = [
            system_instruction::create_account(
                &owner.pubkey(),
                &account_keypair.pubkey(),
                rent,
                TokenAccount::LEN as u64,
                &spl_token::id(),
            ),
            // unwrap is fine here because initialize_account only throws error if the token program id is incorrect
            token_instruction::initialize_account(
                &spl_token::id(),
                &account_keypair.pubkey(),
                mint,
                &owner.pubkey(),
            )
            .unwrap(),
        ];

        self.process_transaction(&instructions, owner, Some(&[&account_keypair]))
            .await
            .map(|transaction_result| transaction_result.map(|_| account_keypair.pubkey()))
    }

    pub async fn mint_to_account(
        &mut self,
        mint: &Pubkey,
        mint_authority: &Keypair,
        account: &Pubkey,
        amount: u64,
    ) -> TestbenchTransactionResult<()> {
        // unwrap is fine here because mint_to only throws error if the token program id is incorrect
        let instruction = token_instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &mint_authority.pubkey(),
            &[&mint_authority.pubkey()],
            amount,
        )
        .unwrap();

        self.process_transaction(&[instruction], mint_authority, None)
            .await
            .map(|transaction_result| transaction_result.map(|_| ()))
    }

    pub async fn token_balance(&mut self, token_account: &Pubkey) -> TestbenchResult<u64> {
        let token_data = self
            .get_and_deserialize_packed_account_data::<TokenAccount>(token_account)
            .await?;
        Ok(token_data.amount)
    }

    pub async fn total_supply(&mut self, mint_account: &Pubkey) -> TestbenchResult<u64> {
        let mint_data = self
            .get_and_deserialize_packed_account_data::<Mint>(mint_account)
            .await?;
        Ok(mint_data.supply)
    }

    pub async fn get_account_lamports(&mut self, account_pubkey: &Pubkey) -> TestbenchResult<u64> {
        Ok(self
            .get_account(account_pubkey)
            .await?
            .ok_or(TestbenchError::AccountNotFound)?
            .lamports)
    }

    pub async fn get_account_data(&mut self, account_pubkey: &Pubkey) -> TestbenchResult<Vec<u8>> {
        Ok(self
            .get_account(account_pubkey)
            .await?
            .ok_or(TestbenchError::AccountNotFound)?
            .data)
    }

    pub async fn get_and_deserialize_account_data<T: BorshDeserialize>(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> TestbenchResult<T> {
        let account_data = self.get_account_data(account_pubkey).await?;
        try_from_slice_unchecked(account_data.as_slice())
            .map_err(|_| TestbenchError::CouldNotDeserialize)
    }

    pub async fn get_and_deserialize_packed_account_data<T: Pack>(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> TestbenchResult<T> {
        let account_data = self.get_account_data(account_pubkey).await?;
        T::unpack_from_slice(&account_data).map_err(|_| TestbenchError::CouldNotDeserialize)
    }

    pub async fn get_token_account(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> TestbenchResult<TokenAccount> {
        self.get_and_deserialize_packed_account_data::<TokenAccount>(account_pubkey)
            .await
    }

    pub async fn get_mint_account(&mut self, account_pubkey: &Pubkey) -> TestbenchResult<Mint> {
        self.get_and_deserialize_packed_account_data::<Mint>(account_pubkey)
            .await
    }
}
