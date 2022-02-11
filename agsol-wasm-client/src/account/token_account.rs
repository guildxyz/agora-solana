use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodedMint {
    pub decimals: u8,
    pub supply: String,
    pub freeze_authority: Option<String>,
    pub mint_authority: Option<String>,
    pub is_initialized: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodedTokenAccount {
    pub mint: String,
    pub owner: String,
    pub is_native: bool,
    pub token_amount: TokenAmount,
    pub state: TokenAccountState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f32,
    pub ui_amount_string: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TokenAccountState {
    Uninitialized,
    Initialized,
    Frozen,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "info")]
pub enum TokenAccount {
    Account(EncodedTokenAccount),
    Mint(EncodedMint),
}
