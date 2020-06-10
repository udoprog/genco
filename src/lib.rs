//! ## genco
//!
//! genco is an even simpler code generator for Rust, written for use in [reproto].
//!
//! We depend on `proc_macro_hygiene` stabilizations. Until then, you must build
//! with the `nightly` branch.
//!
//! ```bash
//! cargo +nightly run --example rust
//! ```
//!
//! The workhorse of genco is the [quote!] and [quote_in!] macros. While tokens
//! can be constructed manually, these make this process much easier.
//!
//! genco only minimally deals with language-specific syntax, but primarily deals
//! with solving the following:
//!
//! * Generates and groups import statements.
//! * Quote (and escape) strings using [`<stmt>.quoted()`].
//! * Indents and spaces your code according to generic [whitespace rules] that
//!   can be tweaked on a per-language basis.
//!
//! ## Examples
//!
//! The following are language specific examples for genco using the [quote!]
//! macro.
//!
//! * [Rust Example]
//! * [Java Example]
//! * [C# Example]
//! * [Go Example]
//! * Dart Example (TODO)
//! * JavaScript Example (TODO)
//! * Python Example (TODO)
//!
//! You can run one of the examples above using:
//!
//! ```bash
//! cargo run --example go
//! ```
//!
//! The following is the included example Rust program.
//!
//! ```rust
//! use genco::prelude::*;
//! use rand::Rng;
//!
//! use std::fmt;
//!
//! fn main() -> fmt::Result {
//!     // Import the LittleEndian item, without referencing it through the last
//!     // module component it is part of.
//!     let little_endian = rust::imported("byteorder", "LittleEndian");
//!     let big_endian = rust::imported("byteorder", "BigEndian").prefixed();
//!
//!     // This is a trait, so only import it into the scope (unless we intent to
//!     // implement it).
//!     let write_bytes_ext = rust::imported("byteorder", "WriteBytesExt").alias("_");
//!     let read_bytes_ext = rust::imported("byteorder", "ReadBytesExt").alias("_");
//!
//!     let tokens = quote! {
//!         // Markup used for imports without an immediate use.
//!         #@(write_bytes_ext)
//!         #@(read_bytes_ext)
//!
//!         fn test() {
//!             let mut wtr = vec![];
//!             wtr.write_u16::<#little_endian>(517).unwrap();
//!             wtr.write_u16::<#big_endian>(768).unwrap();
//!         }
//!     };
//!
//!     // Simpler printing with default indentation:
//!     // println!("{}", tokens.to_file_string()?);
//!
//!     tokens.to_io_writer_with(
//!         std::io::stdout().lock(),
//!         rust::Config::default(),
//!         FormatterConfig::from_lang::<Rust>().with_indentation(2),
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Whitespace Rules
//!
//! The `quote!` macro has the following rules for dealing with indentation and
//! spacing.
//!
//! **Two tokens** that are separated, are spaced. Regardless of how many spaces
//! there are between them.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let tokens: rust::Tokens = quote! {
//!     fn     test()     {
//!         println!("Hello... ");
//!
//!         println!("World!");
//!     }
//! };
//!
//! assert_eq!(
//!     vec![
//!         "fn test() {",
//!         "    println!(\"Hello... \");",
//!         "",
//!         "    println!(\"World!\");",
//!         "}",
//!     ],
//!     tokens.to_file_vec().unwrap(),
//! )
//! ```
//!
//! **More that two line breaks** are collapsed.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let tokens: rust::Tokens = quote! {
//!     fn test() {
//!         println!("Hello... ");
//!
//!
//!
//!         println!("World!");
//!     }
//! };
//!
//! assert_eq!(
//!     vec![
//!         "fn test() {",
//!         "    println!(\"Hello... \");",
//!         "",
//!         "    println!(\"World!\");",
//!         "}",
//!     ],
//!     tokens.to_file_vec().unwrap(),
//! )
//! ```
//!
//! **Indentation** is determined on a row-by-row basis. If a column is further in
//! than the one on the preceeding row, it is indented **one level** deeper.
//!
//! If a column starts shallower than a previous row, it will be matched against
//! previously known indentation levels.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let tokens: rust::Tokens = quote! {
//!     fn test() {
//!             println!("Hello... ");
//!
//!             println!("World!");
//!     }
//! };
//!
//! assert_eq!(
//!     vec![
//!         "fn test() {",
//!         "    println!(\"Hello... \");",
//!         "",
//!         "    println!(\"World!\");",
//!         "}",
//!     ],
//!     tokens.to_file_vec().unwrap(),
//! )
//! ```
//!
//! A mismatched indentation would result in an error:
//!
//! ```rust,compile_fail
//! use genco::prelude::*;
//!
//! let tokens: rust::Tokens = quote! {
//!     fn test() {
//!             println!("Hello... ");
//!
//!         println!("World!");
//!     }
//! };
//! ```
//!
//! ```text
//! ---- src\lib.rs -  (line 150) stdout ----
//! error: expected 4 less spaces of indentation
//! --> src\lib.rs:157:9
//!    |
//! 10 |         println!("World!");
//!    |         ^^^^^^^
//! ```
//!
//! [reproto]: https://github.com/reproto/reproto
//! [whitespace rules]: https://github.com/udoprog/genco#whitespace-rules
//! [Rust Example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
//! [Java Example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
//! [C# Example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
//! [Go Example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
//! [quote!]: macro.quote.html
//! [quote_in!]: macro.quote_in.html
//! [`<stmt>.quoted()`]: crate::ext::QuotedExt::quoted

#![doc(html_root_url = "https://docs.rs/genco/0.5.0")]
#![deny(missing_docs)]
#![deny(intra_doc_link_resolution_failure)]

pub use genco_macros::{quote, quote_in};

#[macro_use]
mod macros;
pub mod ext;
mod format_tokens;
mod formatter;
mod item;
mod item_str;
mod lang;
/// Prelude to import.
pub mod prelude;
mod register_tokens;
mod tokens;

pub use self::ext::{Display, DisplayExt, Quoted, QuotedExt};
pub use self::format_tokens::FormatTokens;
pub use self::formatter::{Config as FormatterConfig, Formatter};
pub use self::item::Item;
pub use self::item_str::ItemStr;
pub use self::lang::*;
pub use self::register_tokens::RegisterTokens;
pub use self::tokens::Tokens;
