use proc_macro2::{self as pc, Group, LineColumn, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::VecDeque;
use syn::parse::{ParseStream, Parser as _};
use syn::spanned::Spanned;
use syn::token;
use syn::{Result, Token};

use crate::{Binding, Control, Cursor, Delimiter, Encoder, MatchArm};

/// Items to process from the queue.
enum Item {
    Tree {
        cursor: Cursor,
        tt: TokenTree,
    },
    Register {
        cursor: Cursor,
        expr: syn::Expr,
    },
    DelimiterClose {
        cursor: Cursor,
        delimiter: Delimiter,
    },
    Control {
        cursor: Cursor,
        control: Control,
    },
    /// Something to be evaluated as rust.
    Eval {
        cursor: Cursor,
        binding: Option<Binding>,
        stmt: TokenTree,
    },
    /// A loop repetition.
    Loop {
        cursor: Cursor,
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
        cursor: Cursor,
        /// Expression being use as a condition.
        condition: syn::Expr,
        /// Then branch of the conditional.
        then_branch: TokenStream,
        /// Else branch of the conditional.
        else_branch: Option<TokenStream>,
    },
    Match {
        cursor: Cursor,
        condition: syn::Expr,
        arms: Vec<MatchArm>,
    },
}

impl Item {
    pub(crate) fn cursor(&self) -> Cursor {
        match self {
            Self::Tree { cursor, .. } => *cursor,
            Self::Register { cursor, .. } => *cursor,
            Self::DelimiterClose { cursor, .. } => *cursor,
            Self::Control { cursor, .. } => *cursor,
            Self::Eval { cursor, .. } => *cursor,
            Self::Loop { cursor, .. } => *cursor,
            Self::Condition { cursor, .. } => *cursor,
            Self::Match { cursor, .. } => *cursor,
        }
    }
}

pub(crate) struct QuoteParser<'a> {
    /// Used to set the receiver identifier which is being modified by this
    /// macro.
    receiver: &'a syn::Expr,
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
    pub(crate) fn new(receiver: &'a syn::Expr) -> Self {
        Self {
            receiver,
            span_start: None,
            span_end: None,
        }
    }

    /// Override the default starting span.
    pub(crate) fn with_span_start(self, span_start: LineColumn) -> Self {
        Self {
            span_start: Some(span_start),
            ..self
        }
    }

    /// Override the default ending span.
    pub(crate) fn with_span_end(self, span_end: LineColumn) -> Self {
        Self {
            span_end: Some(span_end),
            ..self
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

        let mut registers = Vec::new();

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut encoder = Encoder::new(self.receiver, self.span_start, self.span_end);

        parse_inner(|item| queue.push_back(item), input, receiver, until)?;

        while let Some(item) = queue.pop_front() {
            encoder.step(item.cursor());

            match item {
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
                        queue.push_front(Item::DelimiterClose {
                            cursor: cursor.end_character(),
                            delimiter: d,
                        });

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
                    encoder.encode_tree(tt);
                }
                Item::Control { control, .. } => {
                    encoder.encode_control(control);
                }
                Item::Eval {
                    binding: Some(binding),
                    stmt,
                    ..
                } => {
                    encoder.encode_eval_binding(binding, stmt);
                }
                Item::Eval {
                    binding: None,
                    stmt,
                    ..
                } => {
                    encoder.encode_eval(stmt);
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
                Item::Register { expr, .. } => {
                    registers.push(quote::quote_spanned!(expr.span() => #receiver.register(#expr)));
                    encoder.reset();
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

        let output = encoder.into_output();

        Ok(quote::quote! {
            #(#registers;)*
            #output
        })
    }
}

/// Process expressions in the token stream.
fn parse_tree_iterator(
    queue: impl FnMut(Item),
    stream: TokenStream,
    receiver: &syn::Expr,
) -> Result<()> {
    let parser = |input: ParseStream| parse_inner(queue, input, receiver, |_| false);
    parser.parse2(stream)?;
    Ok(())
}

fn parse_register(start: Span, input: ParseStream) -> Result<Item> {
    let (cursor, expr) = if input.peek(token::Paren) {
        let content;
        let delim = syn::parenthesized!(content in input);
        let expr = content.parse::<syn::Expr>()?;
        let cursor = Cursor::join(start, delim.span);
        (cursor, expr)
    } else {
        let expr = input.parse::<syn::Expr>()?;
        let cursor = Cursor::join(start, expr.span());
        (cursor, expr)
    };

    Ok(Item::Register { cursor, expr })
}

/// Parse `if <condition> { <quoted> } [else { <quoted> }]`.
fn parse_condition(cursor: Cursor, input: ParseStream, receiver: &syn::Expr) -> Result<Item> {
    input.parse::<Token![if]>()?;
    let condition = syn::Expr::parse_without_eager_brace(input)?;

    if input.peek(Token![=>]) {
        input.parse::<Token![=>]>()?;
        let then_branch = QuoteParser::new(receiver).parse(input)?;

        return Ok(Item::Condition {
            cursor,
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

    return Ok(Item::Condition {
        cursor,
        condition,
        then_branch,
        else_branch,
    });
}

/// Parse `for <expr> in <iter> [join (<quoted>)] => <quoted>`.
fn parse_loop(cursor: Cursor, input: ParseStream, receiver: &syn::Expr) -> Result<Item> {
    syn::custom_keyword!(join);

    input.parse::<Token![for]>()?;
    let pattern = input.parse::<syn::Pat>()?;
    input.parse::<Token![in]>()?;
    let expr = input.parse::<syn::Expr>()?;

    let join = if input.peek(join) {
        input.parse::<join>()?;

        let content;
        let paren = syn::parenthesized!(content in input);
        let parser = QuoteParser::new(receiver)
            .with_span_start(adjust_start(paren.span.start()))
            .with_span_end(adjust_end(paren.span.end()));

        Some(parser.parse(&content)?)
    } else {
        None
    };

    input.parse::<Token![=>]>()?;

    let parser = QuoteParser::new(receiver);
    let stream = parser.parse(&input)?;

    return Ok(Item::Loop {
        cursor,
        pattern,
        join,
        expr,
        stream,
    });

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

fn parse_match(cursor: Cursor, input: ParseStream, receiver: &syn::Expr) -> Result<Item> {
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

    return Ok(Item::Match {
        cursor,
        condition,
        arms,
    });
}

/// Parse evaluation: `[*]<binding> => <expr>`.
fn parse_eval(cursor: Cursor, input: ParseStream) -> Result<Item> {
    let binding_borrowed =
        if input.peek(Token![*]) && input.peek2(syn::Ident) && input.peek3(Token![=>]) {
            input.parse::<Token![*]>()?;
            true
        } else {
            false
        };

    let binding = if binding_borrowed || input.peek(syn::Ident) && input.peek2(Token![=>]) {
        let binding = input.parse::<syn::Ident>()?;
        input.parse::<Token![=>]>()?;

        Some(Binding {
            binding,
            binding_borrowed,
        })
    } else {
        None
    };

    let mut stmt = Group::new(pc::Delimiter::None, input.parse()?);
    stmt.set_span(input.span());

    Ok(Item::Eval {
        cursor,
        binding,
        stmt: stmt.into(),
    })
}

fn parse_expression(input: ParseStream, receiver: &syn::Expr) -> Result<Item> {
    let hash = input.parse::<Token![#]>()?;
    let start = hash.span;

    // Single identifier without quoting.
    if !input.peek(token::Paren) {
        let ident = input.parse::<syn::Ident>()?;
        let cursor = Cursor::join(start, ident.span());

        return Ok(Item::Eval {
            cursor,
            binding: None,
            stmt: TokenTree::Ident(ident),
        });
    }

    let scope;
    let outer = syn::parenthesized!(scope in input);

    let cursor = Cursor::join(start, outer.span);

    // If statement.
    if scope.peek(Token![if]) {
        return parse_condition(cursor, &scope, receiver);
    }

    // For loop.
    if scope.peek(Token![for]) && scope.peek3(Token![in]) {
        return parse_loop(cursor, &scope, receiver);
    }

    // Match.
    if scope.peek(Token![match]) {
        return parse_match(cursor, &scope, receiver);
    }

    parse_eval(cursor, &scope)
}

fn parse_inner(
    mut queue: impl FnMut(Item),
    input: ParseStream,
    receiver: &syn::Expr,
    until: impl Fn(ParseStream) -> bool + Copy,
) -> Result<()> {
    syn::custom_punctuation!(Register, #@);
    syn::custom_punctuation!(Escape, ##);
    syn::custom_punctuation!(ControlStart, #<);

    while !input.is_empty() && !until(input) {
        if input.peek(Register) {
            let register = input.parse::<Register>()?;
            queue(parse_register(register.spans[0], input)?);
            continue;
        }

        // Escape sequence.
        if input.peek(Escape) {
            let escape = input.parse::<Escape>()?;
            let cursor = Cursor::join(escape.spans[0], escape.spans[1]);
            let mut punct = Punct::new('#', Spacing::Joint);
            punct.set_span(escape.spans[1]);
            queue(Item::Tree {
                cursor,
                tt: punct.into(),
            });
            continue;
        }

        // Control sequence.
        if input.peek(ControlStart) {
            let escape = input.parse::<ControlStart>()?;
            let control = input.parse::<Control>()?;
            let gt = input.parse::<token::Gt>()?;

            let cursor = Cursor::join(escape.span(), gt.span());
            queue(Item::Control { cursor, control });
            continue;
        }

        let start_expression = input.peek2(token::Paren) || input.peek2(syn::Ident);

        if input.peek(Token![#]) && start_expression {
            queue(parse_expression(input, receiver)?);
            continue;
        }

        let tt: TokenTree = input.parse()?;
        let cursor = Cursor::from(tt.span());
        queue(Item::Tree { cursor, tt });
    }

    Ok(())
}
