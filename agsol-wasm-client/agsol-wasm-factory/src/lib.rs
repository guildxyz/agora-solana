use heck::MixedCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

// NOTE: when using `wasm_instruction!()` in a separate crate,
// the following crates are needed to be included:
// wasm_bindgen, solana_program, serde_json
// NOTE: unwrapping instruction to json string is fine because `Instruction` is always
// serializable
/// Macro for porting Solana
/// [`Instruction`](https://docs.rs/solana-program/latest/solana_program/instruction/struct.Instruction.html)
/// factories to Wasm.
///
/// # Examples
/// ```rust
/// # use agsol_wasm_factory::wasm_instruction;
/// use borsh::BorshDeserialize;
/// use solana_program::instruction::Instruction;
/// use solana_program::pubkey::Pubkey;
/// # use wasm_bindgen::prelude::*;
///
/// #[derive(BorshDeserialize)]
/// struct FooArgs {
///     foo: Option<u32>,
///     bar: Pubkey,
///     baz: u64,
/// }
///
/// fn foo_factory(args: &FooArgs) -> Instruction {
///     // ... computations with args
///     # let data = Vec::new();
///     # let accounts = Vec::new();
///     # let program_id = Pubkey::default();
///
///     Instruction {
///         program_id,
///         accounts,
///         data,
///     }
/// }
///
/// wasm_instruction!(foo_factory);
/// // generated output:
/// // #[wasm_bindgen, js_name = "fooFactory"]
/// // fn foo_factory_wasm(serialized_input: &[u8]) -> Result<String, JsValue> { ... }
/// ```
#[proc_macro]
pub fn wasm_instruction(input: TokenStream) -> TokenStream {
    let instruction_name = Ident::new(&input.to_string(), Span::call_site());
    let function_name = Ident::new(&(input.to_string() + "_wasm"), Span::call_site());
    let wasm_name = function_name.to_string().to_mixed_case();

    let output = quote! {
        #[wasm_bindgen(js_name = #wasm_name)]
        pub fn #function_name(serialized_input: &[u8]) -> Result<String, JsValue> {
            let args = solana_program::borsh::try_from_slice_unchecked(serialized_input)
                .map_err(|e| wasm_bindgen::prelude::JsValue::from(e.to_string()))?;
            let instruction = #instruction_name(&args);
            Ok(serde_json::to_string(&instruction).unwrap())
        }
    };
    output.into()
}
