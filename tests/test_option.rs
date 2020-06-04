#![feature(proc_macro_hygiene)]

use genco::{quote, Ext as _, Rust, Tokens};

#[test]
fn test_option() {
    let test1 = Some(quote!(println!("{}", #("one".quoted()))));
    let test2 = None::<Tokens<Rust>>;

    let tokens: Tokens<Rust> = quote! {
        fn test() -> u32 {
            #test1
            #test2

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", \"one\")\n\n    42\n}\n",
        tokens.to_file_string().unwrap()
    );
}
