use proc_macro2::{Delimiter, Group, LineColumn, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::VecDeque;
use std::iter::FromIterator as _;
use syn::parse::{ParseStream, Parser as _};
use syn::token;
use syn::{Expr, Ident, Result, Token};

use crate::{Cursor, ItemBuffer};
/// Items to process from the queue.
#[derive(Debug)]
pub(crate) enum Item {
    Tree(Cursor, TokenTree),
    Expression(Cursor, TokenTree),
    Register(Cursor, TokenTree),
    DelimiterClose(Cursor, Delimiter),
    /// A local scope which exposes the tokens being built as the specified
    /// variable.
    Scope {
        cursor: Cursor,
        binding: Ident,
        group: TokenTree,
        receiver_borrowed: bool,
    },
}

impl Item {
    pub(crate) fn cursor(&self) -> Cursor {
        match self {
            Self::Tree(cursor, ..) => *cursor,
            Self::Expression(cursor, ..) => *cursor,
            Self::Register(cursor, ..) => *cursor,
            Self::DelimiterClose(cursor, ..) => *cursor,
            Self::Scope { cursor, .. } => *cursor,
        }
    }
}

pub(crate) struct QuoteParser<'a> {
    /// Used to set the receiver identifier which is being modified by this
    /// macro.
    pub(crate) receiver: &'a Expr,
    /// Use to modify the initial line/column in case something was processed
    /// before the input was handed off to the quote parser.
    ///
    /// See [QuoteInParser].
    pub(crate) span_start: Option<LineColumn>,
}

