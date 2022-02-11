use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub program_data: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "info")]
pub enum ProgramAccount {
    Uninitialized,
    Program(Program),
    // ProgramData(AccountData), -> from account/mod.rs
}
