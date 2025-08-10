# genco

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/genco-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/genco)
[<img alt="crates.io" src="https://img.shields.io/crates/v/genco.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/genco)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-genco-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/genco)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/genco/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/genco/actions?query=branch%3Amain)

A whitespace-aware quasiquoter for beautiful code generation.

Central to genco are the [quote!] and [quote_in!] procedural macros which
ease the construction of [token streams].

This project solves the following language-specific concerns:

* **Imports** — Generates and groups [import statements] as they are used.
  So you only import what you use, with no redundancy. We also do our best
  to [solve namespace conflicts].

* **String Quoting** — genco knows how to [quote strings]. And can even
  [interpolate] values *into* the quoted string if it's supported by the
  language.

* **Structural Indentation** — The quoter relies on intuitive
  [whitespace detection] to structurally sort out spacings and indentation.
  Allowing genco to generate beautiful readable code with minimal effort.
  This is also a requirement for generating correctly behaving code in
  languages like Python where [indentation is meaningful].

* **Language Customization** — Building support for new languages is a
  piece of cake with the help of the [impl_lang!] macro.

<br>

To support line changes during [whitespace detection], we depend on span
information which was made available in Rust `1.88`. Before that, we relied
on a nightly [`proc_macro_span` feature] to work.

*Until this is stabilized* and you want fully functional whitespace
detection you must build and run projects using genco with a `nightly`
compiler. This is important for whitespace-sensitive languages like python.

You can try the difference between:

```bash
cargo run --example rust
```

And:

```bash
cargo +nightly run --example rust
```

[`proc_macro_span` feature]: https://github.com/rust-lang/rust/issues/54725

<br>

## Supported Languages

The following are languages which have built-in support in genco.

* [🦀 <b>Rust</b>][rust]<br>
  <small>[Example][rust-example]</small>

* [☕ <b>Java</b>][java]<br>
  <small>[Example][java-example]</small>

* [🧬 <b>Kotlin</b>][kotlin]<br>
  <small>[Example][kotlin-example]</small>

* [🎼 <b>C#</b>][c#]<br>
  <small>[Example][c#-example]</small>

* [🐿️ <b>Go</b>][go]<br>
  <small>[Example][go-example]</small>

* [🎯 <b>Dart</b>][dart]<br>
  <small>[Example][dart-example]</small>

* [🌐 <b>JavaScript</b>][js]<br>
  <small>[Example][js-example]</small>

* [🇨 <b>C</b>][c]<br>
  <small>[Example][c-example]</small>

* [🐍 <b>Python</b>][python]<br>
  <small>[Example][python-example]</small><br>
  **Requires a `nightly` compiler**

<small>Is your favorite language missing? <b>[Open an issue!]</b></small>

You can run one of the examples by:

```bash
cargo +nightly run --example rust
```

<br>

## Rust Example

The following is a simple program producing Rust code to stdout with custom
configuration:

```rust
use genco::prelude::*;

let hash_map = rust::import("std::collections", "HashMap");

let tokens: rust::Tokens = quote! {
    fn main() {
        let mut m = $hash_map::new();
        m.insert(1u32, 2u32);
    }
};

println!("{}", tokens.to_file_string()?);
```

This would produce:

```rust,no_test
use std::collections::HashMap;

fn main() {
    let mut m = HashMap::new();
    m.insert(1u32, 2u32);
}
```

<br>

[c-example]: https://github.com/udoprog/genco/blob/master/examples/c.rs
[c]: https://docs.rs/genco/latest/genco/lang/c/index.html
[c#-example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
[c#]: https://docs.rs/genco/latest/genco/lang/csharp/index.html
[dart-example]: https://github.com/udoprog/genco/blob/master/examples/dart.rs
[dart]: https://docs.rs/genco/latest/genco/lang/dart/index.html
[go-example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
[go]: https://docs.rs/genco/latest/genco/lang/go/index.html
[impl_lang!]: https://docs.rs/genco/latest/genco/macro.impl_lang.html
[import statements]: https://docs.rs/genco/latest/genco/macro.quote.html#imports
[indentation is meaningful]: https://docs.python.org/3/faq/design.html#why-does-python-use-indentation-for-grouping-of-statements
[interpolate]: https://docs.rs/genco/latest/genco/macro.quote.html#quoted-string-interpolation
[java-example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
[java]: https://docs.rs/genco/latest/genco/lang/java/index.html
[kotlin-example]: https://github.com/udoprog/genco/blob/master/examples/kotlin.rs
[kotlin]: https://docs.rs/genco/latest/genco/lang/kotlin/index.html
[js-example]: https://github.com/udoprog/genco/blob/master/examples/js.rs
[js]: https://docs.rs/genco/latest/genco/lang/js/index.html
[Open an issue!]: https://github.com/udoprog/genco/issues/new
[python-example]: https://github.com/udoprog/genco/blob/master/examples/python.rs
[python]: https://docs.rs/genco/latest/genco/lang/python/index.html
[quote strings]: https://docs.rs/genco/latest/genco/macro.quote.html#string-quoting
[quote_in!]: https://docs.rs/genco/latest/genco/macro.quote_in.html
[quote!]: https://docs.rs/genco/latest/genco/macro.quote.html
[quoted()]: https://docs.rs/genco/latest/genco/tokens/fn.quoted.html
[rust-example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
[rust]: https://docs.rs/genco/latest/genco/lang/rust/index.html
[solve namespace conflicts]: https://docs.rs/genco/latest/genco/lang/csharp/fn.import.html
[token streams]: https://docs.rs/genco/latest/genco/tokens/struct.Tokens.html
[whitespace detection]: https://docs.rs/genco/latest/genco/macro.quote.html#whitespace-detection
