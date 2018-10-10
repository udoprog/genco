# GenCo

[![Build Status](https://travis-ci.org/udoprog/genco.svg?branch=master)](https://travis-ci.org/udoprog/genco)
[![crates.io](https://img.shields.io/crates/v/genco.svg?maxAge=2592000)](https://crates.io/crates/genco)

GenCo is an even simpler code generator for Rust, specifically written for use in [reproto][reproto].

It does not deal with language-specific syntax, instead it can do some of the basic necessities
through specialization.

* Handle imports, if needed.
* Quote strings according to language convention.

[reproto]: https://github.com/reproto/reproto

## Examples

This is an example building some JavaScript:

```rust
#[macro_use]
extern crate genco;

use genco::Quoted;

fn main() {
    let mut file: Tokens<JavaScript> = Tokens::new();

    file.push("function foo(v) {");
    file.nested(toks!("return v + ", ", World".quoted(), ";"));
    file.push("}");

    file.push(toks!("foo(", "Hello".quoted(), ");"));

    println!("{}", file.to_string().unwrap());
}
```

Running this example would print:

```js
function foo(v) {
  return v + ", World";
}
foo("Hello");
```

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
