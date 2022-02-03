use super::account::{Account, EncodedAccount};
use super::rpc_config::*;
use super::rpc_request::RpcRequest;
use super::rpc_response::*;

use anyhow::bail;
use borsh::BorshDeserialize;
use log::debug;
use reqwest::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use serde_json::json;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::pubkey::Pubkey;
use solana_sdk::clock::{Slot, UnixTimestamp};
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;

use std::str::FromStr;

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

    /// Returns the balance (in lamports) of the account.
    pub async fn get_balance(&mut self, account_pubkey: &Pubkey) -> ClientResult<u64> {
        let request = RpcRequest::GetBalance
            .build_request_json(
                self.request_id,
                json!([
                    json!(account_pubkey.to_string()),
                    json!(CommitmentConfig::finalized())
                ]),
            )
            .to_string();

        let response: RpcResponse<RpcResultWithContext<u64>> = self.send(request).await?;
        Ok(response.result.value)
    }

    /// Returns the minimum balance (in Lamports) required for an account to be rent exempt.
    pub async fn get_minimum_balance_for_rent_exemption(
        &mut self,
        data_len: usize,
    ) -> ClientResult<u64> {
        let request = RpcRequest::GetMinimumBalanceForRentExemption
            .build_request_json(self.request_id, json!([data_len]))
            .to_string();

        let response: RpcResponse<u64> = self.send(request).await?;
        Ok(response.result)
    }

    /// Requests an airdrop of lamports to a given account.
    pub async fn request_airdrop(
        &mut self,
        pubkey: &Pubkey,
        lamports: u64,
        recent_blockhash: &Hash,
    ) -> ClientResult<Signature> {
        let config = RpcRequestAirdropConfig {
            recent_blockhash: Some(recent_blockhash.to_string()),
            commitment: Some(CommitmentLevel::Finalized),
        };
        let request = RpcRequest::RequestAirdrop
            .build_request_json(
                self.request_id,
                json!([pubkey.to_string(), lamports, config]),
            )
            .to_string();

        let response: RpcResponse<String> = self.send(request).await?;

        let signature = Signature::from_str(&response.result)?;
        Ok(signature)
    }

    /// Returns latest blockhash.
    pub async fn get_latest_blockhash(&mut self) -> ClientResult<Hash> {
        // TODO for some reason latest blockhash returns method not found
        // even though we are using 1.9.0 and the rpc servers are also updated
        let request = RpcRequest::GetRecentBlockhash
            .build_request_json(self.request_id, json!([]))
            .to_string();

        let response: RpcResponse<RpcResultWithContext<Blockhash>> = self.send(request).await?;
        let blockhash = Hash::from_str(&response.result.value.blockhash)?;
        Ok(blockhash)
    }

    /// Attempts to send a signed transaction to the ledger without simulating
    /// it first.
    ///
    /// It is a bit faster, but no logs or confirmation is returned because the
    /// transaction is not simulated.
    pub async fn send_transaction_unchecked(
        &mut self,
        transaction: &Transaction,
    ) -> ClientResult<Signature> {
        let config = RpcTransactionConfig {
            skip_preflight: true,
            preflight_commitment: Some(CommitmentLevel::Processed),
            encoding: Some(Encoding::Base64),
        };
        self.send_transaction_with_config(transaction, &config)
            .await
    }

    pub async fn send_transaction(&mut self, transaction: &Transaction) -> ClientResult<Signature> {
        let config = RpcTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentLevel::Confirmed),
            encoding: Some(Encoding::Base64),
        };
        self.send_transaction_with_config(transaction, &config)
            .await
    }

    pub async fn send_transaction_with_config(
        &mut self,
        transaction: &Transaction,
        config: &RpcTransactionConfig,
    ) -> ClientResult<Signature> {
        let serialized = bincode::serialize(transaction)?;
        let encoded = base64::encode(serialized);
        let request = RpcRequest::SendTransaction
            .build_request_json(self.request_id, json!([json!(encoded), json!(config)]))
            .to_string();

        match self.send::<serde_json::Value, String>(request).await {
            Ok(json_value) => {
                if let Ok(response) =
                    serde_json::from_value::<RpcResponse<String>>(json_value.clone())
                {
                    let signature = Signature::from_str(&response.result)?;
                    Ok(signature)
                } else if let Ok(tx_error) =
                    serde_json::from_value::<RpcResponse<RpcTransactionError>>(json_value)
                {
                    tx_error
                        .result
                        .data
                        .logs
                        .iter()
                        .enumerate()
                        .for_each(|(i, log)| debug!("{} {}", i, log));
                    bail!("{}", tx_error.result.message);
                } else {
                    bail!("failed to parse RPC response")
                }
            }
            Err(err) => bail!(err),
        }
    }

    pub async fn get_slot(&mut self) -> ClientResult<Slot> {
        let request = RpcRequest::GetSlot
            .build_request_json(self.request_id, json!([]))
            .to_string();

        let response: RpcResponse<Slot> = self.send(request).await?;
        Ok(response.result)
    }

    pub async fn get_block_time(&mut self, slot: Slot) -> ClientResult<UnixTimestamp> {
        let request = RpcRequest::GetBlockTime
            .build_request_json(self.request_id, json!([slot]))
            .to_string();

        let response: RpcResponse<UnixTimestamp> = self.send(request).await?;
        Ok(response.result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_sdk::signer::keypair::Keypair;
    use solana_sdk::signer::Signer;
    use solana_sdk::system_transaction::transfer;

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[rustfmt::skip]
    const ALICE: &[u8] = &[
        57,99,241,156,126,127,97,60,
        40,14,39,4,115,72,39,75,
        2,14,30,255,45,79,195,202,
        132,18,131,180,61,12,87,183,
        14,175,192,115,62,33,136,190,
        244,254,192,174,2,126,227,113,
        222,42,224,89,36,89,239,167,
        22,150,31,29,89,188,176,162
    ];

    #[rustfmt::skip]
    const BOB: &[u8] = &[
        176,252,96,172,240,61,215,84,
        138,250,147,178,208,59,227,60,
        190,204,80,88,55,137,236,252,
        231,118,253,64,65,106,39,5,
        14,212,250,187,124,127,43,205,
        30,117,63,227,13,218,202,68,
        160,161,52,12,59,211,152,183,
        119,140,213,205,174,210,108,128
    ];

    const AIRDROP_AMOUNT: u64 = 5500; // tx free of 5000 lamports included
    const TRANSFER_AMOUNT: u64 = 250;

    async fn wait_for_balance_change(
        client: &mut RpcClient,
        account: &Pubkey,
        balance_before: u64,
        expected_change: u64,
    ) {
        let mut i = 0;
        let max_loops = 60;
        loop {
            let balance_after = client.get_balance(account).await.unwrap();
            // NOTE might happen that alice is airdropped only after she
            // transferred the amount to BOB
            match balance_after.checked_sub(balance_before) {
                Some(0) => i += 1,
                Some(delta) => {
                    assert_eq!(delta, expected_change);
                    break;
                }
                None => {
                    assert_eq!(balance_before - balance_after, expected_change);
                    break;
                }
            }
            if i == max_loops {
                panic!("test was running for {} loops", max_loops);
            }
        }
    }

    #[tokio::test]
    async fn airdrop_and_transfer() {
        let alice = Keypair::from_bytes(ALICE).unwrap();
        let bob = Keypair::from_bytes(BOB).unwrap();
        let mut client = RpcClient::new(Net::Devnet);

        let balance_before_airdrop_alice = client.get_balance(&alice.pubkey()).await.unwrap();
        let latest_blockhash = client.get_latest_blockhash().await.unwrap();

        client
            .request_airdrop(&alice.pubkey(), AIRDROP_AMOUNT, &latest_blockhash)
            .await
            .unwrap();

        wait_for_balance_change(
            &mut client,
            &alice.pubkey(),
            balance_before_airdrop_alice,
            AIRDROP_AMOUNT,
        )
        .await;

        let balance_before_bob = client.get_balance(&bob.pubkey()).await.unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transfer_tx = transfer(&alice, &bob.pubkey(), TRANSFER_AMOUNT, recent_blockhash);
        client.send_transaction(&transfer_tx).await.unwrap();

        wait_for_balance_change(
            &mut client,
            &bob.pubkey(),
            balance_before_bob,
            TRANSFER_AMOUNT,
        )
        .await;

        wait_for_balance_change(
            &mut client,
            &alice.pubkey(),
            balance_before_airdrop_alice,
            TRANSFER_AMOUNT, // also losing the 5000 lamport fee
        )
        .await;
    }

    #[tokio::test]
    async fn block_time() {
        let mut client = RpcClient::new(Net::Mainnet);
        for _ in 0..10 {
            let slot = client.get_slot().await.unwrap();
            let block_time = client.get_block_time(slot).await.unwrap();
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let delta_time = (time - block_time) as f32;
            assert!(delta_time.abs() < 60.0); // we are within one minute
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}
