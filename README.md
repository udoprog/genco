# GenCo

[![Build Status](https://github.com/udoprog/genco/workflows/Rust/badge.svg)](https://github.com/udoprog/genco/actions)
[![crates.io](https://img.shields.io/crates/v/genco.svg?maxAge=2592000)](https://crates.io/crates/genco)

GenCo is an even simpler code generator for Rust, written for use in [reproto].

The workhorse of GenCo is the [`quote!`] macro. While tokens can be constructed
manually, [`quote!`] makes this process much easier.

GenCo only minimally deals with language-specific syntax, but primarily deals
with solving the following:

* Generates and groups import statements.
* Quote (and escape) strings using [`<stmt>.quoted()`].
* Indents and spaces your code according to generic [indentation rules] that can
  be tweaked on a per-language basis.

## Examples

The following are language specific examples for GenCo using the [`quote!`]
macro.

* [Rust Example]
* [Java Example]
* [C# Example]
* Dart Example (TODO)
* Go Example (TODO)
* JavaScript Example (TODO)
* Python Example (TODO)

The following is a simple example showcasing code generation for Rust.

```rust
#![feature(proc_macro_hygiene)]

use genco::rust::{imported, Config};
use genco::{quote, FormatterConfig, Rust};

use std::fmt;

fn main() -> fmt::Result {
    // Import the LittleEndian item, without referencing it through the last
    // module component it is part of.
    let little_endian = imported("byteorder", "LittleEndian").qualified();
    let big_endian = imported("byteorder", "BigEndian");

    // This is a trait, so only import it into the scope (unless we intent to
    // implement it).
    let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");

    let tokens = quote! {
        @write_bytes_ext
        fn test() {
            let mut wtr = vec![];
            wtr.write_u16::<#little_endian>(517).unwrap();
            wtr.write_u16::<#big_endian>(768).unwrap();
            assert_eq!(wtr, vec![5, 2, 3, 0]);
        }
    };

    // Simpler printing with default indentation:
    // println!("{}", tokens.to_file_string()?);

    tokens.to_io_writer_with(
        std::io::stdout().lock(),
        Config::default(),
        FormatterConfig::from_lang::<Rust>().with_indentation(2),
    )?;

    Ok(())
}
```

## Indentation Rules

The `quote!` macro has the following rules for dealing with indentation and
spacing.

**Two tokens** that are separated, are spaced. Regardless of how many spaces
there are between them.

So:

```rust
#![feature(proc_macro_hygiene)]

let _: genco::Tokens<genco::Rust> = genco::quote!(fn   test() {});
```

Becomes:

```rust
fn test() {}
```

**More that two line breaks** are collapsed.

So:

```rust
#![feature(proc_macro_hygiene)]

let _: genco::Tokens<genco::Rust> = genco::quote! {
    fn test() {
        println!("Hello...");


        println!("... World!");
    }
};
```

Becomes:

```rust
fn test() {
    println!("Hello...");

    println!("... World!");
}
```

**Indentation** is determined on a row-by-row basis. If a column is further in
than the one on the preceeding row, it is indented **one level** deeper.

Like wise if a column starts before the previous rows column, it is indended one
level shallower.

So:

```rust
#![feature(proc_macro_hygiene)]

let _: genco::Tokens<genco::Rust> = genco::quote! {
  fn test() {
      println!("Hello...");
      println!("... World!");
    }
};
```

Becomes:

```rust
fn test() {
    println!("Hello...");
    println!("... World!");
}
```

[reproto]: https://github.com/reproto/reproto
[indentation rules]: https://github.com/udoprog/genco#indentation-rules
[Rust Example]: https://github.com/udoprog/genco/blob/master/examples/rust.rs
[Java Example]: https://github.com/udoprog/genco/blob/master/examples/java.rs
[C# Example]: https://github.com/udoprog/genco/blob/master/examples/csharp.rs
[`quote!`]: https://github.com/udoprog/genco/blob/master/tests/test_quote.rs
[`<stmt>.quoted()`]: https://docs.rs/genco/latest/genco/trait.Quoted.html