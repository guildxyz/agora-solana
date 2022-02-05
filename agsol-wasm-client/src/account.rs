use serde::Deserialize;
use solana_program::pubkey::Pubkey;

/// The (partial) contents of a Solana account.
#[derive(Clone, Debug)]
pub struct Account {
    /// Account balance in Lamports.
    pub lamports: u64,
    /// Serialized account data.
    pub data: Vec<u8>,
    /// The owner of the account.
    pub owner: Pubkey,
    /// Is the program executable?
    pub executable: bool,
    /// The epoch at which this account will next owe rent.
    pub rent_epoch: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodedAccount {
    lamports: u64,
    data: [String; 2],
    owner: String,
    executable: bool,
    rent_epoch: u64,
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
            executable: self.executable,
            rent_epoch: self.rent_epoch,
        })
    }
}
