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
                #(tokens => quote_in! { tokens => hello })
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
            #(t => for n in 0..3 {
                t.push();
                t.append(n);
            })
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
        &mut output => foo, #(*output => {
            output.append(&bar);
            output.append(Static(","));
            output.space();
        })baz
        biz
    };

    assert_items_eq! {
        output,
        [
            Literal(Static("foo,")),
            Space,
            Literal(Static("bar")),
            Literal(Static(",")),
            Space,
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

#[test]
fn test_repeat() {
    let mut output = rust::Tokens::new();

    let a = 0..3;
    let b = 3..6;

    quote_in! {
        &mut output => foo #((a, b) in a.zip(b) => #a #b)
    };

    assert_items_eq! {
        output,
        [
            Literal(Static("foo")),
            Space,
            Literal("0".into()),
            Space,
            Literal("3".into()),
            Literal("1".into()),
            Space,
            Literal("4".into()),
            Literal("2".into()),
            Space,
            Literal("5".into())
        ]
    };
}

#[test]
fn test_tight_quote() {
    let output: rust::Tokens = quote! {
        You are:#("fine")
    };

    assert_items_eq! {
        output,
        [
            Literal(Static("You")),
            Space,
            Literal(Static("are:")),
            Literal("fine".into()),
        ]
    };
}

#[test]
fn test_tight_repitition() {
    let output: rust::Tokens = quote! {
        You are: #(v in 0..3 join (, ) => #v)
    };

    assert_items_eq! {
        output,
        [
            Literal(Static("You")),
            Space,
            Literal(Static("are:")),
            Space,
            Literal("0".into()),
            Literal(Static(",")),
            Space,
            Literal("1".into()),
            Literal(Static(",")),
            Space,
            Literal("2".into()),
        ]
    };
}
