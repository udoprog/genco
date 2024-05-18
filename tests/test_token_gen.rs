//! Test to assert that the tokens generated are equivalent.

use genco::fmt;
use genco::prelude::*;
use genco::tokens::{Item::*, ItemStr::*};

#[test]
fn test_token_gen() {
    let tokens: rust::Tokens = quote! {
        foo
        bar
        baz
            $(ref tokens => quote_in! { *tokens => hello })
        out?
    };

    assert_eq! {
        vec![
            Literal(Static("foo")),
            Push,
            Literal(Static("bar")),
            Push,
            Literal(Static("baz")),
            Indentation(1),
            Literal(Static("hello")),
            Indentation(-1),
            Literal(Static("out?"))
        ],
        tokens,
    }
}

#[test]
fn test_iterator_gen() {
    let tokens: rust::Tokens = quote! {
        $(ref t => for n in 0..3 {
            t.push();
            t.append(n);
        })
    };

    assert_eq! {
        vec![
            Push,
            Literal(Box("0".into())),
            Push,
            Literal(Box("1".into())),
            Push,
            Literal(Box("2".into())),
        ],
        tokens,
    };

    let tokens: rust::Tokens = quote! {
        $(ref t {
            for n in 0..3 {
                t.push();
                t.append(n);
            }
        })
    };

    assert_eq! {
        vec![
            Push,
            Literal(Box("0".into())),
            Push,
            Literal(Box("1".into())),
            Push,
            Literal(Box("2".into())),
        ],
        tokens,
    };
}

#[test]
fn test_tricky_continuation() {
    let mut output = rust::Tokens::new();

    let bar = Static("bar");

    quote_in! {
        output =>
        foo, $(ref output {
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

    assert_eq! {
        a,
        vec![
            Literal(Static("a")),
            Indentation(1),
            Literal(Static("b")),
            Indentation(-1),
            Literal(Static("c"))
        ]
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
            Indentation(1),
            Literal(Static("b")),
            Indentation(-1),
            Literal(Static("c"))
        ]
    };
}

#[test]
fn test_repeat() {
    let tokens: rust::Tokens = quote! {
        foo $(for (a, b) in (0..3).zip(3..6) => $a $b)
    };

    assert_eq! {
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
        ],
        tokens,
    };

    let tokens: rust::Tokens = quote! {
        foo $(for (a, b) in (0..3).zip(3..6) { $a $b })
    };

    assert_eq! {
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
        ],
        tokens,
    };
}

#[test]
fn test_tight_quote() {
    let output: rust::Tokens = quote! {
        You are:$("fine")
    };

    assert_eq! {
        output,
        vec![
            Literal(Static("You")),
            Space,
            Literal(Static("are:fine")),
        ]
    };
}

#[test]
fn test_tight_repitition() {
    let output: rust::Tokens = quote! {
        You are: $(for v in 0..3 join (, ) => $v)
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
        ]
    };
}

#[test]
fn test_if() {
    let a = true;
    let b = false;

    let output: rust::Tokens = quote! {
        $(if a => foo)
        $(if a { foo2 })
        $(if b { bar })
        $(if b => bar2)
        $(if a => baz)
        $(if a { baz2 })
        $(if b { not_biz } else { biz })
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
        ]
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
            $(match alt { Alt::A => a, Alt::B => b })
        }
    }

    fn test2(alt: Alt) -> rust::Tokens {
        quote! {
            $(match alt { Alt::A => { a }, Alt::B => { b } })
        }
    }

    fn test2_cond(alt: Alt, cond: bool) -> rust::Tokens {
        quote! {
            $(match alt { Alt::A if cond => { a }, _ => { b } })
        }
    }

    assert_eq! {
        test(Alt::A),
        vec![Literal(Static("a"))]
    };

    assert_eq! {
        test(Alt::B),
        vec![Literal(Static("b"))]
    };

    assert_eq! {
        test2(Alt::A),
        vec![Literal(Static("a"))]
    };

    assert_eq! {
        test2(Alt::B),
        vec![Literal(Static("b"))]
    };

    assert_eq! {
        test2_cond(Alt::A, true),
        vec![Literal(Static("a"))]
    };

    assert_eq! {
        test2_cond(Alt::A, false),
        vec![Literal(Static("b"))]
    };
}

#[test]
fn test_let() {
    let tokens: rust::Tokens = quote! {
        $(let x = 1) $x
    };

    assert_eq! {
        tokens,
        vec![Space, Literal("1".into())]
    };

    // Tuple binding
    let tokens: rust::Tokens = quote! {
        $(let (a, b) = ("c", "d")) $a, $b
    };

    assert_eq! {
        tokens,
        vec![
            Space, Literal("c".into()),
            Literal(Static(",")),
            Space, Literal("d".into())
        ]
    };

    // Function call in expression
    let x = "bar";
    fn baz(s: &str) -> String {
        format!("{s}baz")
    }

    let tokens: rust::Tokens = quote! {
        $(let a = baz(x)) $a
    };

    assert_eq! {
        tokens,
        vec![Space, Literal("barbaz".into())]
    };

    // Complex expression
    let x = 2;
    let tokens: rust::Tokens = quote! {
        $(let even = if x % 2 == 0 { "even" } else { "odd" }) $even
    };

    assert_eq! {
        tokens,
        vec![Space, Literal("even".into())]
    };
}

