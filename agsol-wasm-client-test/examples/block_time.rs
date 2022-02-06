use agsol_wasm_client::rpc_config::{CommitmentLevel, Encoding, RpcConfig};
use agsol_wasm_client::{Net, RpcClient};

#[tokio::main]
async fn main() {
    let config = RpcConfig {
        encoding: Some(Encoding::JsonParsed),
        commitment: Some(CommitmentLevel::Confirmed),
    };
    let mut client = RpcClient::new_with_config(Net::Devnet, config);

    loop {
        let slot = client.get_slot().await.unwrap();
        let block_time = client.get_block_time(slot).await.unwrap();

        println!("{}", block_time);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
