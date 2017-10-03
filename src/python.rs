//! Specialization for Python code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt;

/// Python token specialization.
#[derive(Debug, Clone)]
pub enum Python {
}

impl Custom for Python {
    type Extra = ();

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\u{0007}' => out.write_str("\\b")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\u{0014}' => out.write_str("\\f")?,
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
    use super::Python;
    use quoted::Quoted;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }
}
