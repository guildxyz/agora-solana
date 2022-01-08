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
    JsonParsed,
}
