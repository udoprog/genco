use proc_macro2::{self as pc, LineColumn, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::VecDeque;
use syn::parse::{ParseStream, Parser as _};
use syn::spanned::Spanned;
use syn::token;
use syn::{Result, Token};

use crate::string_parser::StringParser;
use crate::{Control, Cursor, Delimiter, Encoder, MatchArm};

/// Items to process from the queue.
enum Item {
    /// A raw token tree.
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

struct QueueItem {
    pub(crate) cursor: Cursor,
    pub(crate) item: Item,
    pub(crate) span: Span,
}

impl QueueItem {
    pub fn with_span(span: Span, cursor: Cursor, item: Item) -> Self {
        Self { span, cursor, item }
    }
}

pub(crate) struct QuoteParser<'a> {
    /// Used to set the receiver identifier which is being modified by this
    /// macro.
    receiver: &'a syn::Ident,
    /// Use to modify the initial line/column in case something was processed
    /// before the input was handed off to the quote parser.
    ///
    /// See [QuoteInParser].
    span_start: Option<LineColumn>,
    /// Override the end span of the quote parser.
    ///
    /// This causes encoder to be emitted at the tail of the expression,
    /// unless it specifically reached the end of the span.
    span_end: Option<LineColumn>,
}

impl<'a> QuoteParser<'a> {
    /// Construct a new quote parser.
    pub(crate) fn new(receiver: &'a syn::Ident) -> Self {
        Self {
            receiver,
            span_start: None,
            span_end: None,
        }
    }

    /// Override the default starting span.
    pub(crate) fn with_span(self, span: Span) -> Self {
        return Self {
            span_start: Some(adjust_start(span.start())),
            span_end: Some(adjust_end(span.end())),
            ..self
        };

        fn adjust_start(start: LineColumn) -> LineColumn {
            LineColumn {
                line: start.line,
                column: start.column + 1,
            }
        }

        fn adjust_end(end: LineColumn) -> LineColumn {
            LineColumn {
                line: end.line,
                column: end.column.saturating_sub(1),
            }
        }
    }

    /// Parse until you've reached the given peek token.
    pub(crate) fn parse_until(
        self,
        input: ParseStream,
        until: impl Fn(ParseStream) -> bool + Copy,
    ) -> Result<TokenStream> {
        self.parse_internal(input, until)
    }

    /// Parse until end of stream.
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        self.parse_internal(input, |_| false)
    }

    fn parse_internal(
        self,
        input: ParseStream,
        until: impl Fn(ParseStream) -> bool + Copy,
    ) -> Result<TokenStream> {
        let receiver = self.receiver;

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut encoder = Encoder::new(self.receiver, self.span_start, self.span_end);

        parse_inner(|item| queue.push_back(item), input, receiver, until)?;

        while let Some(item) = queue.pop_front() {
            item.cursor.check_compat()?;

            encoder.step(item.cursor, item.span)?;

            match item.item {
                // Parse inner groups. Since the delimiters aren't "real", we
                // need to deal with this separately.
                Item::Tree {
                    tt: TokenTree::Group(group),
                    ..
                } => {
                    let delimiter = match group.delimiter() {
                        pc::Delimiter::Parenthesis => Some(Delimiter::Parenthesis),
                        pc::Delimiter::Brace => Some(Delimiter::Brace),
                        pc::Delimiter::Bracket => Some(Delimiter::Bracket),
                        _ => None,
                    };

                    if let Some(d) = delimiter {
                        parse_tree_iterator(|item| queued.push(item), group.stream(), receiver)?;

                        let cursor = Cursor::from(group.span());
                        encoder.encode_start_delimiter(d);

                        // Add an item marker so that we encode the delimiter at
                        // the end.
                        queue.push_front(QueueItem::with_span(
                            group.span(),
                            cursor.end_character(),
                            Item::DelimiterClose { delimiter: d },
                        ));

                        // We've only officially processed one character, so
                        // deal with it here.
                        encoder.set_current(cursor.start_character());

                        while let Some(item) = queued.pop() {
                            queue.push_front(item);
                        }
                    } else {
                        parse_tree_iterator(
                            |item| queue.push_back(item),
                            group.stream(),
                            receiver,
                        )?;
                    }
                }
                Item::Tree { tt, .. } => {
                    encoder.encode_literal(&tt.to_string());
                }
                Item::String { has_eval, stream } => {
                    encoder.encode_string(has_eval, stream);
                }
                Item::Quoted { s } => {
                    encoder.encode_quoted(s);
                }
                Item::Literal { string } => {
                    encoder.encode_literal(&string);
                }
                Item::Control { control, .. } => {
                    encoder.encode_control(control);
                }
                Item::Scope {
                    binding, content, ..
                } => {
                    encoder.encode_scope(binding, content);
                }
                Item::EvalIdent { ident } => {
                    encoder.encode_eval_ident(ident);
                }
                Item::Eval { expr, .. } => {
                    encoder.encode_eval(expr);
                }
                Item::Loop {
                    pattern,
                    expr,
                    join,
                    stream,
                    ..
                } => {
                    encoder.encode_repeat(pattern, expr, join, stream);
                }
                Item::DelimiterClose { delimiter, .. } => {
                    encoder.encode_end_delimiter(delimiter);
                }
                Item::Condition {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    encoder.encode_condition(condition, then_branch, else_branch);
                }
                Item::Match {
                    condition, arms, ..
                } => {
                    encoder.encode_match(condition, arms);
                }
            }
        }

        let output = encoder.into_output()?;

        Ok(quote::quote! {
            #output
        })
    }
}

