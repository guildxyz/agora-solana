use serde::Deserialize;
use solana_program::pubkey::Pubkey;

/// The (partial) contents of a Solana account.
#[derive(Clone, Debug)]
pub struct Account {
    /// Account balance in Lamports
    pub lamports: u64,
    /// Serialized account data
    pub data: Vec<u8>,
    /// The owner of the account
    pub owner: Pubkey,
}

#[derive(Deserialize, Debug)]
pub struct EncodedAccount {
    lamports: u64,
    data: [String; 2],
    owner: String,
}

impl EncodedAccount {
    pub fn decode(self) -> Result<Account, anyhow::Error> {
        let [data_string, encoding] = self.data;
        let data = match encoding.as_str() {
            "base64" => base64::decode(data_string)?,
            _ => return Err(anyhow::anyhow!("encoding {} is not implemented", encoding)),
        };
        let pubkey_bytes = bs58::decode(&self.owner).into_vec()?;
        let owner = Pubkey::new(&pubkey_bytes);

        Ok(Account {
            lamports: self.lamports,
            data,
            owner,
        })
    }
}
