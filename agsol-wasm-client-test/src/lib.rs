// This is necessary because clippy throws 'unneeded unit expression' error
// on the wasm_bindgen expressions
#![allow(clippy::unused_unit)]



use agsol_wasm_client::{wasm_borsh_instruction, wasm_serde_instruction, Net, RpcClient};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use std::str::FromStr;

const TEST_PUBKEY_STR: &str = "PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT";

#[wasm_bindgen(js_name = "getOwner")]
pub async fn get_owner(account: Pubkey) -> Result<Pubkey, JsValue> {
    let mut client = RpcClient::new(Net::Devnet);
    let account = client
        .get_account(&account)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    let decoded_bytes = bs58::decode(account.owner)
        .into_vec()
        .map_err(|e| JsValue::from(e.to_string()))?;
    let owner = Pubkey::new(&decoded_bytes);
    Ok(owner)
}

#[wasm_bindgen(js_name = "getLamports")]
pub async fn get_lamports(account: Pubkey) -> Result<u64, JsValue> {
    let mut client = RpcClient::new(Net::Devnet);
    let account = client
        .get_account(&account)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    Ok(account.lamports)
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
struct TestInstructionArgs {
    pubkey: Pubkey,
    input: Option<u32>,
}

#[derive(BorshSerialize)]
enum TestInstructionEnum {
    FooInstruction { input: Option<u32> },
}

fn foo_seeds(pk: &Pubkey) -> [&[u8]; 2] {
    [b"foo", pk.as_ref()]
}

fn bar_seeds(pk: &Pubkey) -> [&[u8]; 2] {
    [b"bar", pk.as_ref()]
}

fn test_instruction(args: &TestInstructionArgs) -> Instruction {
    let program_id_bytes = bs58::decode(TEST_PUBKEY_STR).into_vec().unwrap();
    let program_id = Pubkey::new(&program_id_bytes);
    let (foo_pubkey, _) = Pubkey::find_program_address(&foo_seeds(&args.pubkey), &program_id);
    let (bar_pubkey, _) = Pubkey::find_program_address(&bar_seeds(&args.pubkey), &program_id);

    let accounts = vec![
        AccountMeta::new(args.pubkey, true),
        AccountMeta::new(foo_pubkey, false),
        AccountMeta::new_readonly(bar_pubkey, false),
    ];

    let instruction = TestInstructionEnum::FooInstruction { input: args.input };

    Instruction {
        program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

wasm_borsh_instruction!(test_instruction);

#[test]
fn test_wasm_borsh_instruction_factory() {
    let args = TestInstructionArgs {
        pubkey: Pubkey::new_unique(),
        input: Some(15),
    };
    let serialized_instruction = args.try_to_vec().unwrap();
    let instruction: Instruction =
        serde_json::from_str(&test_instruction_borsh_wasm(&serialized_instruction).unwrap()).unwrap();
    assert_eq!(instruction.data, &[0, 1, 15, 0, 0, 0]);
    assert_eq!(instruction.accounts[0].pubkey, args.pubkey);
    assert_eq!(
        instruction.accounts[1].pubkey.to_string(),
        "HKp9TzCTQ79TE4eppvHWUUXVaZePZSJCYkExtEVYjezP"
    );
    assert_eq!(
        instruction.accounts[2].pubkey.to_string(),
        "6UUakecVHBoXBxh6sbQd9mEx6yikDQ9cy1f7jobcyucc"
    );
    assert!(instruction.accounts[0].is_writable);
    assert!(instruction.accounts[1].is_writable);
    assert!(!instruction.accounts[2].is_writable);
    assert!(instruction.accounts[0].is_signer);
    assert!(!instruction.accounts[1].is_signer);
    assert!(!instruction.accounts[2].is_signer);
    let program_id_bytes = bs58::decode(TEST_PUBKEY_STR).into_vec().unwrap();
    let program_id = Pubkey::new(&program_id_bytes);
    assert_eq!(instruction.program_id, program_id);
}

// TODO: generate this with the macro as well
#[derive(Deserialize, Debug)]
struct FrontendTestInstructionArgs {
    pubkey: String,
    input: Option<u32>,
}

impl TryFrom<FrontendTestInstructionArgs> for TestInstructionArgs {
    type Error = String;
    fn try_from(args: FrontendTestInstructionArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            pubkey: Pubkey::from_str(&args.pubkey).map_err(|e| e.to_string())?,
            input: args.input,
        })
    }
}

wasm_serde_instruction!(test_instruction);

#[test]
fn test_wasm_serde_instruction_factory() {
    let args = TestInstructionArgs {
        pubkey: Pubkey::new_unique(),
        input: Some(15),
    };
    let args_json = JsValue::from_serde(&serde_json::to_string(&args).unwrap()).unwrap();
    let instruction: Instruction = 
        serde_json::from_str(&test_instruction_serde_wasm(args_json).unwrap().as_string().unwrap()).unwrap();
    assert_eq!(instruction.data, &[0, 1, 15, 0, 0, 0]);
    assert_eq!(instruction.accounts[0].pubkey, args.pubkey);
    assert_eq!(
        instruction.accounts[1].pubkey.to_string(),
        "HKp9TzCTQ79TE4eppvHWUUXVaZePZSJCYkExtEVYjezP"
    );
    assert_eq!(
        instruction.accounts[2].pubkey.to_string(),
        "6UUakecVHBoXBxh6sbQd9mEx6yikDQ9cy1f7jobcyucc"
    );
    assert!(instruction.accounts[0].is_writable);
    assert!(instruction.accounts[1].is_writable);
    assert!(!instruction.accounts[2].is_writable);
    assert!(instruction.accounts[0].is_signer);
    assert!(!instruction.accounts[1].is_signer);
    assert!(!instruction.accounts[2].is_signer);
    let program_id_bytes = bs58::decode(TEST_PUBKEY_STR).into_vec().unwrap();
    let program_id = Pubkey::new(&program_id_bytes);
    assert_eq!(instruction.program_id, program_id);
}
