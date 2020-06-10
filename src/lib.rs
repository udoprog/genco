//! ## genco
//!
//! genco is a simple code generator and quasi quoter for Rust, written for use
//! in [reproto].
//!
//! The workhorse of genco is the [quote!] and [quote_in!] macros. While tokens
//! can be constructed manually, these make this process much easier.
//!
//! genco only minimally deals with language-specific syntax, but primarily deals
//! with solving the following:
//!
//! * **Imports** — genco generates and groups import statements according to
//!   conventions for the language being generated for.
//!
//! * **String Quoting** — Strings can be quoted using the [`<stmt>.quoted()`]
//!   trait function.
//!
//! * **Structural Indentation** — genco's quasi quoting utilizes
//!   [whitespace detection] to structurally sort out spaces and indentation.
//!
//! <br>
//!
//! We depend on `proc_macro_hygiene` stabilizations. Until then, you must build
//! and run with the `nightly` branch.
//!
//! ```bash
//! cargo +nightly run --example rust
//! ```
//!
//! <br>
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
//! <br>
//!
//! ## Rust Example
//!
//! The following is a simple program producing Rust code to stdout with custom
//! configuration:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! use std::fmt;
//!
//! fn main() -> fmt::Result {
//!     let little_endian = rust::imported("byteorder", "LittleEndian");
//!     let big_endian = rust::imported("byteorder", "BigEndian").prefixed();
//!
//!     let write_bytes_ext = rust::imported("byteorder", "WriteBytesExt").alias("_");
//!
//!     let tokens = quote! {
//!         #@(write_bytes_ext)
//!
//!         fn test() {
//!             let mut wtr = vec![];
//!             wtr.write_u16::<#little_endian>(517).unwrap();
//!             wtr.write_u16::<#big_endian>(768).unwrap();
//!         }
//!     };
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
//! This would produce:
//!
//! ```rust,ignore
//! use byteorder::{self, LittleEndian, ReadBytesExt as _, WriteBytesExt as _};
//!
//! fn test() {
//!     let mut wtr = vec![];
//!     wtr.write_u16::<LittleEndian>(517).unwrap();
//!     wtr.write_u16::<byteorder::BigEndian>(768).unwrap();
//! }
//! ```
//!
//! <br>
//!
//! [reproto]: https://github.com/reproto/reproto
//! [whitespace detection]: https://github.com/udoprog/genco#whitespace-detection
//! [Rust Example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
//! [Java Example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
//! [C# Example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
//! [Go Example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
//! [quote!]: https://docs.rs/genco/0/genco/macro.quote.html
//! [quote_in!]: https://docs.rs/genco/0/genco/macro.quote_in.html
//! [`<stmt>.quoted()`]: https://docs.rs/genco/0/genco/trait.QuotedExt.html

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
