//! Specialization for Rust code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt;

/// Rust token specialization.
#[derive(Debug, Clone)]
pub enum Rust {
}

impl Custom for Rust {
    type Extra = ();

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\'' => out.write_str("\\'")?,
                '"' => out.write_str("\\\"")?,
                '\\' => out.write_str("\\\\")?,
                c => out.write_char(c)?,
            };
        }

        out.write_char('"')?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use rust::Rust;
    use quoted::Quoted;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }
}
