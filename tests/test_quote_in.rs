#![feature(proc_macro_hygiene)]

use genco::{quote_in, Rust, Tokens};

#[test]
fn test_quote_in() {
    let mut tokens = Tokens::<Rust>::new();

    quote_in! {
        tokens => {
            fn hello() -> u32 { 42 }
        }
    }
}
