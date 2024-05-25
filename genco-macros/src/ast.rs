use core::fmt;

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::Token;

use crate::static_buffer::StaticBuffer;

/// A single match arm in a match statement.
pub(crate) struct MatchArm {
    pub(crate) attr: Vec<syn::Attribute>,
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

pub(crate) enum LiteralName<'a> {
    /// The literal name as a string.
    Ident(&'a str),
    /// The literal name as a character.
    Char(char),
}

impl fmt::Display for LiteralName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            LiteralName::Ident(ident) => ident.fmt(f),
            LiteralName::Char(c) => write!(f, "{c:?}"),
        }
    }
}

/// The name of an internal fn.
pub(crate) enum Name {
    /// The name is the `const` token.
    Const(Token![const]),
    /// Custom name.
    Ident(String),
    /// Character name.
    Char(char),
}

impl Name {
    /// Get the name as a string.
    pub(crate) fn as_literal_name(&self) -> LiteralName<'_> {
        match self {
            Name::Const(..) => LiteralName::Ident("const"),
            Name::Ident(name) => LiteralName::Ident(name.as_str()),
            Name::Char(c) => LiteralName::Char(*c),
        }
    }
}

impl q::ToTokens for Name {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Name::Const(t) => t.to_tokens(tokens),
            Name::Ident(name) => name.to_tokens(tokens),
            Name::Char(c) => c.to_tokens(tokens),
        }
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

impl Control {
    /// Construct a control from a string identifier.
    pub(crate) fn from_char(span: Span, c: char) -> Option<Self> {
        match c {
            ' ' => Some(Self {
                kind: ControlKind::Space,
                span,
            }),
            '\n' => Some(Self {
                kind: ControlKind::Line,
                span,
            }),
            '\r' => Some(Self {
                kind: ControlKind::Push,
                span,
            }),
            _ => None,
        }
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
        pattern: Box<syn::Pat>,
        /// Expression being bound to an iterator.
        expr: Box<syn::Expr>,
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
    Let {
        /// Variable name (or names for a tuple)
        name: syn::Pat,
        /// Expression
        expr: syn::Expr,
    },
    Match {
        condition: syn::Expr,
        arms: Vec<MatchArm>,
    },
}
