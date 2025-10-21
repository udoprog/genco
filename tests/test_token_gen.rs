//! Test to assert that the tokens generated are equivalent.

use genco::fmt;
use genco::prelude::*;
use genco::tokens::ItemStr;

use genco::__priv::{indentation, line, literal, push, space, static_};

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
            static_("foo"),
            push(),
            static_("bar"),
            push(),
            static_("baz"),
            indentation(1),
            static_("hello"),
            indentation(-1),
            static_("out?")
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
            push(),
            literal("0".into()),
            push(),
            literal("1".into()),
            push(),
            literal("2".into()),
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
            push(),
            literal("0".into()),
            push(),
            literal("1".into()),
            push(),
            literal("2".into()),
        ],
        tokens,
    };
}

#[test]
fn test_tricky_continuation() {
    let mut output = rust::Tokens::new();

    let bar = ItemStr::static_("bar");

    quote_in! {
        output =>
        foo, $(ref output {
            output.append(&bar);
            output.append(static_(","));
            output.space();
        })baz
        biz
    };

    assert_eq! {
        output,
        vec![
            static_("foo,"),
            space(),
            static_("bar"),
            static_(","),
            space(),
            static_("baz"),
            push(),
            static_("biz"),
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
            static_("a"),
            indentation(1),
            static_("b"),
            indentation(-1),
            static_("c")
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
            static_("a"),
            indentation(1),
            static_("b"),
            indentation(-1),
            static_("c")
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
            static_("foo"),
            space(),
            literal("0".into()),
            space(),
            literal("3".into()),
            literal("1".into()),
            space(),
            literal("4".into()),
            literal("2".into()),
            space(),
            literal("5".into())
        ],
        tokens,
    };

    let tokens: rust::Tokens = quote! {
        foo $(for (a, b) in (0..3).zip(3..6) { $a $b })
    };

    assert_eq! {
        vec![
            static_("foo"),
            space(),
            literal("0".into()),
            space(),
            literal("3".into()),
            literal("1".into()),
            space(),
            literal("4".into()),
            literal("2".into()),
            space(),
            literal("5".into())
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
            static_("You"),
            space(),
            static_("are:fine"),
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
            static_("You"),
            space(),
            static_("are:"),
            space(),
            literal("0".into()),
            static_(","),
            space(),
            literal("1".into()),
            static_(","),
            space(),
            literal("2".into()),
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
            static_("foo"),
            push(),
            static_("foo2"),
            push(),
            static_("baz"),
            push(),
            static_("baz2"),
            push(),
            static_("biz"),
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
        vec![static_("a")]
    };

    assert_eq! {
        test(Alt::B),
        vec![static_("b")]
    };

    assert_eq! {
        test2(Alt::A),
        vec![static_("a")]
    };

    assert_eq! {
        test2(Alt::B),
        vec![static_("b")]
    };

    assert_eq! {
        test2_cond(Alt::A, true),
        vec![static_("a")]
    };

    assert_eq! {
        test2_cond(Alt::A, false),
        vec![static_("b")]
    };
}

#[test]
fn test_let() {
    let tokens: rust::Tokens = quote! {
        $(let x = 1) $x
    };

    assert_eq! {
        tokens,
        vec![space(), literal("1".into())]
    };

    // Tuple binding
    let tokens: rust::Tokens = quote! {
        $(let (a, b) = ("c", "d")) $a, $b
    };

    assert_eq! {
        tokens,
        vec![
            space(), literal("c".into()),
            static_(","),
            space(), literal("d".into())
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
        vec![space(), literal("barbaz".into())]
    };

    // Complex expression
    let x = 2;
    let tokens: rust::Tokens = quote! {
        $(let even = if x % 2 == 0 { "even" } else { "odd" }) $even
    };

    assert_eq! {
        tokens,
        vec![space(), literal("even".into())]
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
            push(),
            static_("First"),
            space(),
            static_("is"),
            space(),
            literal("A".into()),
            push(),
            static_("Second"),
            space(),
            static_("is"),
            space(),
            literal("B".into())
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
        vec![static_(","), static_(",")]
    };

    let tokens: rust::Tokens = quote! {
        $(for _ in 0..3 join( ,) =>)
    };

    assert_eq! {
        tokens,
        vec![space(), static_(","), space(), static_(",")]
    };

    let tokens: rust::Tokens = quote! {
          $(for _ in 0..3 join(, ) =>)
    };

    assert_eq! {
        tokens,
        vec![static_(","), space(), static_(","), space()]
    };

    let tokens: rust::Tokens = quote! {
          $(for _ in 0..3 join( , ) =>)
    };

    assert_eq! {
        tokens,
        vec![space(), static_(","), space(), static_(","), space()]
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
            static_("a"),
            static_("b")
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
            static_("a"),
            static_("b")
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
            static_("a"),
            static_("b")
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
            static_("if"),
            space(),
            static_("a:"),
            indentation(1),
            static_("if"),
            space(),
            static_("b:"),
            indentation(1),
            static_("foo"),
            indentation(-2),
            static_("else:"),
            indentation(1),
            static_("c"),
            indentation(-1)
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
            static_("if"),
            space(),
            static_("a:"),
            indentation(1),
            static_("if"),
            space(),
            static_("b:"),
            indentation(1),
            static_("foo"),
            indentation(-2),
            line(),
            static_("baz"),
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
            static_("def"),
            space(),
            static_("foo():"),
            indentation(1),
            static_("pass"),
            indentation(-1),
            line(),
            static_("def"),
            space(),
            static_("bar():"),
            indentation(1),
            static_("pass"),
            indentation(-1)
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
            static_("fn"),
            space(),
            static_("foo()"),
            space(),
            static_("{"),
            push(),
            static_("}"),
            line(),
            static_("fn"),
            space(),
            static_("bar()"),
            space(),
            static_("{"),
            push(),
            static_("}")
        ],
        tokens.clone(),
    };

    assert_eq!(
        vec!["fn foo() {", "}", "", "fn bar() {", "}",],
        tokens.to_file_vec()?
    );

    Ok(())
}
