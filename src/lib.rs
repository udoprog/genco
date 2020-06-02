//! ## GenCo
//!
//! GenCo is an even simpler code generator for Rust, written for use in [reproto].
//!
//! The workhorse of GenCo is the [`quote!`] macro. While tokens can be constructed,
//! manually, [`quote!`] makes this process much easier.
//!
//! GenCo does not deal with language-specific syntax, instead it limits itself to
//! do the following basic necessities through specialization:
//!
//! * Handle and collapse import statements.
//! * Quote strings according to language convention.
//! * Indents and spaces your code according to generic [indentation rules].
//!
//! ## Examples
//!
//! The following are language specific examples for GenCo using the [`quote!`]
//! macro.
//!
//! * [Rust Example]
//! * [Java Example]
//! * [C# Example]
//! * Dart Example (TODO)
//! * Go Example (TODO)
//! * JavaScript Example (TODO)
//! * Python Example (TODO)
//!
//! The following is a simple example showcasing code generation for Rust.
//!
//! ```rust
//! #![feature(proc_macro_hygiene)]
//!
//! use genco::rust::imported;
//! use genco::{quote, Rust, Tokens};
//!
//! // Import the LittleEndian item, without referencing it through the last
//! // module component it is part of.
//! let little_endian = imported("byteorder", "LittleEndian").qualified();
//! let big_endian = imported("byteorder", "BigEndian");
//!
//! // This is a trait, so only import it into the scope (unless we intent to
//! // implement it).
//! let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
//!
//! let tokens: Tokens<Rust> = quote! {
//!     @write_bytes_ext
//!
//!     let mut wtr = vec![];
//!     wtr.write_u16::<#little_endian>(517).unwrap();
//!     wtr.write_u16::<#big_endian>(768).unwrap();
//!     assert_eq!(wtr, vec![5, 2, 3, 0]);
//! };
//! ```
//!
//! ## Indentation Rules
//!
//! The `quote!` macro has the following rules for dealing with indentation and
//! spacing.
//!
//! **Two tokens** that are separated, are spaced. Regardless of how many spaces
//! there are between them.
//!
//! So:
//!
//! ```rust
//! #![feature(proc_macro_hygiene)]
//!
//! let _: genco::Tokens<genco::Rust> = genco::quote!(fn   test() {});
//! ```
//!
//! Becomes:
//!
//! ```rust
//! fn test() {}
//! ```
//!
//! **More that two line breaks** are collapsed.
//!
//! So:
//!
//! ```rust
//! #![feature(proc_macro_hygiene)]
//!
//! let _: genco::Tokens<genco::Rust> = genco::quote! {
//!     fn test() {
//!         println!("Hello...");
//!
//!
//!         println!("... World!");
//!     }
//! };
//! ```
//!
//! Becomes:
//!
//! ```rust
//! fn test() {
//!     println!("Hello...");
//!
//!     println!("... World!");
//! }
//! ```
//!
//! **Indentation** is determined on a row-by-row basis. If a column is further in
//! than the one on the preceeding row, it is indented **one level** deeper.
//!
//! Like wise if a column starts before the previous rows column, it is indended one
//! level shallower.
//!
//! So:
//!
//! ```rust
//! #![feature(proc_macro_hygiene)]
//!
//! let _: genco::Tokens<genco::Rust> = genco::quote! {
//!   fn test() {
//!       println!("Hello...");
//!       println!("... World!");
//!     }
//! };
//! ```
//!
//! Becomes:
//!
//! ```rust
//! fn test() {
//!     println!("Hello...");
//!     println!("... World!");
//! }
//! ```
//!
//! [reproto]: https://github.com/reproto/reproto
//! [indentation rules]: https://github.com/udoprog/genco#indentation-rules
//! [Rust Example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
//! [Java Example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
//! [C# Example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
//! [`quote!`]: https://github.com/udoprog/genco/blob/master/tests/test_quote.rs

#![deny(missing_docs)]
#![feature(proc_macro_hygiene)]
// Note: Replace internal use of macros `push!`, `nested!` with `quote!`.
#![allow(deprecated)]

pub use genco_derive::quote;

#[macro_use]
mod macros;
mod con_;
mod config;
mod cons;
mod element;
mod erased_element;
mod format_tokens;
mod formatter;
mod lang;
mod quoted;
mod tokens;
mod write_tokens;

pub(crate) use self::con_::Con;
pub use self::config::Config;
pub use self::cons::Cons;
pub use self::element::Element;
pub use self::erased_element::ErasedElement;
pub use self::format_tokens::FormatTokens;
pub use self::formatter::{Formatter, IoFmt};
pub use self::lang::*;
pub use self::quoted::Quoted;
pub use self::tokens::Tokens;
pub use self::write_tokens::WriteTokens;

#[cfg(test)]
mod tests {
    use crate::rust::Rust;
    use crate::tokens::Tokens;

    #[test]
    fn test_nested() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push("fn foo() -> u32 {");
        toks.nested("return 42;");
        toks.push("}");

        let output = toks.to_string().unwrap();
        assert_eq!("fn foo() -> u32 {\n    return 42;\n}", output.as_str());
    }
}
