#![feature(proc_macro_hygiene)]

use genco::{quote, quote_in, Rust, Tokens};

#[test]
fn test_token_gen() {
    let tokens: Tokens<Rust> = quote! {
        foo
        bar
        baz
            #{tokens => {
                quote_in! { tokens =>
                    hello
                }
            }}
        out?
    };

    println!("{:?}", tokens);
}
