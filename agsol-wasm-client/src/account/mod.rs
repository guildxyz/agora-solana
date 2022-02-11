mod program_account;
mod token_account;
pub use program_account::*;
pub use token_account::*;

use crate::rpc_config::Encoding;

use anyhow::bail;
use borsh::BorshDeserialize;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use solana_program::borsh::try_from_slice_unchecked;

/// The (partial) contents of a Solana account.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Account balance in Lamports.
    pub lamports: u64,
    /// Serialized account data.
    pub data: AccountData,
    /// The bs58-encoded pubkey of the account owner.
    pub owner: String,
    /// Is the program executable?
    pub executable: bool,
    /// The epoch at which this account will next owe rent.
    pub rent_epoch: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
pub enum AccountData {
    Encoded(String, Encoding),
    JsonParsed(ParsedAccount),
}

impl AccountData {
    pub fn parse_into_borsh<T: BorshDeserialize>(self) -> Result<T, anyhow::Error> {
        match self {
            Self::Encoded(data_string, encoding) => {
                let decoded = match encoding {
                    Encoding::Base64 => base64::decode(data_string)?,
                    _ => {
                        bail!("encoding {:?} is not implemented", encoding)
                    }
                };
                try_from_slice_unchecked::<T>(&decoded).map_err(|e| anyhow::anyhow!(e))
            }
            _ => bail!("cannot borsh-deserialize data"),
        }
    }

    pub fn parse_into_json<T: DeserializeOwned>(self) -> Result<T, anyhow::Error> {
        match self {
            Self::JsonParsed(account) => {
                serde_json::from_value::<T>(account.parsed).map_err(|e| anyhow::anyhow!(e))
            }
            _ => bail!("cannot json-deserialize data"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ParsedAccount {
    pub parsed: serde_json::Value,
    pub program: String,
    pub space: u64,
}

#[cfg(test)]
mod test {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;

    #[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
    struct TestData {
        num: u8,
        owner: Pubkey,
        amount: u64,
    }

    #[test]
    fn parse_borsh() {
        let owner = Pubkey::new_unique();
        let test_data = TestData {
            num: 123,
            owner,
            amount: 987654321,
        };

        let borsh_serialized = test_data.try_to_vec().unwrap();
        let encoded = base64::encode(borsh_serialized);

        let account_data = AccountData::Encoded(encoded, Encoding::Base64);

        let deserialized_data = account_data.parse_into_borsh::<TestData>().unwrap();
        assert_eq!(test_data, deserialized_data);
    }
}
