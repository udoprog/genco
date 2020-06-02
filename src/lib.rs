//! ## Simple and flexible code generator (genco)
#![deny(missing_docs)]
#![feature(proc_macro_hygiene)]
#![allow(deprecated)]

pub use genco_derive::quote;

#[macro_use]
mod macros;
mod con_;
mod config;
mod cons;
pub mod csharp;
mod custom;
pub mod dart;
mod element;
mod erased_element;
mod formatter;
pub mod go;
mod into_tokens;
pub mod java;
pub mod js;
pub mod python;
mod quoted;
pub mod rust;
pub mod swift;
mod tokens;
mod write_tokens;

pub use self::config::Config;
pub use self::cons::Cons;
pub use self::csharp::Csharp;
pub use self::custom::Custom;
pub use self::dart::Dart;
pub use self::element::Element;
pub use self::erased_element::ErasedElement;
pub use self::formatter::{Formatter, IoFmt};
pub use self::go::Go;
pub use self::into_tokens::IntoTokens;
pub use self::java::Java;
pub use self::js::JavaScript;
pub use self::python::Python;
pub use self::quoted::Quoted;
pub use self::rust::Rust;
pub use self::tokens::Tokens;
pub use self::write_tokens::WriteTokens;

#[cfg(test)]
mod tests {
    use crate::rust::Rust;
    use crate::tokens::Tokens;

    #[test]
    fn test_nested() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push("fn foo() -> u32 {");
        toks.nested("return 42;");
        toks.push("}");

        let output = toks.to_string().unwrap();
        assert_eq!("fn foo() -> u32 {\n    return 42;\n}", output.as_str());
    }
}
