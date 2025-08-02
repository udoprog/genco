//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/genco-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/genco)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/genco.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/genco)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-genco-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/genco)
//!
//! A whitespace-aware quasiquoter for beautiful code generation.
//!
//! Central to genco are the [quote!] and [quote_in!] procedural macros which
//! ease the construction of [token streams].
//!
//! This project solves the following language-specific concerns:
//!
//! * **Imports** — Generates and groups [import statements] as they are used.
//!   So you only import what you use, with no redundancy. We also do our best
//!   to [solve namespace conflicts].
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
//! To support line changes during [whitespace detection], we depend on span
//! information which was made available in Rust `1.88`. Before that, we relied
//! on a nightly [`proc_macro_span` feature] to work.
//!
//! *Until this is stabilized* and you want fully functional whitespace
//! detection you must build and run projects using genco with a `nightly`
//! compiler. This is important for whitespace-sensitive languages like python.
//!
//! You can try the difference between:
//!
//! ```bash
//! cargo run --example rust
//! ```
//!
//! And:
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
//! * [🇨 <b>C</b>][c]<br>
//!   <small>[Example][c-example]</small>
//!
//! * [🐍 <b>Python</b>][python]<br>
//!   <small>[Example][python-example]</small><br>
//!   **Requires a `nightly` compiler**
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
//! let hash_map = rust::import("std::collections", "HashMap");
//!
//! let tokens: rust::Tokens = quote! {
//!     fn main() {
//!         let mut m = $hash_map::new();
//!         m.insert(1u32, 2u32);
//!     }
//! };
//!
//! println!("{}", tokens.to_file_string()?);
//! # Ok::<_, genco::fmt::Error>(())
//! ```
//!
//! This would produce:
//!
//! ```rust,no_test
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
//! [c-example]: https://github.com/udoprog/genco/blob/master/examples/c.rs
//! [c]: https://docs.rs/genco/latest/genco/lang/c/index.html
//! [c#-example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
//! [c#]: https://docs.rs/genco/latest/genco/lang/csharp/index.html
//! [dart-example]: https://github.com/udoprog/genco/blob/master/examples/dart.rs
//! [dart]: https://docs.rs/genco/latest/genco/lang/dart/index.html
//! [go-example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
//! [go]: https://docs.rs/genco/latest/genco/lang/go/index.html
//! [impl_lang!]: https://docs.rs/genco/latest/genco/macro.impl_lang.html
//! [import statements]: https://docs.rs/genco/latest/genco/macro.quote.html#imports
//! [indentation is meaningful]: https://docs.python.org/3/faq/design.html#why-does-python-use-indentation-for-grouping-of-statements
//! [interpolate]: https://docs.rs/genco/latest/genco/macro.quote.html#quoted-string-interpolation
//! [java-example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
//! [java]: https://docs.rs/genco/latest/genco/lang/java/index.html
//! [js-example]: https://github.com/udoprog/genco/blob/master/examples/js.rs
//! [js]: https://docs.rs/genco/latest/genco/lang/js/index.html
//! [Open an issue!]: https://github.com/udoprog/genco/issues/new
//! [python-example]: https://github.com/udoprog/genco/blob/master/examples/python.rs
//! [python]: https://docs.rs/genco/latest/genco/lang/python/index.html
//! [quote strings]: https://docs.rs/genco/latest/genco/macro.quote.html#string-quoting
//! [quote_in!]: https://docs.rs/genco/latest/genco/macro.quote_in.html
//! [quote!]: https://docs.rs/genco/latest/genco/macro.quote.html
//! [quoted()]: https://docs.rs/genco/latest/genco/tokens/fn.quoted.html
//! [rust-example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
//! [rust]: https://docs.rs/genco/latest/genco/lang/rust/index.html
//! [solve namespace conflicts]: https://docs.rs/genco/latest/genco/lang/csharp/fn.import.html
//! [token streams]: https://docs.rs/genco/latest/genco/tokens/struct.Tokens.html
//! [whitespace detection]: https://docs.rs/genco/latest/genco/macro.quote.html#whitespace-detection

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(clippy::needless_doctest_main)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "alloc"))]
compile_error!("genco: The `alloc` feature must be enabled");

