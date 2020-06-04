#![feature(proc_macro_hygiene)]

use genco::{quote, quote_in, rust, Rust, Tokens};

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

#[test]
fn test_iterator_gen() {
    let things = 0..3;

    let tokens: Tokens<Rust> = quote! {
        #(things.map(|v| quote! {
            #(None::<rust::Type>)
            #v
        }))*
    };

    println!("{:?}", tokens);
    println!("{}", tokens.to_string().unwrap());
}
