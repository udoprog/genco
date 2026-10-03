use genco::prelude::*;

#[test]
fn test_quoted() -> genco::fmt::Result {
    let t: dart::Tokens = quote!($[str](Hello $($(quoted("World")))));
    assert_eq!("\"Hello ${\"World\"}\"", t.to_string()?);

    let t: dart::Tokens = quote!($[str](Hello "World"));
    assert_eq!("\"Hello \\\"World\\\"\"", t.to_string()?);

    let t: dart::Tokens = quote!($[str](Hello $(World)));
    assert_eq!("\"Hello $World\"", t.to_string()?);

    let t: js::Tokens = quote!($[str](Hello $(World)));
    assert_eq!("`Hello ${World}`", t.to_string()?);
    Ok(())
}

#[test]
fn test_string_in_string_in() -> genco::fmt::Result {
    let t: dart::Tokens = quote!($[str](Hello $($[str]($($[str](World))))));
    assert_eq!("\"Hello ${\"${\"World\"}\"}\"", t.to_string()?);

    let t: js::Tokens = quote!($[str](Hello $($[str]($($[str](World))))));
    assert_eq!("`Hello ${`${\"World\"}`}`", t.to_string()?);
    Ok(())
}

#[test]
fn test_nested_quoted() -> genco::fmt::Result {
    let l2: js::Tokens = quote!(b $(quoted("c")) d);
    let t: js::Tokens = quote!($(quoted(l2)) + 1);
    assert_eq!("\"b \\\"c\\\" d\" + 1", t.to_string()?);

    let l2: js::Tokens = quote!(b $(quoted("c\"\\")) d);
    let t: js::Tokens = quote!($(quoted(l2)) + 1);
    assert_eq!("\"b \\\"c\\\\\\\"\\\\\\\\\\\" d\" + 1", t.to_string()?);

    let l3: js::Tokens = quote!(y $(quoted("z")) w);
    let l2: js::Tokens = quote!(x $(quoted(l3)) v);
    let t: js::Tokens = quote!($(quoted(l2)) + 1);
    assert_eq!("\"x \\\"y \\\\\\\"z\\\\\\\" w\\\" v\" + 1", t.to_string()?);
    Ok(())
}

#[test]
fn test_nested_quoted_in_str() -> genco::fmt::Result {
    let l2: js::Tokens = quote!(b $(quoted("c")) d);
    let t: js::Tokens = quote!($[str](a $($(quoted(l2))) e) + 1);
    assert_eq!("`a ${\"b \\\"c\\\" d\"} e` + 1", t.to_string()?);

    let l3: js::Tokens = quote!(y $(quoted("z")) w);
    let l2: js::Tokens = quote!(x $(quoted(l3)) v);
    let t: js::Tokens = quote!($[str](a $($(quoted(l2))) e) + 1);
    assert_eq!(
        "`a ${\"x \\\"y \\\\\\\"z\\\\\\\" w\\\" v\"} e` + 1",
        t.to_string()?
    );
    Ok(())
}

#[test]
fn test_str_in_nested_quoted() -> genco::fmt::Result {
    let l2: js::Tokens = quote!(b $[str](c) d);
    let t: js::Tokens = quote!($(quoted(l2)) + 1);
    assert_eq!("\"b \\\"c\\\" d\" + 1", t.to_string()?);

    let l3: js::Tokens = quote!(y $[str](z) w);
    let l2: js::Tokens = quote!(x $(quoted(l3)) v);
    let t: js::Tokens = quote!($(quoted(l2)) + 1);
    assert_eq!("\"x \\\"y \\\\\\\"z\\\\\\\" w\\\" v\" + 1", t.to_string()?);

    let l3: js::Tokens = quote!($[str](y "z" w));
    let l2: js::Tokens = quote!(x $(quoted(l3)) v);
    let t: js::Tokens = quote!($(quoted(l2)) + 1);
    assert_eq!(
        "\"x \\\"\\\\\\\"y \\\\\\\\\\\\\\\"z\\\\\\\\\\\\\\\" w\\\\\\\"\\\" v\" + 1",
        t.to_string()?
    );
    Ok(())
}
