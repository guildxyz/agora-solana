//! This library aims to facilitate the workflow between Rust and TypeScript data
//! structures by auto-generating ! TypeScript classes and respective serialization
//! layouts used for Borsh (de)serialization. Check out
//! [`borsh-js`](https://github.com/near/borsh-js) and
//! [`borsh-rs`](https://docs.rs/borsh/0.9.1/borsh/index.html) for more details.
//!
//! By default the library provides a derivable trait `BorshSchema` without any
//! associated methods and constants. It's an empty trait that is essentially a
//! flag for the schema parser that works the following way:
//!
//! 1) the parser traverses all `.rs` files in the provided input directory
//!
//! 2) data structures (`struct`s and `enum`s) annotated with
//!    `#[derive(BorshSchema, ...)]` are parsed into an intermediate data
//!    structure
//!
//! 3) the intermediate data structure is used to generate output files
//!    containing TypeScript classes and serialization schemas
//!
//! The parser itself is only available through the `full` feature flag,
//! because it uses parsing libraries incompatible with `wasm` or `bpf`
//! targets.

pub use agsol_borsh_schema_derive::*;

/// Intermediate data structures used for generating
/// schema an TypeScript class layouts.
#[cfg(feature = "full")]
mod layout;
#[cfg(all(test, feature = "full"))]
mod test;
#[cfg(feature = "full")]
mod utils;

#[cfg(feature = "full")]
pub use utils::*;

/// An empty trait that serves as a flag for the schema parser.
///
/// It has an `alias` attribute that can be used to annotate `struct` and
/// `enum` fields to explicitly indicate the type of that field. This is needed
/// because the parser reads the file as a raw string, therefore it has no way
/// of knowing the underlying type of a type alias.
///
/// # Example
/// ```rust
/// use agsol_borsh_schema::BorshSchema;
/// use std::collections::BTreeMap;
///
/// type SomeAlias = [u8; 32];
///
/// #[derive(BorshSchema)]
/// struct Foo {
///     foo: Option<u64>,
///     bar: BTreeMap<u8, Bar>,
///     #[alias([u8; 32])]
///     baz: SomeAlias,
/// }
///
/// #[derive(BorshSchema)]
/// enum Bar {
///     A,
///     B,
///     C(u64),
///     D {
///         foo: i32,
///         bar: String,
///     },
/// }
/// ```
///
/// In the above example you may notice that `Foo`'s `bar` field doesn't need
/// an alias because `Bar` implements `BorshSchema` itself, however, the parser
/// doesn't know that `SomeAlias` is actually a byte array without the `alias`
/// attribute. If the `alias` attribute is omitted, the generated TypeScript
/// code will contain `SomeAlias` instead of `Uint8Array`.
pub trait BorshSchema {}
