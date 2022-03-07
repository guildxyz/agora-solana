use heck::{MixedCase, CamelCase};
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
/// wasm_borsh_instruction!(foo_factory);
/// // generated output:
/// // #[wasm_bindgen, js_name = "fooFactoryBorshWasm"]
/// // fn foo_factory_borsh_wasm(serialized_input: &[u8]) -> Result<String, JsValue> { ... }
/// ```
#[proc_macro]
pub fn wasm_borsh_instruction(input: TokenStream) -> TokenStream {
    let instruction_name = Ident::new(&input.to_string(), Span::call_site());
    let function_name = Ident::new(&(input.to_string() + "_borsh_wasm"), Span::call_site());
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

/// wasm_serde_instruction!(foo_factory);
/// // generated output:
/// // #[wasm_bindgen, js_name = "fooFactorySerdeWasm"]
/// // fn foo_factory_serde_wasm(args: JsValue) -> Result<JsValue, JsValue> { ... }
#[proc_macro]
pub fn wasm_serde_instruction(input: TokenStream) -> TokenStream {
    let instruction_name = Ident::new(&input.to_string(), Span::call_site());
    let function_name = Ident::new(&(input.to_string() + "_serde_wasm"), Span::call_site());
    let wasm_name = function_name.to_string().to_mixed_case();
    let frontend_struct_name = Ident::new(&(Ident::new(
        &("Frontend_".to_string() + &input.to_string() + "_args"),
        Span::call_site(),
    )
    .to_string()
    .to_camel_case()), Span::call_site());

    let output = quote! {
        #[wasm_bindgen(js_name = #wasm_name)]
        // TODO: async?
        pub fn #function_name(args: JsValue) -> Result<JsValue, JsValue> {
            let frontend_args: #frontend_struct_name = args
                .into_serde()
                .map_err(|e| JsValue::from(e.to_string()))?;

            let args = frontend_args.try_into().map_err(JsValue::from)?;
            let instruction = #instruction_name(&args);
            JsValue::from_serde(&instruction).map_err(|e| JsValue::from(e.to_string()))
        }
    };
    output.into()
}