/// Process expressions in the token stream.
fn parse_tree_iterator(
    queue: impl FnMut(QueueItem),
    stream: TokenStream,
    receiver: &syn::Ident,
) -> Result<()> {
    let parser = |input: ParseStream| parse_inner(queue, input, receiver, |_| false);
    parser.parse2(stream)?;
    Ok(())
}

/// Parse `if <condition> { <quoted> } [else { <quoted> }]`.
fn parse_condition(input: ParseStream, receiver: &syn::Ident) -> Result<Item> {
    input.parse::<Token![if]>()?;
    let condition = syn::Expr::parse_without_eager_brace(input)?;

    if input.peek(Token![=>]) {
        input.parse::<Token![=>]>()?;
        let then_branch = QuoteParser::new(receiver).parse(input)?;

        return Ok(Item::Condition {
            condition,
            then_branch,
            else_branch: None,
        });
    }

    let content;
    syn::braced!(content in input);

    let then_branch = QuoteParser::new(receiver).parse(&content)?;

    let else_branch = if input.peek(Token![else]) {
        input.parse::<Token![else]>()?;

        let content;
        syn::braced!(content in input);

        Some(QuoteParser::new(receiver).parse(&content)?)
    } else {
        None
    };

    Ok(Item::Condition {
        condition,
        then_branch,
        else_branch,
    })
}

/// Parse `for <expr> in <iter> [join (<quoted>)] => <quoted>`.
fn parse_loop(input: ParseStream, receiver: &syn::Ident) -> Result<Item> {
    syn::custom_keyword!(join);

    input.parse::<Token![for]>()?;
    let pattern = input.parse::<syn::Pat>()?;
    input.parse::<Token![in]>()?;
    let expr = syn::Expr::parse_without_eager_brace(input)?;

    let join = if input.peek(join) {
        input.parse::<join>()?;

        let content;
        let paren = syn::parenthesized!(content in input);

        let parser = QuoteParser::new(receiver).with_span(paren.span);

        Some(parser.parse(&content)?)
    } else {
        None
    };

    let content;

    let input = if input.peek(Token![=>]) {
        input.parse::<Token![=>]>()?;
        input
    } else {
        syn::braced!(content in input);
        &content
    };

    let parser = QuoteParser::new(receiver);
    let stream = parser.parse(&input)?;

    return Ok(Item::Loop {
        pattern,
        join,
        expr,
        stream,
    });
}

fn parse_match(input: ParseStream, receiver: &syn::Ident) -> Result<Item> {
    input.parse::<Token![match]>()?;
    let condition = syn::Expr::parse_without_eager_brace(input)?;

    let body;
    syn::braced!(body in input);

    let mut arms = Vec::new();

    while !body.is_empty() {
        let pattern = body.parse::<syn::Pat>()?;

        let condition = if body.peek(Token![if]) {
            body.parse::<Token![if]>()?;
            let condition = body.parse::<syn::Expr>()?;
            Some(condition)
        } else {
            None
        };

        body.parse::<Token![=>]>()?;

        let block = if body.peek(token::Brace) {
            let block;
            syn::braced!(block in body);

            let parser = QuoteParser::new(receiver);
            parser.parse(&block)?
        } else {
            let parser = QuoteParser::new(receiver);
            parser.parse_until(&body, |s| s.peek(Token![,]))?
        };

        arms.push(MatchArm {
            pattern,
            condition,
            block,
        });

        if body.peek(Token![,]) {
            body.parse::<Token![,]>()?;
        }
    }

    Ok(Item::Match { condition, arms })
}