#[test]
fn test_mutable_let() {
    let path = "A.B.C.D";

    let tokens: Tokens<()> = quote! {
        $(let mut items = path.split('.'))
        $(if let Some(first) = items.next() =>
            First is $first
        )
        $(if let Some(second) = items.next() =>
            Second is $second
        )
    };

    assert_eq!(
        tokens,
        vec![
            Push,
            Literal(Static("First")),
            Space,
            Literal(Static("is")),
            Space,
            Literal("A".into()),
            Push,
            Literal(Static("Second")),
            Space,
            Literal(Static("is")),
            Space,
            Literal("B".into())
        ]
    );
}

#[test]
fn test_empty_loop_whitespace() {
    // Bug: This should generate two commas. But did generate a space following
    // it!
    let tokens: rust::Tokens = quote! {
        $(for _ in 0..3 join(,) =>)
    };

    assert_eq! {
        tokens,
        vec![Literal(Static(",")), Literal(Static(","))]
    };

    let tokens: rust::Tokens = quote! {
        $(for _ in 0..3 join( ,) =>)
    };

    assert_eq! {
        tokens,
        vec![Space, Literal(Static(",")), Space, Literal(Static(","))]
    };

    let tokens: rust::Tokens = quote! {
          $(for _ in 0..3 join(, ) =>)
    };

    assert_eq! {
        tokens,
        vec![Literal(Static(",")), Space, Literal(Static(",")), Space]
    };

    let tokens: rust::Tokens = quote! {
          $(for _ in 0..3 join( , ) =>)
    };

    assert_eq! {
        tokens,
        vec![Space, Literal(Static(",")), Space, Literal(Static(",")), Space]
    };
}

#[test]
fn test_indentation_empty() {
    let tokens: rust::Tokens = quote! {
        a
            $(for _ in 0..3 =>)
        b
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("a")),
            Literal(Static("b"))
        ]
    };

    let tokens: rust::Tokens = quote! {
        a
            $(if false {})
        b
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("a")),
            Literal(Static("b"))
        ]
    };

    let tokens: rust::Tokens = quote! {
        a
            $(ref _tokens =>)
        b
    };

    assert_eq! {
        tokens,
        vec![
            Literal(Static("a")),
            Literal(Static("b"))
        ]
    };
}

#[test]
fn test_indentation_management() {
    let tokens: rust::Tokens = quote! {
        if a:
            if b:
                foo
        else:
            c
    };

    assert_eq! {
        vec![
            Literal(Static("if")),
            Space,
            Literal(Static("a:")),
            Indentation(1),
            Literal(Static("if")),
            Space,
            Literal(Static("b:")),
            Indentation(1),
            Literal(Static("foo")),
            Indentation(-2),
            Literal(Static("else:")),
            Indentation(1),
            Literal(Static("c")),
            Indentation(-1)
        ],
        tokens,
    };

    let tokens: rust::Tokens = quote! {
        if a:
            if b:
                foo

        $(if false => bar)

        $(if true => baz)
    };

    assert_eq! {
        vec![
            Literal(Static("if")),
            Space,
            Literal(Static("a:")),
            Indentation(1),
            Literal(Static("if")),
            Space,
            Literal(Static("b:")),
            Indentation(1),
            Literal(Static("foo")),
            Indentation(-2),
            Line,
            Literal(Static("baz")),
        ],
        tokens,
    };
}

#[test]
fn test_indentation_management2() -> fmt::Result {
    let tokens: python::Tokens = quote! {
        def foo():
            pass

        def bar():
            pass
    };

    assert_eq! {
        vec![
            Literal(Static("def")),
            Space,
            Literal(Static("foo():")),
            Indentation(1),
            Literal(Static("pass")),
            Indentation(-1),
            Line,
            Literal(Static("def")),
            Space,
            Literal(Static("bar():")),
            Indentation(1),
            Literal(Static("pass")),
            Indentation(-1)
        ],
        tokens,
    };

    assert_eq!(
        vec!["def foo():", "    pass", "", "def bar():", "    pass",],
        tokens.to_file_vec()?
    );

    Ok(())
}

#[test]
fn test_lines() -> fmt::Result {
    let mut tokens: rust::Tokens = quote! {
        fn foo() {
        }
    };

    tokens.line();

    quote_in! { tokens =>
        $(if false =>)
        fn bar() {
        }
    };

    assert_eq! {
        vec![
            Literal(Static("fn")),
            Space,
            Literal(Static("foo()")),
            Space,
            Literal(Static("{")),
            Push,
            Literal(Static("}")),
            Line,
            Literal(Static("fn")),
            Space,
            Literal(Static("bar()")),
            Space,
            Literal(Static("{")),
            Push,
            Literal(Static("}"))
        ],
        tokens.clone(),
    };

    assert_eq!(
        vec!["fn foo() {", "}", "", "fn bar() {", "}",],
        tokens.to_file_vec()?
    );

    Ok(())
}
