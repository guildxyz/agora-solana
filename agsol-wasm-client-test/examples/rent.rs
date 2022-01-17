use agsol_wasm_client::{Net, RpcClient};
use reqwest::Client;
use serde::Deserialize;

const LAMPORTS: f32 = 1e9;
const COIN_GECKO_API_URL: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd";

#[derive(Deserialize)]
struct SolUsd {
    solana: Usd,
}

#[derive(Deserialize)]
struct Usd {
    usd: f32,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut client = RpcClient::new(Net::Devnet);
    let response = Client::new()
        .get(COIN_GECKO_API_URL)
        .send()
        .await?
        .json::<SolUsd>()
        .await?;

    for &bytes in &[0, 1, 32, 320, 3200, 32_000, 320_000, 3_200_000, 10_000_000] {
        let price = compute_price(&mut client, bytes, response.solana.usd).await?;
        println!("rent for {:>9} bytes is {} USD", bytes, price);
    }

    Ok(())
}

async fn compute_price(
    client: &mut RpcClient,
    bytes: usize,
    price: f32,
) -> Result<f32, anyhow::Error> {
    let rent: u64 = client.get_minimum_balance_for_rent_exemption(bytes).await?;
    Ok(rent as f32 * price / LAMPORTS)
}
