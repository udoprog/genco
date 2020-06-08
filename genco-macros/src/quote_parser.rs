use proc_macro2::{self as pc, Group, LineColumn, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::VecDeque;
use syn::parse::{ParseStream, Parser as _};
use syn::spanned::Spanned;
use syn::token;
use syn::{Result, Token};

use crate::{Binding, Control, Cursor, Delimiter, Encoder};

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
    Repeat {
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
}

impl Item {
    pub(crate) fn cursor(&self) -> Cursor {
        match self {
            Self::Tree { cursor, .. } => *cursor,
            Self::Register { cursor, .. } => *cursor,
            Self::DelimiterClose { cursor, .. } => *cursor,
            Self::Control { cursor, .. } => *cursor,
            Self::Eval { cursor, .. } => *cursor,
            Self::Repeat { cursor, .. } => *cursor,
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

    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        let receiver = self.receiver;

        let mut registers = Vec::new();

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut encoder = Encoder::new(
            self.receiver,
            self.span_start,
            self.span_end,
            input.span().start().column,
        );

        parse_inner(|item| queue.push_back(item), input, receiver)?;

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
                        d.encode_start(&mut encoder.item_buffer);

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
                Item::Repeat {
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
                    delimiter.encode_end(&mut encoder.item_buffer);
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
    let parser = |input: ParseStream| parse_inner(queue, input, receiver);
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

fn parse_expression(input: ParseStream, receiver: &syn::Expr) -> Result<Item> {
    syn::custom_keyword!(join);

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

    // Repeat
    if scope.peek2(Token![in]) {
        let pattern = scope.parse::<syn::Pat>()?;
        scope.parse::<Token![in]>()?;
        let expr = scope.parse::<syn::Expr>()?;

        let join = if scope.peek(join) {
            scope.parse::<join>()?;

            let content;
            let paren = syn::parenthesized!(content in scope);

            // TODO: use end span.
            let parser = QuoteParser::new(receiver).with_span_end(paren.span.end());

            Some(parser.parse(&content)?)
        } else {
            None
        };

        scope.parse::<Token![=>]>()?;

        let parser = QuoteParser::new(receiver).with_span_start(pattern.span().start());

        let stream = parser.parse(&scope)?;

        return Ok(Item::Repeat {
            cursor,
            pattern,
            join,
            expr,
            stream,
        });
    }

    let binding_borrowed =
        if scope.peek(Token![*]) && scope.peek2(syn::Ident) && scope.peek3(Token![=>]) {
            scope.parse::<Token![*]>()?;
            true
        } else {
            false
        };

    let binding = if binding_borrowed || scope.peek(syn::Ident) && scope.peek2(Token![=>]) {
        let binding = scope.parse::<syn::Ident>()?;
        scope.parse::<Token![=>]>()?;

        Some(Binding {
            binding,
            binding_borrowed,
        })
    } else {
        None
    };

    let mut stmt = Group::new(pc::Delimiter::None, scope.parse()?);
    stmt.set_span(scope.span());

    Ok(Item::Eval {
        cursor,
        binding,
        stmt: stmt.into(),
    })
}

fn parse_inner(
    mut queue: impl FnMut(Item),
    input: ParseStream,
    receiver: &syn::Expr,
) -> Result<()> {
    syn::custom_punctuation!(Register, #@);
    syn::custom_punctuation!(Escape, ##);
    syn::custom_punctuation!(ControlStart, #<);

    while !input.is_empty() {
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
