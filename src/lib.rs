//! genco is a code generator and quasi quoter for Rust, written for use in
//! [reproto].
//!
//! The workhorse of genco is the [quote!] and [quote_in!] macros. While tokens
//! can be constructed manually, these make this process much easier.
//!
//! genco only minimally deals with language-specific syntax, but primarily deals
//! with solving the following:
//!
//! * **Imports** — genco generates and groups import statements according to
//!   language convention.
//!
//! * **String Quoting** — strings can be quoted in a language specific way
//!   either by including them literally in the token stream by using
//!   `quote!("hello")` or `quote!(#_(hello))`. Or explicitly with the
//!   [quoted()] function.
//!
//! * **Structural Indentation** — genco's quasi quoting utilizes
//!   [whitespace detection] to structurally sort out spaces and indentation.
//!
//! * **Language Customization** — building support for an unsupported language
//!   is easy with the [impl_lang!] macro.
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
//! * [Dart Example]
//! * [JavaScript Example]
//! * [Python Example]
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
//! ```rust,no_run
//! use genco::prelude::*;
//! use genco::fmt;
//!
//! # fn main() -> fmt::Result {
//! let map = rust::import("std::collections", "HashMap");
//!
//! let tokens: rust::Tokens = quote! {
//!     fn main() {
//!         let mut m = #map::new();
//!         m.insert(1u32, 2u32);
//!     }
//! };
//!
//! let stdout = std::io::stdout();
//! let mut w = fmt::IoWriter::new(stdout.lock());
//!
//! let fmt = fmt::Config::from_lang::<Rust>()
//!     .with_indentation(fmt::Indentation::Space(2));
//! let config = rust::Config::default();
//!
//! tokens.format_file(&mut w.as_formatter(fmt), &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! This would produce:
//!
//! ```rust,no_run
//! use std::collections::HashMap;
//!
//! fn main() {
//!     let mut m = HashMap::new();
//!     m.insert(1u32, 2u32);
//! }
//! ```
//!
//! <br>
//!
//! [reproto]: https://github.com/reproto/reproto
//! [whitespace detection]: https://docs.rs/genco/0/genco/macro.quote.html#whitespace-detection
//! [Rust Example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
//! [Java Example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
//! [C# Example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
//! [Go Example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
//! [Dart Example]: https://github.com/udoprog/genco/blob/master/examples/dart.rs
//! [JavaScript Example]: https://github.com/udoprog/genco/blob/master/examples/js.rs
//! [Python Example]: https://github.com/udoprog/genco/blob/master/examples/python.rs
//! [quote!]: https://docs.rs/genco/0/genco/macro.quote.html
//! [quote_in!]: https://docs.rs/genco/0/genco/macro.quote_in.html
//! [impl_lang!]: https://docs.rs/genco/0/genco/fn.impl_lang.html
//! [quoted()]: https://docs.rs/genco/0/genco/tokens/fn.quoted.html

#![doc(html_root_url = "https://docs.rs/genco/0.10.1")]
#![deny(missing_docs)]
#![deny(intra_doc_link_resolution_failure)]

pub use genco_macros::{quote, quote_in};

#[macro_use]
mod macros;
pub mod fmt;
pub mod lang;
pub mod prelude;
pub mod tokens;

pub use self::tokens::Tokens;
