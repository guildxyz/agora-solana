use solana_program::pubkey::Pubkey;
use solana_program_runtime::invoke_context::ProcessInstructionWithContext;

/// A BPF program that can be loaded into the testbench.
///
/// If the `process_instruction` field is `None`, the testbench will look for a
/// compiled BPF file in the `tests/fixtures` directory based on the `name` and
/// `program_id`.
pub struct TestbenchProgram<'a> {
    pub name: &'a str,
    pub id: Pubkey,
    pub process_instruction: Option<ProcessInstructionWithContext>,
}
