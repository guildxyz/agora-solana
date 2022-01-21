use super::account::{Account, EncodedAccount};
use super::rpc_config::*;
use super::rpc_request::RpcRequest;
use super::rpc_response::{RpcResponse, RpcResultWithContext};

use borsh::BorshDeserialize;
use reqwest::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use serde_json::json;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::{Signature, Transaction};

/// Specifies which Solana cluster will be queried by the client.
pub enum Net {
    Localhost,
    Testnet,
    Devnet,
    Mainnet,
}

impl Net {
    pub fn to_url(&self) -> &str {
        match self {
            Self::Localhost => "http://localhost:8899",
            Self::Testnet => "https://api.testnet.solana.com",
            Self::Devnet => "https://api.devnet.solana.com",
            Self::Mainnet => "https://api.mainnet-beta.solana.com",
        }
    }
}

pub type ClientResult<T> = Result<T, anyhow::Error>;

/// An async client to make rpc requests to the Solana blockchain.
pub struct RpcClient {
    client: reqwest::Client,
    config: RpcConfig,
    net: Net,
    request_id: u64,
}

impl RpcClient {
    pub fn new_with_config(net: Net, config: RpcConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            net,
            request_id: 0,
        }
    }

    pub fn new(net: Net) -> Self {
        let config = RpcConfig {
            encoding: Some(Encoding::JsonParsed),
        };
        Self::new_with_config(net, config)
    }

    async fn send<T: DeserializeOwned, R: Into<reqwest::Body>>(
        &mut self,
        request: R,
    ) -> reqwest::Result<T> {
        self.request_id = self.request_id.wrapping_add(1);
        let response = self
            .client
            .post(self.net.to_url())
            .header(CONTENT_TYPE, "application/json")
            .body(request)
            .send()
            .await?;

        response.json::<T>().await
    }

    /// Returns the decoded contents of a Solana account.
    pub async fn get_account(&mut self, account_pubkey: &Pubkey) -> ClientResult<Account> {
        let request = RpcRequest::GetAccountInfo
            .build_request_json(
                self.request_id,
                json!([json!(account_pubkey.to_string()), json!(self.config)]),
            )
            .to_string();
        let response: RpcResponse<RpcResultWithContext<EncodedAccount>> =
            self.send(request).await?;
        response.result.value.decode()
    }

    /// Returns the raw bytes in an account's data field.
    pub async fn get_account_data(&mut self, account_pubkey: &Pubkey) -> ClientResult<Vec<u8>> {
        let account = self.get_account(account_pubkey).await?;
        Ok(account.data)
    }

    /// Attempts to deserialize the contents of an account's data field into a
    /// given type.
    pub async fn get_and_deserialize_account_data<T: BorshDeserialize>(
        &mut self,
        account_pubkey: &Pubkey,
    ) -> ClientResult<T> {
        let account_data = self.get_account_data(account_pubkey).await?;
        try_from_slice_unchecked(&account_data).map_err(|e| anyhow::anyhow!(e))
    }

    /// Returns the owner of the account.
    pub async fn get_owner(&mut self, account_pubkey: &Pubkey) -> ClientResult<Pubkey> {
        let account = self.get_account(account_pubkey).await?;
        Ok(account.owner)
    }

    /// Returns the balance (in Lamports) of the account.
    pub async fn get_lamports(&mut self, account_pubkey: &Pubkey) -> ClientResult<u64> {
        let account = self.get_account(account_pubkey).await?;
        Ok(account.lamports)
    }

    /// Returns the minimum balance (in Lamports) required for an account to be rent exempt.
    pub async fn get_minimum_balance_for_rent_exemption(
        &mut self,
        data_len: usize,
    ) -> ClientResult<u64> {
        let request = RpcRequest::GetMinimumBalanceForRentExemption
            .build_request_json(self.request_id, json!([data_len]))
            .to_string();

        let response: RpcResponse<u64> = self.send(request)
            .await?;
        Ok(response.result)
    }

    pub async fn request_airdrop(
        &mut self,
        pubkey: &Pubkey,
        lamports: u64,
    ) -> ClientResult<Signature> {
        let config = RpcRequestAirdropConfig {
            recent_blockhash: None,
            commitment: Some(CommitmentLevel::Confirmed),
        };
        let request = RpcRequest::RequestAirdrop
            .build_request_json(
                self.request_id,
                json!([pubkey.to_string(), lamports, config]),
            )
            .to_string();

        let response: RpcResponse<Signature> = self.send(request)
            .await?;

        Ok(response.result)
    }

    pub async fn send_transaction(&mut self, transaction: &Transaction) -> ClientResult<Signature> {
        todo!();
    }
}
