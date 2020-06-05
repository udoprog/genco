//! Test to assert that the tokens generated are equivalent.
//!
//! Note: Because of genco::LangBox, building Eq implementations are hard, so
//! we do comparisons based on `fmt::Debug` representation, which is already
//! available. But do note that they will not represent language items.

use genco::prelude::*;
use genco::{Item, Item::*, ItemStr::*};

/// Check that the expression equals a collection of items.
macro_rules! assert_items_eq {
    ($value:expr, [$($expect:expr,)*]) => {{
        let value: rust::Tokens = $value;
        let expected: Vec<Item<Rust>> = vec![$($expect,)*];
        assert_eq!(format!("{:?}", value), format!("{:?}", expected));
    }};

    ($value:expr, [$($expect:expr),*]) => {
        assert_items_eq!($value, [$($expect,)*])
    };
}

#[test]
fn test_token_gen() {
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

#[test]
fn test_tricky_continuation() {
    let mut output = rust::Tokens::new();
    // let output = &mut output;

    let bar = Static("bar");

    quote_in! {
        &mut output => foo, #{*output => {
            output.append(&bar);
            output.append(Static(","));
            output.spacing();
        }}baz
        biz
    };

    assert_items_eq! {
        output,
        [
            Literal(Static("foo,")),
            Spacing,
            Literal(Static("bar")),
            Literal(Static(",")),
            Spacing,
            Literal(Static("baz")),
            Push,
            Literal(Static("biz")),
        ]
    };
}

#[test]
fn test_indentation() {
    // Bug: Since we carry the span of out, the line after counts as unindented.
    //
    // These two should be identical:

    let mut a = rust::Tokens::new();

    quote_in! { a =>
        a
            b
        c
    };

    assert_items_eq! {
        a,
        [Literal(Static("a")), Indent, Literal(Static("b")), Unindent, Literal(Static("c"))]
    };

    let mut b = rust::Tokens::new();

    quote_in! {
        b =>
        a
            b
        c
    };

    assert_items_eq! {
        b,
        [Literal(Static("a")), Indent, Literal(Static("b")), Unindent, Literal(Static("c"))]
    };
}
