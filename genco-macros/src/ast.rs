use proc_macro2::{Span, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};
use syn::Result;

use crate::static_buffer::StaticBuffer;

/// A single match arm in a match statement.
pub(crate) struct MatchArm {
    pub(crate) pattern: syn::Pat,
    pub(crate) condition: Option<syn::Expr>,
    pub(crate) block: TokenStream,
}

/// A delimiter that can be encoded.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
}

impl Delimiter {
    pub(crate) fn encode_open(self, output: &mut StaticBuffer) {
        let c = match self {
            Self::Parenthesis => '(',
            Self::Brace => '{',
            Self::Bracket => '[',
        };

        output.push(c);
    }

    pub(crate) fn encode_close(self, output: &mut StaticBuffer) {
        let c = match self {
            Self::Parenthesis => ')',
            Self::Brace => '}',
            Self::Bracket => ']',
        };

        output.push(c);
    }
}

#[derive(Debug)]
pub(crate) enum ControlKind {
    Space,
    Push,
    Line,
}

#[derive(Debug)]
pub(crate) struct Control {
    pub(crate) kind: ControlKind,
    pub(crate) span: Span,
}

impl Parse for Control {
    fn parse(input: ParseStream) -> Result<Self> {
        syn::custom_keyword!(space);
        syn::custom_keyword!(push);
        syn::custom_keyword!(line);

        if input.peek(space) {
            let space = input.parse::<space>()?;

            return Ok(Self {
                kind: ControlKind::Space,
                span: space.span,
            });
        }

        if input.peek(push) {
            let push = input.parse::<push>()?;

            return Ok(Self {
                kind: ControlKind::Push,
                span: push.span,
            });
        }

        if input.peek(line) {
            let line = input.parse::<line>()?;

            return Ok(Self {
                kind: ControlKind::Line,
                span: line.span,
            });
        }

        return Err(input.error("Expected one of: `space`, `push`, or `line`."));
    }
}

/// Items to process from the queue.
pub(crate) enum Ast {
    Tree {
        tt: TokenTree,
    },
    String {
        has_eval: bool,
        stream: TokenStream,
    },
    /// A quoted string.
    Quoted {
        s: syn::LitStr,
    },
    /// A literal value embedded in the stream.
    Literal {
        string: String,
    },
    DelimiterOpen {
        delimiter: Delimiter,
    },
    DelimiterClose {
        delimiter: Delimiter,
    },
    Control {
        control: Control,
    },
    EvalIdent {
        ident: syn::Ident,
    },
    /// Something to be evaluated as rust.
    Eval {
        expr: syn::Expr,
    },
    /// A bound scope.
    Scope {
        binding: Option<syn::Ident>,
        content: TokenStream,
    },
    /// A loop repetition.
    Loop {
        /// The pattern being bound.
        pattern: syn::Pat,
        /// Expression being bound to an iterator.
        expr: syn::Expr,
        /// If a join is specified, this is the token stream used to join.
        /// It's evaluated in the loop scope.
        join: Option<TokenStream>,
        /// The inner stream processed.
        stream: TokenStream,
    },
    Condition {
        /// Expression being use as a condition.
        condition: syn::Expr,
        /// Then branch of the conditional.
        then_branch: TokenStream,
        /// Else branch of the conditional.
        else_branch: Option<TokenStream>,
    },
    Match {
        condition: syn::Expr,
        arms: Vec<MatchArm>,
    },
}