/// Whitespace sensitive quasi-quoting.
///
/// This and the [quote_in!] macro is the thing that this library revolves
/// around.
///
/// It provides a flexible and intuitive mechanism for efficiently generating
/// beautiful code directly inside of Rust.
///
/// > Note that this macro **can only detect line changes** if it's built under
/// > a `nightly` compiler. See the [main genco documentation] for more
/// > information.
///
/// ```
/// use genco::prelude::*;
///
/// let hash_map = &dart::import("dart:collection", "HashMap");
///
/// let tokens: dart::Tokens = quote! {
///     print_greeting(String name) {
///         print($[str](Hello $(name)));
///     }
///
///     $hash_map<int, String> map() {
///         return new $hash_map<int, String>();
///     }
/// };
///
/// println!("{}", tokens.to_file_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Interpolation
///
/// Variables are interpolated using `$`, so to include the variable `test`, you
/// would write `$test`. Interpolated variables must implement [FormatInto].
/// Expressions can be interpolated with `$(<expr>)`.
///
/// > *Note:* The `$` punctuation itself can be escaped by repeating it twice.
/// > So `$$` would produce a single `$` token.
///
/// ```
/// use genco::prelude::*;
///
/// let hash_map = rust::import("std::collections", "HashMap");
///
/// let tokens: rust::Tokens = quote! {
///     struct Quoted {
///         field: $hash_map<u32, u32>,
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "use std::collections::HashMap;",
///         "",
///         "struct Quoted {",
///         "    field: HashMap<u32, u32>,",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// The following is an expression interpolated with `$(<expr>)`.
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: genco::Tokens = quote! {
///     hello $("world".to_uppercase())
/// };
///
/// assert_eq!("hello WORLD", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// Interpolations are evaluated in the same scope as the macro, so you can
/// freely make use of Rust operations like the try keyword (`?`) if
/// appropriate:
///
/// ```
/// use std::error::Error;
///
/// use genco::prelude::*;
///
/// fn age_fn(age: &str) -> Result<rust::Tokens, Box<dyn Error>> {
///     Ok(quote! {
///         fn age() {
///             println!("You are {} years old!", $(str::parse::<u32>(age)?));
///         }
///     })
/// }
/// ```
///
/// [FormatInto]: crate::tokens::FormatInto
/// [main genco documentation]: https://docs.rs/genco
///
/// <br>
///
/// # Escape Sequences
///
/// Because this macro is *whitespace sensitive*, it might sometimes be
/// necessary to provide hints of where whitespace should be inserted.
///
/// `quote!` trims any trailing and leading whitespace that it sees. So
/// `quote!(Hello )` is the same as `quote!(Hello)`. To include a space at the
/// end, we can use the special `$[' ']` escape sequence:
/// `quote!(Hello$[' '])`.
///
/// The available escape sequences are:
///
/// * `$[' ']` — Inserts spacing between tokens. This corresponds to the
///   [Tokens::space] function.
///
/// * `$['\r']` — Inserts a push operation. Push operations makes sure that
///   any following tokens are on their own dedicated line. This corresponds to
///   the [Tokens::push] function.
///
/// * `$['\n']` — Inserts a forced line. Line operations makes sure that any
///   following tokens have an empty line separating them. This corresponds to
///   the [Tokens::line] function.
///
/// ```
/// use genco::prelude::*;
///
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote!(foo$['\r']bar$['\n']baz$[' ']biz);
///
/// assert_eq!("foo\nbar\n\nbaz biz", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # String Quoting
///
/// Literal strings like `"hello"` are automatically quoted for the target
/// language according to its [Lang::write_quoted] implementation.
///
/// [Lang::write_quoted]: crate::lang::Lang::write_quoted
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: java::Tokens = quote! {
///     "hello world 😊"
///     $(quoted("hello world 😊"))
///     $("\"hello world 😊\"")
///     $[str](hello world $[const]("😊"))
/// };
///
/// assert_eq!(
///     vec![
///         "\"hello world \\ud83d\\ude0a\"",
///         "\"hello world \\ud83d\\ude0a\"",
///         "\"hello world 😊\"",
///         "\"hello world \\ud83d\\ude0a\"",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Efficient String Quoting
///
/// It's worth investigating the different forms of tokens produced by the
/// above example.
///
/// * The first one is a static *quoted string*.
/// * The second one is a boxed *quoted string*, who's content will be copied
///   and is stored on the heap.
/// * The third one is a static *literal* which bypasses language quoting
///   entirely.
/// * Finally the fourth one is an interpolated string. They are really neat,
///   and will be covered more in the next section. It's worth noting that
///   `$("😊")` is used, because 😊 is not a valid identifier in Rust. So this
///   example showcases how strings can be directly embedded in an
///   interpolation.
///
/// Here you can see the items produced by the macro.
///
/// ```
/// # use genco::prelude::*;
/// # let tokens: rust::Tokens = quote! {
/// #     "hello world 😊"
/// #     $(quoted("hello world 😊"))
/// #     $("\"hello world 😊\"")
/// #     $[str](hello world $[const]("😊"))
/// # };
/// use genco::tokens::{Item, ItemStr};
///
/// assert_eq!(
///     vec![
///         Item::OpenQuote(false),
///         Item::Literal(ItemStr::Static("hello world 😊")),
///         Item::CloseQuote,
///         Item::Push,
///         Item::OpenQuote(false),
///         Item::Literal(ItemStr::Box("hello world 😊".into())),
///         Item::CloseQuote,
///         Item::Push,
///         Item::Literal(ItemStr::Static("\"hello world 😊\"")),
///         Item::Push,
///         Item::OpenQuote(false),
///         Item::Literal(ItemStr::Static("hello world 😊")),
///         Item::CloseQuote
///     ],
///     tokens,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # Quoted String Interpolation
///
/// Some languages support interpolating values into strings.
///
/// Examples of these are:
///
/// * JavaScript - With [template literals] `` `Hello ${a}` `` (note the
///   backticks).
/// * Dart - With [interpolated strings] like `"Hello $a"` or `"Hello ${a +
///   b}"`.
///
/// The [quote!] macro supports this through `$[str](<content>)`. This will
/// produce literal strings with the appropriate language-specific quoting and
/// string interpolation formats used.
///
/// Components of the string are runtime evaluated with the typical variable
/// escape sequences `$ident`, `$(<expr>)`. In order to interpolate the string
/// at compile time we can instead make use of `$[const](<content>)` like you can see with the smile below:
///
/// ```
/// use genco::prelude::*;
///
/// let smile = "😊";
///
/// let t: js::Tokens = quote!($[str](Hello $[const](smile) $world));
/// assert_eq!("`Hello 😊 ${world}`", t.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// Interpolated values are specified with `$(<quoted>)`. And `$` itself is
/// escaped by repeating it twice through `$$`. The `<quoted>` section is
/// interpreted the same as in the [quote!] macro, but is whitespace sensitive.
/// This means that `$(foo)` is not the same as `$(foo )` since the latter will
/// have a space preserved at the end.
///
/// ```
/// use genco::prelude::*;
///
/// let smile = "😊";
///
/// let t: dart::Tokens = quote!($[str](Hello $[const](smile) $(world)));
/// assert_eq!("\"Hello 😊 $world\"", t.to_string()?);
///
/// let t: dart::Tokens = quote!($[str](Hello $[const](smile) $(a + b)));
/// assert_eq!("\"Hello 😊 ${a + b}\"", t.to_string()?);
///
/// let t: js::Tokens = quote!($[str](Hello $[const](smile) $(world)));
/// assert_eq!("`Hello 😊 ${world}`", t.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// [template literals]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
/// [interpolated strings]: https://medium.com/run-dart/dart-dartlang-introduction-string-interpolation-8ed99174119a
///
/// # Control Flow
///
/// [quote!] provides some limited mechanisms for control flow inside of the
/// macro for convenience. The supported mechanisms are:
///
/// * [Loops](#loops) - `$(for <bindings> in <expr> [join (<quoted>)] => <quoted>)`.
/// * [Conditionals](#conditionals) - `$(if <pattern> => <quoted>)`.
/// * [Match Statements](#match-statements) - `$(match <expr> { [<pattern> => <quoted>,]* })`.
///
/// <br>
///
/// # Loops
///
/// To repeat a pattern you can use `$(for <bindings> in <expr> { <quoted> })`,
/// where `<expr>` is an iterator.
///
/// It is also possible to use the more compact `$(for <bindings> in <expr> =>
/// <quoted>)` (note the arrow).
///
/// `<quoted>` will be treated as a quoted expression, so anything which works
/// during regular quoting will work here as well, with the addition that
/// anything defined in `<bindings>` will be made available to the statement.
///
/// ```
/// use genco::prelude::*;
///
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote! {
///     Your numbers are: $(for n in numbers => $n$[' '])
/// };
///
/// assert_eq!("Your numbers are: 3 4 5", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # Joining Loops
///
/// You can add `join (<quoted>)` to the end of a repetition.
///
/// The expression specified in `join (<quoted>)` is added _between_ each
/// element produced by the loop.
///
/// > *Note:* The argument to `join` is *whitespace sensitive*, so leading and
/// > trailing is preserved. `join (,)` and `join (, )` would therefore produce
/// > different results.
///
/// ```
/// use genco::prelude::*;
///
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote! {
///     Your numbers are: $(for n in numbers join (, ) => $n).
/// };
///
/// assert_eq!("Your numbers are: 3, 4, 5.", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # Conditionals
///
/// You can specify a conditional with `$(if <pattern> => <then>)` where
/// `<pattern>` is an pattern or expression evaluating to a `bool`, and `<then>`
/// is a quoted expressions.
///
/// It's also possible to specify a condition with an else branch, by using
/// `$(if <pattern> { <then> } else { <else> })`. `<else>` is also a quoted
/// expression.
///
/// ```
/// use genco::prelude::*;
///
/// fn greeting(hello: bool, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting: $(if hello {
///         Hello $name
///     } else {
///         Goodbye $name
///     }))
/// }
///
/// let tokens = greeting(true, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string()?);
///
/// let tokens = greeting(false, "John");
/// assert_eq!("Custom Greeting: Goodbye John", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// The `<else>` branch is optional, conditionals which do not have an else
/// branch and evaluated to `false` won't produce any tokens:
///
/// ```
/// use genco::prelude::*;
///
/// fn greeting(hello: bool, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting:$(if hello {
///         $[' ']Hello $name
///     }))
/// }
///
/// let tokens = greeting(true, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string()?);
///
/// let tokens = greeting(false, "John");
/// assert_eq!("Custom Greeting:", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # Match Statements
///
/// You can specify a match expression using `$(match <expr> { [<pattern> =>
/// <quoted>,]* }`, where `<expr>` is an evaluated expression that is match
/// against each subsequent `<pattern>`. If a pattern matches, the arm with the
/// matching `<quoted>` block is evaluated.
///
/// ```
/// use genco::prelude::*;
///
/// fn greeting(name: &str) -> Tokens<()> {
///     quote!(Hello $(match name {
///         "John" | "Jane" => $("Random Stranger"),
///         other => $other,
///     }))
/// }
///
/// let tokens = greeting("John");
/// assert_eq!("Hello Random Stranger", tokens.to_string()?);
///
/// let tokens = greeting("Mio");
/// assert_eq!("Hello Mio", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// If a match arm contains parenthesis (`=> (<quoted>)`), the expansion will be
/// *whitespace sensitive*. Allowing leading and trailing whitespace to be
/// preserved:
///
/// ```
/// use genco::prelude::*;
///
/// fn greeting(name: &str) -> Tokens<()> {
///     quote!(Hello$(match name {
///         "John" | "Jane" => ( $("Random Stranger")),
///         other => ( $other),
///     }))
/// }
///
/// let tokens = greeting("John");
/// assert_eq!("Hello Random Stranger", tokens.to_string()?);
///
/// let tokens = greeting("Mio");
/// assert_eq!("Hello Mio", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// The following is an example with more complex matching:
///
/// ```
/// use genco::prelude::*;
///
/// enum Greeting {
///     Named(&'static str),
///     Unknown,
/// }
///
/// fn greeting(name: Greeting) -> Tokens<()> {
///     quote!(Hello $(match name {
///         Greeting::Named("John") | Greeting::Named("Jane") => $("Random Stranger"),
///         Greeting::Named(other) => $other,
///         Greeting::Unknown => $("Unknown Person"),
///     }))
/// }
///
/// let tokens = greeting(Greeting::Named("John"));
/// assert_eq!("Hello Random Stranger", tokens.to_string()?);
///
/// let tokens = greeting(Greeting::Unknown);
/// assert_eq!("Hello Unknown Person", tokens.to_string()?);
///
/// let tokens = greeting(Greeting::Named("Mio"));
/// assert_eq!("Hello Mio", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # Variable assignment
///
/// You can use `$(let <binding> = <expr>)` to define variables with their value.
/// This is useful within loops to compute values from iterator items.
///
/// ```
/// use genco::prelude::*;
///
/// let names = ["A.B", "C.D"];
///
/// let tokens: Tokens<()> = quote! {
///     $(for name in names =>
///         $(let (first, second) = name.split_once('.').unwrap())
///         $first and $second.
///     )
/// };
/// assert_eq!("A and B.\nC and D.", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// Variables can also be mutable:
///
/// ```
/// use genco::prelude::*;
/// let path = "A.B.C.D";
///
/// let tokens: Tokens<()> = quote! {
///     $(let mut items = path.split('.'))
///     $(if let Some(first) = items.next() =>
///         First is $first
///     )
///     $(if let Some(second) = items.next() =>
///         Second is $second
///     )
/// };
///
/// assert_eq!("First is A\nSecond is B", tokens.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// # Scopes
///
/// You can use `$(ref <binding> { <expr> })` to gain access to the current
/// token stream. This is an alternative to existing control flow operators if
/// you want to run some custom code during evaluation which is otherwise not
/// supported. This is called a *scope*.
///
/// For a more compact variant you can omit the braces with `$(ref <binding> =>
/// <expr>)`.
///
/// ```
/// use genco::prelude::*;
///
/// fn quote_greeting(surname: &str, lastname: Option<&str>) -> rust::Tokens {
///     quote! {
///         Hello $surname$(ref toks {
///             if let Some(lastname) = lastname {
///                 toks.space();
///                 toks.append(lastname);
///             }
///         })
///     }
/// }
///
/// assert_eq!("Hello John", quote_greeting("John", None).to_string()?);
/// assert_eq!("Hello John Doe", quote_greeting("John", Some("Doe")).to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// ## Whitespace Detection
///
/// The [quote!] macro has the following rules for dealing with indentation and
/// spacing.
///
/// **Spaces** — Two tokens that are separated are spaced. Regardless of how
/// many spaces there are between them. This can be controlled manually by
/// inserting the [`$[' ']`][escape] escape sequence in the token stream.
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn     test()     {
///         println!("Hello... ");
///
///         println!("World!");
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "fn test() {",
///         "    println!(\"Hello... \");",
///         "",
///         "    println!(\"World!\");",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// **Line breaking** — Line breaks are detected by leaving two empty lines
/// between two tokens. This can be controlled manually by inserting the
/// [`$['\n']`][escape] escape in the token stream.
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn test() {
///         println!("Hello... ");
///
///
///
///         println!("World!");
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "fn test() {",
///         "    println!(\"Hello... \");",
///         "",
///         "    println!(\"World!\");",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// <br>
///
/// **Indentation** — Indentation is determined on a row-by-row basis. If a
/// column is further in than the one on the preceeding row, it is indented *one
/// level* deeper.
///
/// If a column starts shallower than a preceeding, non-whitespace only row, it
/// will be matched against previously known indentation levels. Failure to
/// match a previously known level is an error.
///
/// All indentations inserted during the macro will be unrolled at the end of
/// it. So any trailing indentations will be matched by unindentations.
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn test() {
///             println!("Hello... ");
///
///             println!("World!");
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "fn test() {",
///         "    println!(\"Hello... \");",
///         "",
///         "    println!(\"World!\");",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// Example showcasing an indentation mismatch:
///
/// ```,compile_fail
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn test() {
///             println!("Hello... ");
///
///         println!("World!");
///     }
/// };
/// ```
///
/// ```text
/// ---- src\lib.rs -  (line 150) stdout ----
/// error: expected 4 less spaces of indentation
/// --> src\lib.rs:157:9
///    |
/// 10 |         println!("World!");
///    |         ^^^^^^^
/// ```
///
/// [escape]: #escape-sequences
pub use genco_macros::quote;

/// Convenience macro for constructing a [FormatInto] implementation in-place.
///
/// Constructing [FormatInto] implementation instead of short lived [token
/// streams] can be more beneficial for memory use and performance.
///
/// [FormatInto]: crate::tokens::FormatInto
/// [token streams]: Tokens
///
/// # Comparison
///
/// In the below example, `f1` and `f2` are equivalent. In here [quote_fn!]
/// simply makes it easier to build.
///
/// ```
/// use genco::prelude::*;
/// use genco::tokens::from_fn;
///
/// let f1 = from_fn(move |t| {
///     quote_in!{ *t =>
///         println!("Hello World");
///     }
/// });
///
/// let f2 = quote_fn!{
///     println!("Hello World");
/// };
///
/// let tokens: rust::Tokens = quote!{
///     $f1
///     $f2
/// };
///
/// assert_eq!{
///     vec![
///         "println!(\"Hello World\");",
///         "println!(\"Hello World\");",
///     ],
///     tokens.to_file_vec()?,
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Examples which borrow
///
/// ```
/// use genco::prelude::*;
///
/// fn greeting(name: &str) -> impl FormatInto<Rust> + '_ {
///     quote_fn! {
///         println!($[str](Hello $[const](name)))
///     }
/// }
///
/// fn advanced_greeting<'a>(first: &'a str, last: &'a str) -> impl FormatInto<Rust> + 'a {
///     quote_fn! {
///         println!($[str](Hello $[const](first) $[const](last)))
///     }
/// }
///
/// let tokens = quote! {
///     $(greeting("Mio"));
///     $(advanced_greeting("Jane", "Doe"));
/// };
///
/// assert_eq!{
///     vec![
///         "println!(\"Hello Mio\");",
///         "println!(\"Hello Jane Doe\");",
///     ],
///     tokens.to_file_vec()?
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub use genco_macros::quote_fn;

/// Behaves the same as [quote!] while quoting into an existing token stream
/// with `<target> => <quoted>`.
///
/// This macro takes a destination stream followed by an `=>` and the tokens to
/// extend that stream with.
///
/// Note that the `<target>` arguments must be borrowable. So a mutable
/// reference like `&mut rust::Tokens` will have to be dereferenced when used
/// with this macro.
///
/// ```
/// # use genco::prelude::*;
///
/// # fn generate() -> rust::Tokens {
/// let mut tokens = rust::Tokens::new();
/// quote_in!(tokens => hello world);
/// # tokens
/// # }
///
/// fn generate_into(tokens: &mut rust::Tokens) {
///     quote_in! { *tokens =>
///         hello...
///         world!
///     };
/// }
/// ```
///
/// # Example
///
/// ```
/// use genco::prelude::*;
///
/// let mut tokens = rust::Tokens::new();
///
/// quote_in! { tokens =>
///     fn foo() -> u32 {
///         42
///     }
/// }
/// ```
///
/// # Use with scopes
///
/// [quote_in!] can be used inside of a [quote!] through [a scope].
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn foo(v: bool) -> u32 {
///         $(ref out {
///             quote_in! { *out =>
///                 if v {
///                     1
///                 } else {
///                     0
///                 }
///             }
///         })
///     }
/// };
/// ```
///
/// [a scope]: quote#scopes
pub use genco_macros::quote_in;

#[macro_use]
mod macros;
pub mod fmt;
pub mod lang;
pub mod prelude;
pub mod tokens;

pub use self::tokens::Tokens;
