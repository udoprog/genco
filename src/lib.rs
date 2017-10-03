//! ## Simple and flexible code generator (genco)
#![deny(missing_docs)]

#[macro_use]
pub mod macros;
mod custom;
mod element;
mod formatter;
mod java;
mod js;
mod python;
mod quoted;
mod rust;
mod tokens;
mod write_tokens;

pub use self::element::Element;
pub use self::java::Java;
pub use self::rust::Rust;
pub use self::js::JavaScript;
pub use self::python::Python;
pub use self::tokens::Tokens;
pub use self::formatter::WriteFormatter;
pub use self::write_tokens::WriteTokens;
pub use self::quoted::Quoted;

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use rust::Rust;

    #[test]
    fn test_nested() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push("fn foo() -> u32 {");
        toks.nested("return 42;");
        toks.push("}");

        let output = toks.to_string().unwrap();
        assert_eq!("fn foo() -> u32 {\n  return 42;\n}", output.as_str());
    }
}
