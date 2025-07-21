use genco::prelude::*;

#[test]
fn test_register() -> genco::fmt::Result {
    let import = rust::import("std::iter", "FromIterator").with_alias("_");

    let tokens: Tokens<Rust> = quote! {
        $(register(import))
        // additional lines are ignored!

        fn test() -> u32 {
            42
        }
    };

    println!("{tokens:?}");

    assert_eq!(
        vec![
            "use std::iter::FromIterator as _;",
            "",
            "fn test() -> u32 {",
            "    42",
            "}"
        ],
        tokens.to_file_vec()?
    );

    Ok(())
}
