[![Build Status](https://github.com/udoprog/genco/workflows/Rust/badge.svg)](https://github.com/udoprog/genco/actions)
[![crates.io](https://img.shields.io/crates/v/genco.svg)](https://crates.io/crates/genco)
[![docs.rs](https://docs.rs/genco/badge.svg)](https://docs.rs/genco)

# genco

genco is a language neutral code generator and quasi quoter.

Central to genco is the [quote!] and [quote_in!] macros. While token
streams can be constructed manually, these makes the process much
more intuitive.

genco solves the following, language-specific concerns.

* **Imports** — genco generates and groups [import statements] as they are
  used. No redundancy, no mess. Only exactly what you need. Worried about
  conflicts? We've got you covered.

* **String Quoting** — we figure out how to correctly [quote strings]
  for your language. We even know how to generate code to [interpolate]
  values *into* the quoted string ([like `"Hello $name"` in Dart](https://dart.dev/guides/language/language-tour#strings)).

* **Structural Indentation** — our quasi quoter performs efficient
  [whitespace detection] to structurally sort out spaces and indentation.
  Allowing us to generate beautiful, readable code with minimal effort.

* **Language Customization** — building support for new languages is a
  piece of cake with the help of the batteries included [impl_lang!] macro.

<br>

In order to do whitespace detection, we depend on the
[`proc_macro_span` feature] to access information on spans.
Until this is stable, you must build and run projects using genco with the
`nightly` compiler.

```bash
cargo +nightly run --example rust
```

[`proc_macro_span` feature]: https://github.com/rust-lang/rust/issues/54725

<br>

### Examples

The following are language specific examples for genco using the [quote!]
macro.

* [Rust Example]
* [Java Example]
* [C# Example]
* [Go Example]
* [Dart Example]
* [JavaScript Example]
* [Python Example]

You can run one of the examples above using:

```bash
cargo run --example go
```

<br>

### Rust Example

The following is a simple program producing Rust code to stdout with custom
configuration:

```rust
use genco::prelude::*;
use genco::fmt;

let map = rust::import("std::collections", "HashMap");

let tokens: rust::Tokens = quote! {
    fn main() {
        let mut m = #map::new();
        m.insert(1u32, 2u32);
    }
};

let stdout = std::io::stdout();
let mut w = fmt::IoWriter::new(stdout.lock());

let fmt = fmt::Config::from_lang::<Rust>()
    .with_indentation(fmt::Indentation::Space(2));
let config = rust::Config::default();

tokens.format_file(&mut w.as_formatter(fmt), &config)?;
```

This would produce:

```rust
use std::collections::HashMap;

fn main() {
    let mut m = HashMap::new();
    m.insert(1u32, 2u32);
}
```

<br>

[import statements]: https://docs.rs/genco/0/genco/macro.quote.html#imports
[quote strings]: https://docs.rs/genco/0/genco/macro.quote.html#string-quoting
[interpolate]: https://docs.rs/genco/0/genco/macro.quote.html#quoted-string-interpolation
[reproto]: https://github.com/reproto/reproto
[whitespace detection]: https://docs.rs/genco/0/genco/macro.quote.html#whitespace-detection
[Rust Example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
[Java Example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
[C# Example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
[Go Example]: https://github.com/udoprog/genco/blob/master/examples/go.rs
[Dart Example]: https://github.com/udoprog/genco/blob/master/examples/dart.rs
[JavaScript Example]: https://github.com/udoprog/genco/blob/master/examples/js.rs
[Python Example]: https://github.com/udoprog/genco/blob/master/examples/python.rs
[quote!]: https://docs.rs/genco/0/genco/macro.quote.html
[quote_in!]: https://docs.rs/genco/0/genco/macro.quote_in.html
[impl_lang!]: https://docs.rs/genco/0/genco/macro.impl_lang.html
[quoted()]: https://docs.rs/genco/0/genco/tokens/fn.quoted.html
