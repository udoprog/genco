//! A whitespace-aware quasiquoter for beautiful code generation.
//!
//! Central to genco are the [quote!] and [quote_in!] procedural macros which
//! ease the construction of [token streams].
//!
//! This project solves the following language-specific concerns:
//!
//! * **Imports** — Generates and groups [import statements] as they are used.
//!   What you use is what you get with no redundancy and consistency. We also
//!   do our best to [solve namespace conflicts] for you.
//!
//! * **String Quoting** — genco knows how to [quote strings]. And can even
//!   [interpolate] values *into* the quoted string if it's supported by the
//!   language.
//!
//! * **Structural Indentation** — The quoter relies on intuitive
//!   [whitespace detection] to structurally sort out spacings and indentation.
//!   Allowing genco to generate beautiful readable code with minimal effort.
//!   This is also a requirement for generating correctly behaving code in
//!   languages like Python where [indentation is meaningful].
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
//!
//! * [🦀 <b>Rust</b>][rust]<br>
//!   <small>[Example][rust-example]</small>
//!
//! * [☕ <b>Java</b>][java]<br>
//!   <small>[Example][java-example]</small>
//!
//! * [🎼 <b>C#</b>][c#]<br>
//!   <small>[Example][c#-example]</small>
//!
//! * [🐿️ <b>Go</b>][go]<br>
//!   <small>[Example][go-example]</small>
//!
//! * [🎯 <b>Dart</b>][dart]<br>
//!   <small>[Example][dart-example]</small>
//!
//! * [🌐 <b>JavaScript</b>][js]<br>
//!   <small>[Example][js-example]</small>
//!
//! * [🐍 <b>Python</b>][python]<br>
//!   <small>[Example][python-example]</small>
//!
//! <small>Is your favorite language missing? <b>[Open an issue!]</b></small>
//!
//! You can run one of the examples by:
//!
//! ```bash
//! cargo +nightly run --example rust
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
//!
//! # fn main() -> genco::fmt::Result {
//! let hash_map = rust::import("std::collections", "HashMap");
//!
//! let tokens: rust::Tokens = quote! {
//!     fn main() {
//!         let mut m = #hash_map::new();
//!         m.insert(1u32, 2u32);
//!     }
//! };
//!
//! println!("{}", tokens.to_file_string()?);
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
//! [rust]: https://docs.rs/genco/0/genco/lang/rust/index.html
//! [rust-example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
//! [java]: https://docs.rs/genco/0/genco/lang/java/index.html
//! [java-example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
//! [c#]: https://docs.rs/genco/0/genco/lang/csharp/index.html
//! [c#-example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
//! [go]: https://docs.rs/genco/0/genco/lang/go/index.html
//! [go-example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
//! [dart]: https://docs.rs/genco/0/genco/lang/dart/index.html
//! [dart-example]: https://github.com/udoprog/genco/blob/master/examples/dart.rs
//! [js]: https://docs.rs/genco/0/genco/lang/js/index.html
//! [js-example]: https://github.com/udoprog/genco/blob/master/examples/js.rs
//! [python]: https://docs.rs/genco/0/genco/lang/python/index.html
//! [python-example]: https://github.com/udoprog/genco/blob/master/examples/python.rs
//! [solve namespace conflicts]: file:///home/udoprog/repo/genco/target/doc/genco/lang/csharp/fn.import.html
//! [indentation is meaningful]: https://docs.python.org/3/faq/design.html#why-does-python-use-indentation-for-grouping-of-statements
//! [token streams]: https://docs.rs/genco/0/genco/tokens/struct.Tokens.html
//! [import statements]: https://docs.rs/genco/0/genco/macro.quote.html#imports
//! [quote strings]: https://docs.rs/genco/0/genco/macro.quote.html#string-quoting
//! [interpolate]: https://docs.rs/genco/0/genco/macro.quote.html#quoted-string-interpolation
//! [whitespace detection]: https://docs.rs/genco/0/genco/macro.quote.html#whitespace-detection
//! [quote!]: https://docs.rs/genco/0/genco/macro.quote.html
//! [quote_in!]: https://docs.rs/genco/0/genco/macro.quote_in.html
//! [impl_lang!]: https://docs.rs/genco/0/genco/macro.impl_lang.html
//! [quoted()]: https://docs.rs/genco/0/genco/tokens/fn.quoted.html
//! [Open an issue!]: https://github.com/udoprog/genco/issues/new

#![doc(html_root_url = "https://docs.rs/genco/0.13.0")]
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
