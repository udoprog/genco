use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::VecDeque;
use std::iter::FromIterator as _;
use syn::parse::{ParseStream, Parser as _};
use syn::token;
use syn::{Ident, LitStr, Result, Token};

use crate::{Cursor, ItemBuffer};
/// Items to process from the queue.
#[derive(Debug)]
pub(crate) enum Item {
    Tree(Cursor, TokenTree),
    Expression(Cursor, TokenTree),
    Register(Cursor, TokenTree),
    DelimiterClose(Cursor, Delimiter),
    Repeat(Cursor, TokenTree, Option<TokenTree>),
}

impl Item {
    pub(crate) fn cursor(&self) -> Cursor {
        match self {
            Self::Tree(cursor, ..) => *cursor,
            Self::Expression(cursor, ..) => *cursor,
            Self::Register(cursor, ..) => *cursor,
            Self::DelimiterClose(cursor, ..) => *cursor,
            Self::Repeat(cursor, ..) => *cursor,
        }
    }
}

pub(crate) struct QuoteParser<'a> {
    pub(crate) receiver: &'a Ident,
}

impl QuoteParser<'_> {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        let receiver = self.receiver;

        let mut registers = Vec::new();

        let mut tokens = Vec::new();

        let mut cursor = None::<Cursor>;
        let mut last_column = input.span().start().column;

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut item_buffer = ItemBuffer::new(receiver);

        parse_expression(|item| queue.push_back(item), input)?;

        while let Some(item) = queue.pop_front() {
            let next = item.cursor();

            if let Some(cursor) = cursor {
                if cursor.start.line != next.start.line {
                    item_buffer.flush(&mut tokens);

                    debug_assert!(next.start.line > cursor.start.line);

                    let stream = if next.start.line - cursor.start.line > 1 {
                        quote::quote!(#receiver.line_spacing();)
                    } else {
                        quote::quote!(#receiver.push_spacing();)
                    };

                    tokens.extend(stream);

                    if last_column < next.start.column {
                        tokens.extend(quote::quote!(#receiver.indent();));
                    } else if last_column > next.start.column {
                        tokens.extend(quote::quote!(#receiver.unindent();));
                    }

                    last_column = next.start.column;
                } else {
                    // Same line, but next item doesn't match.
                    if cursor.end.column < next.start.column && last_column != next.start.column {
                        item_buffer.flush(&mut tokens);
                        tokens.extend(quote::quote!(#receiver.spacing();));
                    }
                }
            }

            // Assign the current cursor to the next item.
            // This can then be used to make future indentation decisions.
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
                    item_buffer.flush(&mut tokens);
                    tokens.extend(quote::quote_spanned!(expr.span() => #receiver.append(#expr);));
                }
                Item::Repeat(_, inner, separator) => {
                    item_buffer.flush(&mut tokens);

                    if let Some(separator) = separator {
                        let separator = LitStr::new(&separator.to_string(), separator.span());

                        tokens.extend(quote::quote! {{
                            let mut iter = std::iter::IntoIterator::into_iter(#inner).peekable();

                            while let Some(element) = iter.next() {
                                #receiver.append(element);

                                if iter.peek().is_some() {
                                    #receiver.append(genco::ItemStr::Static(#separator));
                                    #receiver.spacing();
                                }
                            }
                        }});
                    } else {
                        tokens.extend(quote::quote! {{
                            for element in #inner {
                                #receiver.append(element);
                            }
                        }});
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

        item_buffer.flush(&mut tokens);

        let tokens = TokenStream::from_iter(tokens);

        Ok(quote::quote! {
            #(#registers;)*
            #tokens
        })
    }
}

/// Process expressions in the token stream.
fn parse_tree_iterator(queue: impl FnMut(Item), it: impl Iterator<Item = TokenTree>) -> Result<()> {
    let parser = |input: ParseStream| parse_expression(queue, input);

    let stream = TokenStream::from_iter(it);
    parser.parse2(stream)?;
    Ok(())
}

fn parse_group(
    start: Span,
    input: ParseStream,
    can_repeat: bool,
    factory: fn(Cursor, TokenTree) -> Item,
) -> Result<Item> {
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

    if can_repeat {
        if input.peek2(Token![*]) {
            let separator = input.parse::<TokenTree>()?;
            let star = input.parse::<Token![*]>()?;
            let cursor = cursor.with_end(star.span.end());
            return Ok(Item::Repeat(cursor, inner, Some(separator)));
        }

        if input.peek(Token![*]) {
            let star = input.parse::<Token![*]>()?;
            let cursor = cursor.with_end(star.span.end());
            return Ok(Item::Repeat(cursor, inner, None));
        }
    }

    Ok(factory(cursor, inner))
}

fn parse_expression(mut queue: impl FnMut(Item), input: ParseStream) -> Result<()> {
    syn::custom_punctuation!(Register, #@);
    syn::custom_punctuation!(Escape, ##);

    while !input.is_empty() {
        if input.peek(Register) {
            let register = input.parse::<Register>()?;
            queue(parse_group(
                register.spans[0],
                input,
                false,
                Item::Register,
            )?);
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

        if input.peek(Token![#]) {
            let hash = input.parse::<Token![#]>()?;
            queue(parse_group(hash.span, input, true, Item::Expression)?);
            continue;
        }

        let tt: TokenTree = input.parse()?;
        let cursor = Cursor::from(tt.span());
        queue(Item::Tree(cursor, tt));
    }

    Ok(())
}
