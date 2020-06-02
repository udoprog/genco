#![feature(proc_macro_hygiene)]

use genco::{quote, Rust, Tokens};

#[test]
fn test_quote_simple_expression() {
    let tokens: Tokens<Rust> = quote!(fn #"test"());
    assert_eq!("fn test()", tokens.to_string().unwrap());

    let expr: Tokens<Rust> = quote!(test);
    let tokens: Tokens<Rust> = quote!(fn #expr());
    assert_eq!("fn test()", tokens.to_string().unwrap());

    let tokens: Tokens<Rust> = quote!(fn #(expr)());
    assert_eq!("fn test()", tokens.to_string().unwrap());

    // inline macro expansion.
    let tokens: Tokens<Rust> = quote!(fn #(quote!(test))());
    assert_eq!("fn test()", tokens.to_string().unwrap());
}
