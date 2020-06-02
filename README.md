# GenCo

[![Build Status](https://travis-ci.org/udoprog/genco.svg?branch=master)](https://travis-ci.org/udoprog/genco)
[![crates.io](https://img.shields.io/crates/v/genco.svg?maxAge=2592000)](https://crates.io/crates/genco)

GenCo is an even simpler code generator for Rust, specifically written for use in [reproto][reproto].

It does not deal with language-specific syntax, instead it can do some of the basic necessities
through specialization.

* Handle imports, if needed.
* Quote strings according to language convention.
* Indents and spaces your code according to [simple rules](#indentation-rules).

[reproto]: https://github.com/reproto/reproto

## Examples

* [Rust Example](/examples/rust.rs)
* [Java Example](/examples/rust.rs)
* [C# Example](/examples/csharp.rs)

## Language Support

This section contains example code for some of the supported languages.

For more information, see [docs.rs/genco](https://docs.rs/genco).

#### Dart

Simple support for importing names.

```rust
#[macro_use]
extern crate genco;

fn main() {
    use genco::dart::imported;

    let m = imported("dart:math").alias("m");
    let sqrt = m.name("sqrt");

    let mut t = toks!();
    t.push("void main() {");
    t.nested({
        let mut body = toks!();
        body.push(toks!("print(", "The Square Root Is:".quoted(), " + ", sqrt, "(42));"));
        body
    });
    t.push("}");
}
```

## Indentation Rules

The `quote!` macro has the following rules for dealing with indentation and
spacing.

**Two tokens** that are separated, are spaced. Regardless of how many spaces
there are between them.

So:

```rust
quote!(fn   test());
```

Becomes:

```rust
fn test()
```

**More that two line breaks** are collapsed.

So:

```rust
quote! {
    fn test() {
        println!("Hello...");


        println!("... World!");
    }
}
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
quote! {
  fn test() {
      println!("Hello...");
      println!("... World!");
    }
}
```

Becomes:

```rust
fn test() {
    println!("Hello...");
    println!("... World!");
}
```