//! Macros in GenCo

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// ## Examples
///
/// ```rust,ignore
/// let n1: genco::Tokens<()> = toks!("var v = ", "bar".quoted(), ";");
/// ```
#[macro_export]
macro_rules! toks {
    ($($vars:expr),*) => {
        {
            let mut tokens = $crate::tokens::Tokens::new();
            $(tokens.append($vars);)*
            tokens
        }
    }
}

#[cfg(test)]
mod tests {
    use quoted::Quoted;
    use tokens::Tokens;
    use js::JavaScript;

    #[test]
    fn test_quoted() {
        let n1: Tokens<JavaScript> = toks!("var v = ", "bar".quoted(), ";");
        assert_eq!("var v = \"bar\";", n1.to_string().unwrap().as_str());
    }
}
