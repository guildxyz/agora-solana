//! Convenience data structures and methods for interacting with the
//! Solana program test context.
//!
//! # Examples
//!
//! Assume there's a Solana program with a processor of the form
//! ```
//! # use solana_program::pubkey::Pubkey;
//! # use solana_program::account_info::AccountInfo;
//! # use solana_program::program_error::ProgramError;
//! // foo_program::processor
//! fn processor(
//!     id: &Pubkey,
//!     accounts: &[AccountInfo],
//!     data: &[u8]
//! ) -> Result<(), ProgramError>
//! {
//!     // snip
//!     Ok(())
//! }
//! ```
//! It can be loaded into the testbench as
//!
//! ```no_run
//! use agsol_testbench::{Testbench, TestbenchProgram, TestUser};
//! use solana_program_test::processor;
//! # use solana_program::pubkey::Pubkey;
//! # use solana_program::account_info::AccountInfo;
//! # use solana_program::program_error::ProgramError;
//! # use solana_program_test::tokio;
//! # mod foo_program {
//! # use super::*;
//! # pub fn processor(id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> Result<(), ProgramError> { Ok(()) }
//! # }
//! # #[tokio::main]
//! # async fn main() {
//! #   let FOO_PROGRAM_ID = Pubkey::new(&[0; 32]);
//! #   let BAR_PROGRAM_ID = Pubkey::new(&[1_u8; 32]);
//!
//! // program `foo` with program pubkey FOO_PROGRAM_ID
//! let program_from_processor = TestbenchProgram {
//!     name: "foo_bpf_program",
//!     id: FOO_PROGRAM_ID,
//!     process_instruction: processor!(foo_program::processor),
//! };
//!
//! // program `bar` with program pubkey BAR_PROGRAM_ID
//! let program_from_binary = TestbenchProgram {
//!     name: "bar_bpf_program",
//!     id: BAR_PROGRAM_ID,
//!     process_instruction: None,
//! };
//!
//! // load programs into the test context
//! let mut testbench = Testbench::new(&[program_from_processor, program_from_binary]).await;
//!
//! // create a test user with an airdrop
//! let test_user = TestUser::new(&mut testbench).await;
//! # }
//! ```

mod test_user;
mod testbench;
mod testbench_program;

pub use solana_program_test::{self, tokio};
pub use test_user::TestUser;
pub use testbench::Testbench;
pub use testbench_program::TestbenchProgram;
