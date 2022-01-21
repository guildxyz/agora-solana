use serde::{Deserialize, Serialize};

/// Configuration for building an RPC request.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcConfig {
    pub encoding: Option<Encoding>,
}

/// Required parameter in the RPC request that specifies the encoding of the
/// RPC data.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Encoding {
    Base58,
    Base64,
    JsonParsed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum CommitmentLevel {
    Processed,
    Confirmed,
    Finalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcRequestAirdropConfig {
    pub recent_blockhash: Option<String>,
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcRequestTransactionConfig {
    #[serde(default)]
    pub skip_preflight: bool,
    pub preflight_commitment: Option<CommitmentLevel>,
    pub encoding: Option<Encoding>,
}
