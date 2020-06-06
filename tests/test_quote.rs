use genco::prelude::*;

#[test]
fn test_quote() {
    let test = "one".quoted();

    let tokens: rust::Tokens = quote! {
        fn test() -> u32 {
            println!("{}", #(test));

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", \"one\");\n\n    42\n}",
        tokens.to_string().unwrap()
    );

    let tokens: rust::Tokens = quote! {
        fn test() -> u32 {
            println!("{}", #("two".quoted()));

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", \"two\");\n\n    42\n}",
        tokens.to_string().unwrap()
    );

    let tokens: rust::Tokens = quote! {
        fn test() -> u32 {
            println!("{}", ##("two".quoted()));

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", #(\"two\".quoted()));\n\n    42\n}",
        tokens.to_string().unwrap()
    );
}

#[test]
fn test_tight_quote() {
    let foo = "foo";
    let bar = "bar";
    let baz = "baz";
    let tokens: rust::Tokens = quote!(#(foo)#(bar)#(baz));

    assert_eq!("foobarbaz", tokens.to_string().unwrap());
}

#[test]
fn test_escape() {
    let tokens: rust::Tokens = quote!(#### ## #### #### ## ## ##[test]);
    assert_eq!("## # ## ## # # #[test]", tokens.to_string().unwrap());
}

#[test]
fn test_scope() {
    let tokens: rust::Tokens = quote! {
        // Nested factory.
        #(tokens => {
            quote_in!(tokens => fn test() -> u32 { 42 });
        })
    };

    assert_eq!("fn test() -> u32 { 42 }", tokens.to_string().unwrap());
}
