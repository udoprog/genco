//! genco is a language neutral quasi quoter.
//!
//! Central to genco are the [quote!] and [quote_in!] procedural macros which
//! ease the construction of [token streams].
//!
//! This projects solves the following, language-specific concerns:
//!
//! * **Imports** — genco generates and groups [import statements] as they are
//!   used. What you use is what you get, with no redundancy or mess. We also
//!   do our best to solve namespacing conflicts transparently for you.
//!
//! * **String Quoting** — genco knows how to [quote strings]. And can even
//!   [interpolate] values *into* the quoted string if it's supported by the
//!   language ([like `"Hello $name"` in Dart](https://dart.dev/guides/language/language-tour#strings)).
//!
//! * **Structural Indentation** — The quoter relies on intuitive
//!   [whitespace detection] to structurally sort out spacings and indentation.
//!   Allowing genco to generate beautiful readable code with minimal effort.
//!
//! * **Language Customization** — Building support for new languages is a
//!   piece of cake with the help of the [impl_lang!] macro.
//!
//! <br>
//!
//! To do whitespace detection, we depend on the [`proc_macro_span` feature].
//!
//! Until this is stabilized, you must build and run projects using genco with
//! the `nightly` compiler.
//!
//! ```bash
//! cargo +nightly run --example rust
//! ```
//!
//! [`proc_macro_span` feature]: https://github.com/rust-lang/rust/issues/54725
//!
//! <br>
//!
//! ## Supported Languages
//!
//! The following are languages which have built-in support in genco.
//! Is your favorite language missing? [Open an issue!]
//!
//! * [Rust Example]
//! * [Java Example]
//! * [C# Example]
//! * [Go Example]
//! * [Dart Example]
//! * [JavaScript Example]
//! * [Python Example]
//!
//! You can run one of the examples using:
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
//! [token streams]: https://docs.rs/genco/0/genco/tokens/struct.Tokens.html
//! [import statements]: https://docs.rs/genco/0/genco/macro.quote.html#imports
//! [quote strings]: https://docs.rs/genco/0/genco/macro.quote.html#string-quoting
//! [interpolate]: https://docs.rs/genco/0/genco/macro.quote.html#quoted-string-interpolation
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
//! [impl_lang!]: https://docs.rs/genco/0/genco/macro.impl_lang.html
//! [quoted()]: https://docs.rs/genco/0/genco/tokens/fn.quoted.html
//! [Open an issue!]: https://github.com/udoprog/genco/issues/new

#![doc(html_root_url = "https://docs.rs/genco/0.10.10")]
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
