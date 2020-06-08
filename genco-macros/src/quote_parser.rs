use proc_macro2::{Delimiter, Group, LineColumn, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::VecDeque;
use std::iter::FromIterator as _;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::spanned::Spanned;
use syn::token;
use syn::{Result, Token};

use crate::{Cursor, ItemBuffer, WhitespaceEmitter};

enum ControlKind {
    Space,
    Push,
    Line,
}

struct Control {
    kind: ControlKind,
    span: Span,
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

/// An evaluated binding to the current token stream.
struct Binding {
    binding: syn::Ident,
    binding_borrowed: bool,
}

/// Items to process from the queue.
enum Item {
    Tree(Cursor, TokenTree),
    Register(Cursor, TokenTree),
    DelimiterClose(Cursor, Delimiter),
    Control(Cursor, Control),
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
            Self::Tree(cursor, ..) => *cursor,
            Self::Register(cursor, ..) => *cursor,
            Self::DelimiterClose(cursor, ..) => *cursor,
            Self::Control(cursor, ..) => *cursor,
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
    /// This causes whitespace to be emitted at the tail of the expression,
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

        let mut output = TokenStream::new();

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut item_buffer = ItemBuffer::new(receiver);
        let mut whitespace = WhitespaceEmitter::new(
            self.receiver,
            self.span_start,
            self.span_end,
            input.span().start().column,
        );

        parse_inner(|item| queue.push_back(item), input, receiver)?;

        while let Some(item) = queue.pop_front() {
            whitespace.step(&mut output, &mut item_buffer, item.cursor());

            match item {
                Item::Tree(_, tt) => match tt {
                    TokenTree::Group(group) => {
                        parse_tree_iterator(
                            |item| queued.push(item),
                            group.stream().into_iter(),
                            receiver,
                        )?;

                        match group.delimiter() {
                            Delimiter::Parenthesis => item_buffer.push('('),
                            Delimiter::Brace => item_buffer.push('{'),
                            Delimiter::Bracket => item_buffer.push('['),
                            _ => (),
                        }

                        let span_cursor = Cursor::from(group.span());
                        queue.push_front(Item::DelimiterClose(
                            span_cursor.end_character(),
                            group.delimiter(),
                        ));
                        whitespace.set_current(span_cursor.start_character());

                        while let Some(item) = queued.pop() {
                            queue.push_front(item);
                        }
                    }
                    other => {
                        item_buffer.push_str(&other.to_string());
                    }
                },
                Item::Control(_, control) => {
                    item_buffer.flush(&mut output);

                    match control.kind {
                        ControlKind::Space => {
                            output
                                .extend(quote::quote_spanned!(control.span => #receiver.space();));
                        }
                        ControlKind::Push => {
                            output.extend(quote::quote_spanned!(control.span => #receiver.push();));
                        }
                        ControlKind::Line => {
                            output.extend(quote::quote_spanned!(control.span => #receiver.line();));
                        }
                    }
                }
                Item::Eval {
                    binding:
                        Some(Binding {
                            binding,
                            binding_borrowed,
                        }),
                    stmt,
                    ..
                } => {
                    item_buffer.flush(&mut output);

                    // If the receiver is borrowed, we need to reborrow to
                    // satisfy the borrow checker in case it's in a loop.
                    if binding_borrowed {
                        let binding = quote::quote_spanned!(binding.span() => let #binding = &mut *#receiver;);

                        output.extend(quote::quote! {{
                            #binding
                            #stmt
                        }});
                    } else {
                        let binding =
                            quote::quote_spanned!(binding.span() => let #binding = &mut #receiver;);

                        output.extend(quote::quote! {{
                            #binding
                            #stmt
                        }});
                    }
                }
                Item::Eval {
                    binding: None,
                    stmt,
                    ..
                } => {
                    item_buffer.flush(&mut output);
                    output.extend(quote::quote! {
                        #receiver.append(#stmt);
                    });
                }
                Item::Repeat {
                    pattern,
                    expr,
                    stream,
                    join,
                    ..
                } => {
                    item_buffer.flush(&mut output);

                    if let Some(join) = join {
                        output.extend(quote::quote! {
                            {
                                let mut __it = IntoIterator::into_iter(#expr).peekable();

                                while let Some(#pattern) = __it.next() {
                                    #stream

                                    if __it.peek().is_some() {
                                        #join
                                    }
                                }
                            }
                        });
                    } else {
                        output.extend(quote::quote! {
                            for #pattern in #expr {
                                #stream
                            }
                        });
                    }
                }
                Item::Register(_, expr) => {
                    registers.push(quote::quote_spanned!(expr.span() => #receiver.register(#expr)));
                    whitespace.reset();
                }
                Item::DelimiterClose(_, delimiter) => match delimiter {
                    Delimiter::Parenthesis => item_buffer.push(')'),
                    Delimiter::Brace => item_buffer.push('}'),
                    Delimiter::Bracket => item_buffer.push(']'),
                    _ => (),
                },
            }
        }

        whitespace.end(&mut output, &mut item_buffer);

        item_buffer.flush(&mut output);

        Ok(quote::quote! {
            #(#registers;)*
            #output
        })
    }
}

/// Process expressions in the token stream.
fn parse_tree_iterator(
    queue: impl FnMut(Item),
    it: impl Iterator<Item = TokenTree>,
    receiver: &syn::Expr,
) -> Result<()> {
    let parser = |input: ParseStream| parse_inner(queue, input, receiver);

    let stream = TokenStream::from_iter(it);
    parser.parse2(stream)?;
    Ok(())
}

fn parse_register(start: Span, input: ParseStream) -> Result<Item> {
    let (cursor, inner) = if input.peek(token::Paren) {
        let content;
        let delim = syn::parenthesized!(content in input);

        let mut group = Group::new(Delimiter::None, content.parse()?);
        group.set_span(delim.span);

        let cursor = Cursor::join(start, delim.span);
        (cursor, TokenTree::Group(group))
    } else {
        let ident = input.parse::<syn::Ident>()?;
        let cursor = Cursor::join(start, ident.span());
        (cursor, TokenTree::Ident(ident))
    };

    Ok(Item::Register(cursor, inner))
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

    let mut stmt = Group::new(Delimiter::None, scope.parse()?);
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
            let mut punct = Punct::new('#', Spacing::Joint);
            let cursor = Cursor::join(escape.spans[0], escape.spans[1]);
            punct.set_span(escape.spans[1]);
            queue(Item::Tree(cursor, TokenTree::Punct(punct)));
            continue;
        }

        // Control sequence.
        if input.peek(ControlStart) {
            let escape = input.parse::<ControlStart>()?;
            let control = input.parse::<Control>()?;
            let gt = input.parse::<token::Gt>()?;

            let cursor = Cursor::join(escape.span(), gt.span());
            queue(Item::Control(cursor, control));
            continue;
        }

        let start_expression = input.peek2(token::Paren) || input.peek2(syn::Ident);

        if input.peek(Token![#]) && start_expression {
            queue(parse_expression(input, receiver)?);
            continue;
        }

        let tt: TokenTree = input.parse()?;
        let cursor = Cursor::from(tt.span());
        queue(Item::Tree(cursor, tt));
    }

    Ok(())
}
