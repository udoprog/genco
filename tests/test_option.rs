use genco::prelude::*;

#[test]
fn test_option() {
    let test1 = Some(quote!(println!("{}", #("one".quoted()))));
    let test2 = None::<rust::Tokens>;

    let tokens: rust::Tokens = quote! {
        fn test() -> u32 {
            #test1
            #test2

            42
        }
    };

    assert_eq!(
        vec![
            "fn test() -> u32 {",
            "    println!(\"{}\", \"one\")",
            "",
            "    42",
            "}"
        ],
        tokens.to_file_vec().unwrap()
    );
}
