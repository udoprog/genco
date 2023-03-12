use genco::prelude::*;

/// basic smoketests.
#[test]
fn test_quote_in() -> genco::fmt::Result {
    let mut tokens = rust::Tokens::new();
    quote_in!(tokens => fn hello() -> u32 { 42 });
    assert_eq!("fn hello() -> u32 { 42 }", tokens.to_string()?);
    Ok(())
}

/// quote_in! must expand into a unit expression.
#[test]
fn test_quote_into_unit() -> genco::fmt::Result {
    let tokens = &mut go::Tokens::new();
    quote_in!(*tokens => uint32);
    assert_eq!("uint32", tokens.to_string()?);
    Ok(())
}
