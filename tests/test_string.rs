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
