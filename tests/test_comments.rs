use genco::prelude::*;

#[test]
fn test_java_block_comment_terminator() -> genco::fmt::Result {
    let toks: java::Tokens = quote! {
        $(java::block_comment(["hello */ evil", "a **/ b */"]))
        class A {}
    };

    assert_eq!(
        vec![
            "/**",
            " * hello *&#47; evil",
            " * a **&#47; b *&#47;",
            " */",
            "class A {}",
        ],
        toks.to_file_vec()?
    );

    Ok(())
}

#[test]
fn test_java_block_comment_newlines() -> genco::fmt::Result {
    let toks: java::Tokens = quote! {
        $(java::block_comment(["line1\nline2", "line3\r\nline4\rline5"]))
        class A {}
    };

    assert_eq!(
        vec![
            "/**",
            " * line1",
            " * line2",
            " * line3",
            " * line4",
            " * line5",
            " */",
            "class A {}",
        ],
        toks.to_file_vec()?
    );

    Ok(())
}

#[test]
fn test_csharp_comment_newlines() -> genco::fmt::Result {
    let toks: csharp::Tokens = quote! {
        $(csharp::comment(["line1\nline2", "line3\r\nline4"]))
        class A {}
    };

    assert_eq!(
        vec!["// line1", "// line2", "// line3", "// line4", "class A {}"],
        toks.to_file_vec()?
    );

    Ok(())
}

#[test]
fn test_csharp_block_comment_newlines() -> genco::fmt::Result {
    let toks: csharp::Tokens = quote! {
        $(csharp::block_comment(["line1\nline2", "line3\r\nline4"]))
        class A {}
    };

    assert_eq!(
        vec![
            "/// line1",
            "/// line2",
            "/// line3",
            "/// line4",
            "class A {}"
        ],
        toks.to_file_vec()?
    );

    Ok(())
}

#[test]
fn test_dart_doc_comment_newlines() -> genco::fmt::Result {
    let toks: dart::Tokens = quote! {
        $(dart::doc_comment(["line1\nline2", "line3\r\nline4"]))
        class A {}
    };

    assert_eq!(
        vec![
            "/// line1",
            "/// line2",
            "/// line3",
            "/// line4",
            "class A {}"
        ],
        toks.to_file_vec()?
    );

    Ok(())
}

#[test]
fn test_comment_blank_lines() -> genco::fmt::Result {
    let toks: csharp::Tokens = quote! {
        $(csharp::comment(["a\n\nb", ""]))
        class A {}
    };

    assert_eq!(
        vec!["// a", "//", "// b", "//", "class A {}"],
        toks.to_file_vec()?
    );

    Ok(())
}
