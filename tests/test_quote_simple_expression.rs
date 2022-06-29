use genco::prelude::*;

#[test]
fn test_quote_simple_expression() -> genco::fmt::Result {
    let tokens: Tokens<Rust> = quote!(fn $("test")());
    assert_eq!("fn test()", tokens.to_string()?);

    let expr = &quote!(test);
    let tokens: Tokens<Rust> = quote!(fn $expr());
    assert_eq!("fn test()", tokens.to_string()?);

    let tokens: Tokens<Rust> = quote!(fn $(expr)());
    assert_eq!("fn test()", tokens.to_string()?);

    // inline macro expansion.
    let tokens: Tokens<Rust> = quote!(fn $(quote!(test))());
    assert_eq!("fn test()", tokens.to_string()?);

    Ok(())
}