/// Parse evaluation: `[*]<binding> => <expr>`.
fn parse_scope(input: ParseStream) -> Result<Item> {
    input.parse::<Token![ref]>()?;

    let binding = if input.peek(Token![_]) {
        input.parse::<Token![_]>()?;
        None
    } else {
        Some(input.parse()?)
    };

    let content;

    let content = if input.peek(token::Brace) {
        syn::braced!(content in input);
        &content
    } else {
        input.parse::<Token![=>]>()?;
        input
    };

    Ok(Item::Scope {
        binding,
        content: content.parse()?,
    })
}

fn parse_expression(input: ParseStream, receiver: &syn::Ident) -> Result<QueueItem> {
    let span = input.span();
    let hash = input.parse::<Token![#]>()?;
    let start = hash.span;

    // Single identifier without quoting.
    if !input.peek(token::Paren) {
        let ident = input.parse::<syn::Ident>()?;
        let cursor = Cursor::join(start, ident.span());

        return Ok(QueueItem::with_span(
            span,
            cursor,
            Item::EvalIdent { ident },
        ));
    }

    let scope;
    let outer = syn::parenthesized!(scope in input);

    let cursor = Cursor::join(start, outer.span);

    let item = if scope.peek(Token![if]) {
        parse_condition(&scope, receiver)?
    } else if scope.peek(Token![for]) {
        parse_loop(&scope, receiver)?
    } else if scope.peek(Token![match]) {
        parse_match(&scope, receiver)?
    } else if scope.peek(Token![ref]) {
        parse_scope(&scope)?
    } else if scope.peek(syn::LitStr) && scope.peek2(crate::token::Eof) {
        let string = scope.parse::<syn::LitStr>()?.value();

        Item::Literal { string }
    } else {
        Item::Eval {
            expr: scope.parse()?,
        }
    };

    Ok(QueueItem::with_span(span, cursor, item))
}

fn parse_inner(
    mut queue: impl FnMut(QueueItem),
    input: ParseStream,
    receiver: &syn::Ident,
    until: impl Fn(ParseStream) -> bool + Copy,
) -> Result<()> {
    syn::custom_punctuation!(Escape, ##);
    syn::custom_punctuation!(ControlStart, #<);

    while !input.is_empty() && !until(input) {
        // Escape sequence.
        if input.peek(Escape) {
            let escape = input.parse::<Escape>()?;
            let cursor = Cursor::join(escape.spans[0], escape.spans[1]);
            let mut punct = Punct::new('#', Spacing::Joint);
            punct.set_span(escape.spans[1]);
            queue(QueueItem::with_span(
                escape.span(),
                cursor,
                Item::Tree { tt: punct.into() },
            ));
            continue;
        }

        if input.peek(syn::Token![#]) && input.peek2(syn::Token![_]) && input.peek3(token::Paren) {
            let start = input.parse::<syn::Token![#]>()?;
            input.parse::<syn::Token![_]>()?;

            let content;
            let paren = syn::parenthesized!(content in input);

            let parser = StringParser::new(receiver, paren.span);

            let (options, stream) = parser.parse(&content)?;

            let cursor = Cursor::join(start.span(), paren.span);

            queue(QueueItem::with_span(
                content.span(),
                cursor,
                Item::String {
                    has_eval: options.has_eval,
                    stream,
                },
            ));
            continue;
        }

        // Control sequence.
        if input.peek(ControlStart) {
            let escape = input.parse::<ControlStart>()?;
            let control = input.parse::<Control>()?;
            let gt = input.parse::<token::Gt>()?;

            let cursor = Cursor::join(escape.span(), gt.span());
            queue(QueueItem::with_span(
                escape.span(),
                cursor,
                Item::Control { control },
            ));
            continue;
        }

        let start_expression = input.peek2(token::Paren) || input.peek2(syn::Ident);

        if input.peek(Token![#]) && start_expression {
            queue(parse_expression(input, receiver)?);
            continue;
        }

        if input.peek(syn::LitStr) {
            let s = input.parse::<syn::LitStr>()?;
            let cursor = Cursor::from(s.span());
            let span = s.span();
            queue(QueueItem::with_span(span, cursor, Item::Quoted { s }));
            continue;
        }

        let tt: TokenTree = input.parse()?;
        let cursor = Cursor::from(tt.span());
        let span = tt.span();

        queue(QueueItem::with_span(span, cursor, Item::Tree { tt }));
    }

    Ok(())
}
