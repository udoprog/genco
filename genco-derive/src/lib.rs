#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::{LineColumn, Span, Delimiter, Group, TokenTree, TokenStream};
use syn::{LitStr, parse_macro_input, Result};
use syn::parse::{ParseStream, Parse};
use std::iter::FromIterator;
use std::collections::VecDeque;

#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Tokens(output) = parse_macro_input!(input as Tokens);

    let output = TokenStream::from_iter(output);

    let gen = quote::quote! {{
        let mut __toks = genco::Tokens::new();
        #output
        __toks
    }};

    gen.into()
}

struct Tokens(Vec<TokenTree>);

#[derive(Clone, Copy, Debug)]
struct Cursor {
    start: LineColumn,
    end: LineColumn,
}

impl Cursor {
    /// Calculate the start character for the span.
    fn start_character(self) -> Self {
        Cursor {
            start: self.start,
            end: LineColumn {
                line: self.start.line,
                column: self.start.column + 1,
            },
        }
    }

    /// Calculate the end character for the span.
    fn end_character(self) -> Self {
        Cursor {
            start: LineColumn {
                line: self.end.line,
                column: self.end.column - 1,
            },
            end: self.end,
        }
    }
}

impl From<Span> for Cursor {
    fn from(span: Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl Parse for Tokens {
    fn parse(input: ParseStream) -> Result<Self> {
        use std::iter::from_fn;

        let mut tokens = Vec::new();

        let mut cursor = Cursor::from(input.span());
        let mut last_column = cursor.start.column;

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut line_buffer = String::new();

        process_expressions(|item| queue.push_back(item), from_fn(move || {
            if !input.is_empty() {
                Some(input.parse::<TokenTree>())
            } else {
                None
            }
        }))?;

        while let Some(item) = queue.pop_front() {
            let next = item.cursor();

            if cursor.start.line != next.start.line {
                if !line_buffer.is_empty() {
                    let s = LitStr::new(&line_buffer, Span::call_site());
                    let group = Group::new(Delimiter::None, quote::quote!(__toks.append(#s);));
                    tokens.push(TokenTree::Group(group));
                    line_buffer.clear();
                }

                debug_assert!(next.start.line > cursor.start.line);

                let stream = if next.start.line - cursor.start.line > 1 {
                    quote::quote!(__toks.append(genco::Element::LineSpacing);)
                } else {
                    quote::quote!(__toks.append(genco::Element::PushSpacing);)
                };

                tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));

                if last_column < next.start.column {
                    let stream = quote::quote!(__toks.append(genco::Element::Indent););
                    tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));
                } else if last_column > next.start.column {
                    let stream = quote::quote!(__toks.append(genco::Element::Unindent););
                    tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));
                }

                last_column = next.start.column;
            } else {
                if cursor.end.column < next.start.column && last_column != next.start.column {
                    line_buffer.push(' ');
                }
            }

            cursor = next;

            match item {
                Item::Tree(tt) => {
                    match tt {
                        TokenTree::Group(group) => {
                            process_expressions(|item| queued.push(item), group.stream().into_iter().map(Ok))?;

                            match group.delimiter() {
                                Delimiter::Parenthesis => line_buffer.push('('),
                                Delimiter::Brace => line_buffer.push('{'),
                                Delimiter::Bracket => line_buffer.push('['),
                                _ => (),
                            }

                            let span_cursor = Cursor::from(group.span());
                            queue.push_front(Item::DelimiterClose(span_cursor.end_character(), group.delimiter()));
                            cursor = span_cursor.start_character();

                            while let Some(item) = queued.pop() {
                                queue.push_front(item);
                            }
                        }
                        other => {
                            line_buffer.push_str(&other.to_string());
                        }
                    }
                }
                Item::Group(_, group) => {
                    if !line_buffer.is_empty() {
                        let s = LitStr::new(&line_buffer, Span::call_site());
                        let group = Group::new(Delimiter::None, quote::quote!(__toks.append(#s);));
                        tokens.push(TokenTree::Group(group));
                        line_buffer.clear();
                    }

                    let group = Group::new(Delimiter::None, quote::quote!(__toks.append(Clone::clone(&#group));));
                    tokens.push(TokenTree::Group(group));
                }
                Item::Expression(_, expr) => {
                    if !line_buffer.is_empty() {
                        let s = LitStr::new(&line_buffer, Span::call_site());
                        let group = Group::new(Delimiter::None, quote::quote!(__toks.append(#s);));
                        tokens.push(TokenTree::Group(group));
                        line_buffer.clear();
                    }

                    let group = Group::new(Delimiter::None, quote::quote!(__toks.append(Clone::clone(&#expr));));
                    tokens.push(TokenTree::Group(group));
                }
                Item::DelimiterClose(_, delimiter) => {
                    match delimiter {
                        Delimiter::Parenthesis => line_buffer.push(')'),
                        Delimiter::Brace => line_buffer.push('}'), 
                        Delimiter::Bracket => line_buffer.push(']'),
                        _ => (),
                    }
                }
            }
        }

        if !line_buffer.is_empty() {
            let s = LitStr::new(&line_buffer, Span::call_site());
            let group = Group::new(Delimiter::None, quote::quote!(__toks.append(#s);));
            tokens.push(TokenTree::Group(group));
            line_buffer.clear();
        }

        Ok(Self(tokens))
    }
}

/// Items to process from the queue.
#[derive(Debug)]
enum Item {
    Tree(TokenTree),
    Group(Cursor, TokenStream),
    Expression(Cursor, TokenTree),
    DelimiterClose(Cursor, Delimiter),
}

impl Item {
    fn cursor(&self) -> Cursor {
        match self {
            Self::Tree(tt) => Cursor::from(tt.span()),
            Self::Group(cursor, ..) => *cursor,
            Self::Expression(cursor, ..) => *cursor,
            Self::DelimiterClose(cursor, ..) => *cursor,
        }
    }
}

/// Process expressions in the token stream.
fn process_expressions(mut queue: impl FnMut(Item), mut it: impl Iterator<Item = Result<TokenTree>>) -> Result<()> {
    let mut n1 = it.next().transpose()?;

    while let Some(n0) = std::mem::replace(&mut n1, it.next().transpose()?) {
        n1 = match (n0, n1) {
            // Escape sequence for hash.
            (TokenTree::Punct(mut a), Some(TokenTree::Punct(b))) if a.as_char() == '#' && b.as_char() == '#' => {
                let span = a.span().join(b.span()).expect("failed to join spans");
                a.set_span(span);
                queue(Item::Tree(TokenTree::Punct(a)));
                it.next().transpose()?
            }
            // Context evaluation.
            (TokenTree::Punct(first), Some(argument)) if first.as_char() == '#' => {
                let span = first.span().join(argument.span()).expect("failed to join spans");
                let cursor = Cursor::from(span);

                match argument {
                    TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                        queue(Item::Group(cursor, group.stream()));
                        it.next().transpose()?
                    }
                    other => {
                        queue(Item::Expression(cursor, other));
                        it.next().transpose()?
                    }
                }
            }
            (tt, next) => {
                queue(Item::Tree(tt));
                next
            }
        }
    }

    if let Some(tt) = n1 {
        queue(Item::Tree(tt));
    }

    Ok(())
}