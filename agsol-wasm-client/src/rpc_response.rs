use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub id: u64,
    pub jsonrpc: String,
    pub result: T,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct Context {
    pub slot: u64,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct RpcResultWithContext<T> {
    pub context: Context,
    pub value: T,
}
