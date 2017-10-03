//! Specialization for JavaScript code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt;

/// JavaScript token specialization.
#[derive(Debug, Clone)]
pub enum JavaScript {
}

impl Custom for JavaScript {
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
    use super::JavaScript;
    use quoted::Quoted;

    #[test]
    fn test_function() {
        let mut file: Tokens<JavaScript> = Tokens::new();

        file.push("function foo(v) {");
        file.nested(toks!("return v + ", ", World".quoted(), ";"));
        file.push("}");

        file.push(toks!("foo(", "Hello".quoted(), ");"));

        assert_eq!(
            Ok(String::from(
                "function foo(v) {\n  return v + \", World\";\n}\nfoo(\"Hello\");",
            )),
            file.to_string()
        );
    }

    #[test]
    fn test_string() {
        let mut toks: Tokens<JavaScript> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!(Ok(String::from("\"hello \\n world\"")), toks.to_string());
    }
}
