//! This is an experimental crate that aims to implement a Wasm-compatible
//! non-blocking (async) [`RpcClient`] for Solana. Currently, Solana uses its
//! own *blocking*
//! [`RpcClient`](https://docs.rs/solana-client/latest/solana_client/rpc_client/struct.RpcClient.html#method.confirm_transaction)
//! for querying the blockchain which cannot be ported to Wasm.
//!
//! This crate uses an async
//! [`reqwest::Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html)
//! to make [`RpcRequest`]s to the blockchain. Thus, querying the blockchain
//! can be written entirely in Rust and then it can be easily ported to Wasm
//! using [`wasm_bindgen`](https://docs.rs/wasm-bindgen/latest/wasm_bindgen/).
//!
//! # Examples
//! ```no_run
//! use agsol_wasm_client::{Net, RpcClient};
//! use solana_program::pubkey::Pubkey;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = RpcClient::new(Net::Devnet);
//!     let balance = client
//!         .get_minimum_balance_for_rent_exemption(50)
//!         .await
//!         .unwrap();
//!
//!     let owner = client.get_owner(&Pubkey::default()).await.unwrap();
//! }
//! ```
//!
//! When built with the `wasm-factory` feature enabled, this crate provides the
//! `wasm_instruction!()` macro that can be used for quickly exposing a Solana
//! [`Instruction`](https://docs.rs/solana-program/latest/solana_program/instruction/struct.Instruction.html)
//! factory to Wasm.
mod account;
mod rpc_client;
mod rpc_config;
mod rpc_request;
mod rpc_response;

pub use account::Account;
pub use rpc_client::{Net, RpcClient};
pub use rpc_config::{Encoding, RpcConfig};
pub use rpc_request::RpcRequest;

#[cfg(any(test, feature = "wasm-factory"))]
#[allow(unused_imports)]
#[macro_use]
extern crate agsol_wasm_factory;
#[cfg(any(test, feature = "wasm-factory"))]
pub use agsol_wasm_factory::*;
