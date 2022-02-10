use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodedMint {
    pub decimals: u8,
    pub supply: u64,
    pub freeze_authority: Option<String>,
    pub mint_authority: Option<String>,
    pub is_intialized: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodedTokenAccount {
    pub mint: String,
    pub owner: String,
    pub is_native: bool,
    pub state: TokenAccountState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TokenAccountState {
    Uninitialized,
    Initialized,
    Frozen,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "info")]
pub enum TokenAccountType {
    Account(EncodedTokenAccount),
    Mint(EncodedMint),
}
