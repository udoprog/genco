#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::{LineColumn, Span, Delimiter, Group, TokenTree, TokenStream};
use syn::{LitStr, parse_macro_input, Result};
use syn::parse::{ParseStream, Parse};
use std::iter::FromIterator;
use std::collections::VecDeque;

/// Quotes the specified expression as a stream of tokens for use with genco.
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
///
/// use genco::rust::imported;
/// use genco::{quote, Rust, Tokens};
///
/// // Import the LittleEndian item, without referencing it through the last
/// // module component it is part of.
/// let little_endian = imported("byteorder", "LittleEndian").qualified();
/// let big_endian = imported("byteorder", "BigEndian");
///
/// // This is a trait, so only import it into the scope (unless we intent to
/// // implement it).
/// let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
///
/// let tokens: Tokens<Rust> = quote! {
///     @write_bytes_ext
/// 
///     let mut wtr = vec![];
///     wtr.write_u16::<#little_endian>(517).unwrap();
///     wtr.write_u16::<#big_endian>(768).unwrap();
///     assert_eq!(wtr, vec![5, 2, 3, 0]);
/// };
///
/// println!("{}", tokens.to_file_string().unwrap());
/// ```
#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Tokens(registers, output) = parse_macro_input!(input as Tokens);

    let output = TokenStream::from_iter(output);

    let gen = quote::quote! {{
        let mut __toks = genco::Tokens::new();
        #(#registers;)*
        #output
        __toks
    }};

    gen.into()
}

struct Tokens(Vec<TokenStream>, Vec<TokenTree>);

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

        let mut registers = Vec::new();

        let mut tokens = Vec::new();

        let mut cursor = Cursor::from(input.span());
        let mut last_column = cursor.start.column;

        let mut queued = Vec::new();
        let mut queue = VecDeque::new();

        let mut item_buffer = ItemBuffer::new();

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
                item_buffer.flush(&mut tokens);

                debug_assert!(next.start.line > cursor.start.line);

                let stream = if next.start.line - cursor.start.line > 1 {
                    quote::quote!(__toks.line_spacing();)
                } else {
                    quote::quote!(__toks.push_spacing();)
                };

                tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));

                if last_column < next.start.column {
                    let stream = quote::quote!(__toks.indent(););
                    tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));
                } else if last_column > next.start.column {
                    let stream = quote::quote!(__toks.unindent(););
                    tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));
                }

                last_column = next.start.column;
            } else {
                // Same line, but next item doesn't match.
                if cursor.end.column < next.start.column && last_column != next.start.column {
                    item_buffer.flush(&mut tokens);

                    let stream = quote::quote!(__toks.spacing(););
                    tokens.push(TokenTree::Group(Group::new(Delimiter::None, stream)));
                }
            }

            cursor = next;

            match item {
                Item::Tree(tt) => {
                    match tt {
                        TokenTree::Group(group) => {
                            process_expressions(|item| queued.push(item), group.stream().into_iter().map(Ok))?;

                            match group.delimiter() {
                                Delimiter::Parenthesis => item_buffer.push('('),
                                Delimiter::Brace => item_buffer.push('{'),
                                Delimiter::Bracket => item_buffer.push('['),
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
                            item_buffer.push_str(&other.to_string());
                        }
                    }
                }
                Item::Expression(_, expr) => {
                    item_buffer.flush(&mut tokens);

                    let group = Group::new(Delimiter::None, quote::quote!(__toks.append(Clone::clone(&#expr));));
                    tokens.push(TokenTree::Group(group));
                }
                Item::Register(_, expr) => {
                    registers.push(quote::quote!(__toks.register(#expr)));
                }
                Item::DelimiterClose(_, delimiter) => {
                    match delimiter {
                        Delimiter::Parenthesis => item_buffer.push(')'),
                        Delimiter::Brace => item_buffer.push('}'), 
                        Delimiter::Bracket => item_buffer.push(']'),
                        _ => (),
                    }
                }
            }
        }

        item_buffer.flush(&mut tokens);
        Ok(Self(registers, tokens))
    }
}

struct ItemBuffer {
    buffer: String,
}

impl ItemBuffer {
    /// Construct a new line buffer.
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Push the given character to the line buffer.
    fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Push the given string to the line buffer.
    fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Flush the line buffer if necessary.
    fn flush(&mut self, tokens: &mut Vec<TokenTree>) {
        if !self.buffer.is_empty() {
            let s = LitStr::new(&self.buffer, Span::call_site());
            let group = Group::new(Delimiter::None, quote::quote!(__toks.append(#s);));
            tokens.push(TokenTree::Group(group));
            self.buffer.clear();
        }
    }
}

/// Items to process from the queue.
#[derive(Debug)]
enum Item {
    Tree(TokenTree),
    Expression(Cursor, TokenTree),
    Register(Cursor, TokenTree),
    DelimiterClose(Cursor, Delimiter),
}

impl Item {
    fn cursor(&self) -> Cursor {
        match self {
            Self::Tree(tt) => Cursor::from(tt.span()),
            Self::Expression(cursor, ..) => *cursor,
            Self::Register(cursor, ..) => *cursor,
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
            // Escape sequence for register.
            (TokenTree::Punct(mut a), Some(TokenTree::Punct(b))) if a.as_char() == '@' && b.as_char() == '@' => {
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
                    other => {
                        queue(Item::Expression(cursor, other));
                        it.next().transpose()?
                    }
                }
            }
            // Register evaluation.
            (TokenTree::Punct(first), Some(argument)) if first.as_char() == '@' => {
                let span = first.span().join(argument.span()).expect("failed to join spans");
                let cursor = Cursor::from(span);

                match argument {
                    other => {
                        queue(Item::Register(cursor, other));
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