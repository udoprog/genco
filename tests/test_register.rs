#![feature(proc_macro_hygiene)]

use genco::rust::imported;
use genco::{quote, Rust, Tokens};

#[test]
fn test_register() {
    let import = imported("std::iter", "FromIterator").alias("_");

    let tokens: Tokens<Rust> = quote! {
        #@import
        // additional lines are ignored!

        fn test() -> u32 {
            42
        }
    };

    assert_eq!(
        vec![
            "use std::iter::FromIterator as _;",
            "",
            "fn test() -> u32 {",
            "    42",
            "}",
            ""
        ],
        tokens.to_file_vec().unwrap()
    );
}
