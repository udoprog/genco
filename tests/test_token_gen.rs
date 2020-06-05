//! Test to assert that the tokens generated are equivalent.
//!
//! Note: Because of genco::LangBox, building Eq implementations are hard, so
//! we do comparisons based on `fmt::Debug` representation, which is already
//! available. But do note that they will not represent language items.

use genco::prelude::*;

/// Check that the expression equals a collection of items.
macro_rules! assert_items_eq {
    ($value:expr, [$($expect:expr,)*]) => {{
        let value: Tokens<Rust> = $value;
        let expected: Vec<Item<Rust>> = vec![$($expect,)*];
        assert_eq!(format!("{:?}", value), format!("{:?}", expected));
    }};

    ($value:expr, [$($expect:expr),*]) => {
        assert_items_eq!($value, [$($expect,)*])
    };
}

#[test]
fn test_token_gen() {
    use genco::{Item, Item::*, ItemStr::*};

    assert_items_eq! {
        quote! {
            foo
            bar
            baz
                #{tokens => quote_in! { tokens => hello }}
            out?
        },
        [
            Literal(Static("foo")),
            Push,
            Literal(Static("bar")),
            Push,
            Literal(Static("baz")),
            Indent,
            Literal(Static("hello")),
            Unindent,
            Literal(Static("out?"))
        ]
    }
}

#[test]
fn test_iterator_gen() {
    use genco::{Item, Item::*, ItemStr::*};

    assert_items_eq! {
        quote! {
            #((0..3).map(|v| quote! {
                #(None::<()>)
                #v
            }))*
        },
        [
            Push,
            Literal(Box("0".into())),
            Push,
            Literal(Box("1".into())),
            Push,
            Literal(Box("2".into())),
        ]
    };
}
