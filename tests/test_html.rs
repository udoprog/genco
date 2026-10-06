use genco::prelude::*;

#[test]
fn test_quoted_escapes() -> genco::fmt::Result {
    let toks: html::Tokens = quote!(<a title=$(quoted("& < > \" ' x"))></a>);
    assert_eq!(
        toks.to_string()?,
        "<a title=\"&amp; &lt; &gt; &quot; &apos; x\"></a>"
    );
    Ok(())
}

#[test]
fn test_text_escapes() -> genco::fmt::Result {
    let toks: html::Tokens = quote!(<p>$(html::text("& < > \" ' x"))</p>);
    assert_eq!(toks.to_string()?, "<p>&amp; &lt; &gt; \" ' x</p>");

    let toks: html::Tokens = quote!(<p>$(html::text(String::from("plain")))</p>);
    assert_eq!(toks.to_string()?, "<p>plain</p>");
    Ok(())
}

#[test]
fn test_raw_interpolation_is_verbatim() -> genco::fmt::Result {
    let toks: html::Tokens = quote!(<p>$("<b>")</p>);
    assert_eq!(toks.to_string()?, "<p><b></p>");
    Ok(())
}
