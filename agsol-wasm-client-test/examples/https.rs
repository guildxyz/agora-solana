use agsol_wasm_client::{Net, RpcClient};
use solana_program::pubkey::Pubkey;

#[tokio::main]
async fn main() {
    let mut client = RpcClient::new(Net::Devnet);
    //let pubkey_str = "7z9HJcqrouhUHo3EkbVXRtRxGccJxGGNUYy8AdbseoZa";
    //let pubkey_bytes = bs58::decode(pubkey_str).into_vec().unwrap();
    //let pubkey = Pubkey::new(&pubkey_bytes);

    let rent: u64 = client
        .get_minimum_balance_for_rent_exemption(32)
        .await
        .unwrap();
    println!("{:?}", rent);
}