impl QuoteParser<'_> {
    pub(crate) fn parse(mut self, input: ParseStream) -> Result<TokenStream> {
        let receiver = self.receiver;

        let mut registers = Vec::new();

        let mut output = TokenStream::new();

        // Keeping track of the span of the last token processed so we can
        // determine when to insert spacing or indentation.
        let mut cursor = None::<Cursor>;

        // Used to determine the indentation state of a token.
        let mut last_column = input.span().start().column;

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut item_buffer = ItemBuffer::new(receiver);

        parse_inner(|item| queue.push_back(item), input)?;

        while let Some(item) = queue.pop_front() {
            let mut next = item.cursor();

            // So we encountered the first ever token, while we have a spanned
            // start like `quote_in! { out => foo }`, `foo` is now `next`.
            //
            // What we want to do is treat the beginning out `out` as the
            // indentation position, so we adjust the token.
            //
            // But we also want to avoid situations like this:
            //
            // ```
            // quote_in! { out =>
            //     foo
            //     bar
            // }
            // ```
            //
            // If we would treat `out` as the start, `foo` would be seen as
            // unindented. So check if the first encountered token is on the
            // same line as the binding `out` or not before adjusting them!
            if let Some(span_start) = self.span_start.take() {
                if next.start.line == span_start.line {
                    last_column = span_start.column;
                    next = next.with_start(span_start);
                }
            }

            // Insert spacing if appropriate.
            handle_spacing(
                &mut output,
                receiver,
                &next,
                cursor.as_ref(),
                &mut last_column,
                &mut item_buffer,
            );

            // Assign the current cursor to the next item.
            // This will then be used to make future indentation decisions.
            cursor = Some(next);

            match item {
                Item::Tree(_, tt) => match tt {
                    TokenTree::Group(group) => {
                        parse_tree_iterator(|item| queued.push(item), group.stream().into_iter())?;

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
                        cursor = Some(span_cursor.start_character());

                        while let Some(item) = queued.pop() {
                            queue.push_front(item);
                        }
                    }
                    other => {
                        item_buffer.push_str(&other.to_string());
                    }
                },
                Item::Expression(_, expr) => {
                    item_buffer.flush(&mut output);
                    output.extend(quote::quote_spanned!(expr.span() => #receiver.append(#expr);));
                }
                Item::Scope {
                    binding,
                    group,
                    receiver_borrowed,
                    ..
                } => {
                    item_buffer.flush(&mut output);

                    // If the receiver is borrowed, we need to reborrow to
                    // satisfy the borrow checker in case it's in a loop.
                    if receiver_borrowed {
                        let binding = quote::quote_spanned!(binding.span() => let #binding = &mut *#receiver;);

                        output.extend(quote::quote! {
                            {
                                #binding
                                #group
                            }
                        });
                    } else {
                        let binding =
                            quote::quote_spanned!(binding.span() => let #binding = &mut #receiver;);

                        output.extend(quote::quote! {
                            {
                                #binding
                                #group
                            }
                        });
                    }
                }
                Item::Register(_, expr) => {
                    registers.push(quote::quote_spanned!(expr.span() => #receiver.register(#expr)));
                    // Reset cursor, so that registers don't count as items to be offset from.
                    // This allows imports to be grouped without affecting formatting.
                    cursor = None;
                }
                Item::DelimiterClose(_, delimiter) => match delimiter {
                    Delimiter::Parenthesis => item_buffer.push(')'),
                    Delimiter::Brace => item_buffer.push('}'),
                    Delimiter::Bracket => item_buffer.push(']'),
                    _ => (),
                },
            }
        }

        item_buffer.flush(&mut output);

        Ok(quote::quote! {
            #(#registers;)*
            #output
        })
    }
}

/// Insert indentation and spacing if appropriate in the output token stream.
fn handle_spacing(
    output: &mut TokenStream,
    receiver: &Expr,
    next: &Cursor,
    cursor: Option<&Cursor>,
    last_column: &mut usize,
    item_buffer: &mut ItemBuffer,
) {
    // Do nothing unless we have a cursor.
    let cursor = match cursor {
        Some(cursor) => cursor,
        None => return,
    };

    // Insert spacing if we are on the same line, but column has changed.
    if cursor.end.line == next.start.line {
        // Same line, but next item doesn't match.
        if cursor.end.column < next.start.column && *last_column != next.start.column {
            item_buffer.flush(output);
            output.extend(quote::quote!(#receiver.spacing();));
        }

        return;
    }

    // Line changed. Determine whether to indent, unindent, or hard break the
    // line.
    item_buffer.flush(output);

    debug_assert!(next.start.line > cursor.start.line);

    let line_spaced = if next.start.line - cursor.end.line > 1 {
        output.extend(quote::quote!(#receiver.push_line();));
        true
    } else {
        false
    };

    if *last_column < next.start.column {
        output.extend(quote::quote!(#receiver.indent();));
    } else if *last_column > next.start.column {
        output.extend(quote::quote!(#receiver.unindent();));
    } else if !line_spaced {
        output.extend(quote::quote!(#receiver.push();));
    }

    *last_column = next.start.column;
}

/// Process expressions in the token stream.
fn parse_tree_iterator(queue: impl FnMut(Item), it: impl Iterator<Item = TokenTree>) -> Result<()> {
    let parser = |input: ParseStream| parse_inner(queue, input);

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
        let ident = input.parse::<Ident>()?;
        let cursor = Cursor::join(start, ident.span());
        (cursor, TokenTree::Ident(ident))
    };

    Ok(Item::Register(cursor, inner))
}

fn parse_expression(start: Span, input: ParseStream) -> Result<Item> {
    if input.peek(token::Brace) {
        let scope;
        let outer_span = syn::braced!(scope in input);

        let receiver_borrowed = if scope.peek(Token![*]) {
            scope.parse::<Token![*]>()?;
            true
        } else {
            false
        };

        let binding = scope.parse::<Ident>()?;
        scope.parse::<Token![=>]>()?;

        let mut group = Group::new(Delimiter::None, scope.parse()?);
        group.set_span(scope.span());

        let cursor = Cursor::join(start, outer_span.span);

        return Ok(Item::Scope {
            cursor,
            binding,
            group: TokenTree::Group(group),
            receiver_borrowed,
        });
    }

    let (cursor, inner) = if input.peek(token::Paren) {
        let content;
        let delim = syn::parenthesized!(content in input);

        let mut group = Group::new(Delimiter::None, content.parse()?);
        group.set_span(delim.span);

        let cursor = Cursor::join(start, delim.span);
        (cursor, TokenTree::Group(group))
    } else {
        let ident = input.parse::<Ident>()?;
        let cursor = Cursor::join(start, ident.span());
        (cursor, TokenTree::Ident(ident))
    };

    Ok(Item::Expression(cursor, inner))
}

fn parse_inner(mut queue: impl FnMut(Item), input: ParseStream) -> Result<()> {
    syn::custom_punctuation!(Register, #@);
    syn::custom_punctuation!(Escape, ##);

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

        let start_expression =
            input.peek2(token::Brace) || input.peek2(token::Paren) || input.peek2(Ident);

        if input.peek(Token![#]) && start_expression {
            let hash = input.parse::<Token![#]>()?;
            queue(parse_expression(hash.span, input)?);
            continue;
        }

        let tt: TokenTree = input.parse()?;
        let cursor = Cursor::from(tt.span());
        queue(Item::Tree(cursor, tt));
    }

    Ok(())
}
