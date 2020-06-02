#![feature(proc_macro_hygiene)]

use genco::{quote, Rust, Tokens};

#[test]
fn test_indentation_rules() {
    let rule1: Tokens<Rust> = quote!(fn     test());

    let rule2: Tokens<Rust> = quote! {
        fn test() {
            println!("Hello...");


            println!("... World!");
        }
    };

    let rule3: Tokens<Rust> = quote! {
        fn test() {
            println!("Hello...");
            println!("... World!");
          }
    };

    assert_eq!("fn test()", rule1.to_string().unwrap());

    assert_eq!(
        "fn test() {\n    println!(\"Hello...\");\n\n    println!(\"... World!\");\n}",
        rule2.to_string().unwrap()
    );

    assert_eq!(
        "fn test() {\n    println!(\"Hello...\");\n    println!(\"... World!\");\n}",
        rule3.to_string().unwrap()
    );
}
