use genco::prelude::*;

#[test]
fn test_option() -> genco::fmt::Result {
    let test1 = Some(quote!(println!("{}", $(quoted("one")))));
    let test2 = None::<rust::Tokens>;

    let tokens: rust::Tokens = quote! {
        fn test_option() -> u32 {
            $test1
            $test2

            42
        }
    };

    assert_eq!(
        vec![
            "fn test_option() -> u32 {",
            "    println!(\"{}\", \"one\")",
            "",
            "    42",
            "}"
        ],
        tokens.to_file_vec()?
    );

    Ok(())
}
