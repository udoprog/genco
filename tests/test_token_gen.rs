//! Test to assert that the tokens generated are equivalent.
//!
//! Note: Because of genco::LangBox, building Eq implementations are hard, so
//! we do comparisons based on `fmt::Debug` representation, which is already
//! available. But do note that they will not represent language items.

use std::num::NonZeroI16;

use genco::prelude::*;
use genco::{Item, Item::*, ItemStr::*};

#[test]
fn test_token_gen() {
    assert_eq! {
        quote! {
            foo
            bar
            baz
                #(tokens => quote_in! { tokens => hello })
            out?
        },
        vec![
            Literal(Static("foo")),
            Push,
            Literal(Static("bar")),
            Push,
            Literal(Static("baz")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("hello")),
            Indentation(NonZeroI16::new(-1).unwrap()),
            Literal(Static("out?"))
        ] as Vec<Item<Rust>>
    }
}

#[test]
fn test_iterator_gen() {
    assert_eq! {
        quote! {
            #(t => for n in 0..3 {
                t.push();
                t.append(n);
            })
        },
        vec![
            Push,
            Literal(Box("0".into())),
            Push,
            Literal(Box("1".into())),
            Push,
            Literal(Box("2".into())),
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_tricky_continuation() {
    let mut output = rust::Tokens::new();

    let bar = Static("bar");

    quote_in! {
        &mut output =>
        foo, #(*output => {
            output.append(&bar);
            output.append(Static(","));
            output.space();
        })baz
        biz
    };

    assert_eq! {
        output,
        vec![
            Literal(Static("foo,")),
            Space,
            Literal(Static("bar")),
            Literal(Static(",")),
            Space,
            Literal(Static("baz")),
            Push,
            Literal(Static("biz")),
        ] as Vec<Item<Rust>>
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

    assert_eq! {
        a,
        vec![
            Literal(Static("a")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("b")),
            Indentation(NonZeroI16::new(-1).unwrap()),
            Literal(Static("c"))
        ] as Vec<Item<Rust>>
    };

    let mut b = rust::Tokens::new();

    quote_in! {
        b =>
        a
            b
        c
    };

    assert_eq! {
        b,
        vec![
            Literal(Static("a")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("b")),
            Indentation(NonZeroI16::new(-1).unwrap()),
            Literal(Static("c"))
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_repeat() {
    let mut output = rust::Tokens::new();

    let a = 0..3;
    let b = 3..6;

    quote_in! {
        &mut output => foo #(for (a, b) in a.zip(b) => #a #b)
    };

    assert_eq! {
        output,
        vec![
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
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_tight_quote() {
    let output: rust::Tokens = quote! {
        You are:#("fine")
    };

    assert_eq! {
        output,
        vec![
            Literal(Static("You")),
            Space,
            Literal(Static("are:")),
            Literal("fine".into()),
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_tight_repitition() {
    let output: rust::Tokens = quote! {
        You are: #(for v in 0..3 join (, ) => #v)
    };

    assert_eq! {
        output,
        vec![
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
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_if() {
    let a = true;
    let b = false;

    let output: rust::Tokens = quote! {
        #(if a => foo)
        #(if a { foo2 })
        #(if b { bar })
        #(if b => bar2)
        #(if a => baz)
        #(if a { baz2 })
        #(if b { not_biz } else { biz })
    };

    assert_eq! {
        output,
        vec![
            Literal(Static("foo")),
            Push,
            Literal(Static("foo2")),
            Push,
            Literal(Static("baz")),
            Push,
            Literal(Static("baz2")),
            Push,
            Literal(Static("biz")),
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_match() {
    enum Alt {
        A,
        B,
    }

    fn test(alt: Alt) -> rust::Tokens {
        quote! {
            #(match alt { Alt::A => a, Alt::B => b })
        }
    }

    fn test2(alt: Alt) -> rust::Tokens {
        quote! {
            #(match alt { Alt::A => { a }, Alt::B => { b } })
        }
    }

    fn test2_cond(alt: Alt, cond: bool) -> rust::Tokens {
        quote! {
            #(match alt { Alt::A if cond => { a }, _ => { b } })
        }
    }

    assert_eq! {
        test(Alt::A),
        vec![Literal(Static("a"))] as Vec<Item<Rust>>
    };

    assert_eq! {
        test(Alt::B),
        vec![Literal(Static("b"))] as Vec<Item<Rust>>
    };

    assert_eq! {
        test2(Alt::A),
        vec![Literal(Static("a"))] as Vec<Item<Rust>>
    };

    assert_eq! {
        test2(Alt::B),
        vec![Literal(Static("b"))] as Vec<Item<Rust>>
    };

    assert_eq! {
        test2_cond(Alt::A, true),
        vec![Literal(Static("a"))] as Vec<Item<Rust>>
    };

    assert_eq! {
        test2_cond(Alt::A, false),
        vec![Literal(Static("b"))] as Vec<Item<Rust>>
    };
}

#[test]
fn test_empty_loop_whitespace() {
    // Bug: This should generate two commas. But did generate a space following
    // it!
    let tokens: rust::Tokens = quote! {
        #(for _ in 0..3 join(,) =>)
    };

    assert_eq! {
        tokens,
        vec![Literal(Static(",")), Literal(Static(","))] as Vec<Item<Rust>>
    };

    let tokens: rust::Tokens = quote! {
        #(for _ in 0..3 join( ,) =>)
    };

    assert_eq! {
        tokens,
        vec![Space, Literal(Static(",")), Space, Literal(Static(","))] as Vec<Item<Rust>>
    };

    let tokens: rust::Tokens = quote! {
          #(for _ in 0..3 join(, ) =>)
    };

    assert_eq! {
        tokens,
        vec![Literal(Static(",")), Space, Literal(Static(",")), Space] as Vec<Item<Rust>>
    };

    let tokens: rust::Tokens = quote! {
          #(for _ in 0..3 join( , ) =>)
    };

    assert_eq! {
        tokens,
        vec![Space, Literal(Static(",")), Space, Literal(Static(",")), Space] as Vec<Item<Rust>>
    };
}

#[test]
fn test_indentation_empty() {
    let tokens: rust::Tokens = quote! {
        a
            #(for _ in 0..3 =>)
        b
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("a")),
            Literal(Static("b"))
        ] as Vec<Item<Rust>>
    };

    let tokens: rust::Tokens = quote! {
        a
            #(if false {})
        b
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("a")),
            Literal(Static("b"))
        ] as Vec<Item<Rust>>
    };

    let tokens: rust::Tokens = quote! {
        a
            #(_tokens =>)
        b
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("a")),
            Literal(Static("b"))
        ] as Vec<Item<Rust>>
    };
}

#[test]
fn test_indentation_management() {
    assert_eq! {
        quote! {
            if a:
                if b:
                    foo
            else:
                c
        },
        vec![
            Literal(Static("if")),
            Space,
            Literal(Static("a:")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("if")),
            Space,
            Literal(Static("b:")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("foo")),
            Indentation(NonZeroI16::new(-2).unwrap()),
            Literal(Static("else:")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("c")),
            Indentation(NonZeroI16::new(-1).unwrap())
        ] as Vec<Item<Rust>>
    };

    let tokens = quote! {
        if a:
            if b:
                foo
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("if")),
            Space,
            Literal(Static("a:")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("if")),
            Space,
            Literal(Static("b:")),
            Indentation(NonZeroI16::new(1).unwrap()),
            Literal(Static("foo")),
            Indentation(NonZeroI16::new(-2).unwrap()),
        ] as Vec<Item<Rust>>
    };
}
