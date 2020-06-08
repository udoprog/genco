use crate::{Cursor, ItemBuffer};
use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};
use syn::Result;

/// A delimiter that can be encoded.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
}

impl Delimiter {
    pub(crate) fn encode_start(self, output: &mut ItemBuffer) {
        let c = match self {
            Self::Parenthesis => '(',
            Self::Brace => '{',
            Self::Bracket => '[',
        };

        output.push(c);
    }

    pub(crate) fn encode_end(self, output: &mut ItemBuffer) {
        let c = match self {
            Self::Parenthesis => ')',
            Self::Brace => '}',
            Self::Bracket => ']',
        };

        output.push(c);
    }
}

/// An evaluated binding to the current token stream.
#[derive(Debug)]
pub(crate) struct Binding {
    pub(crate) binding: syn::Ident,
    pub(crate) binding_borrowed: bool,
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

/// Struct to deal with emitting the necessary spacing.
pub(crate) struct Encoder<'a> {
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
    // Used to determine the indentation state of a token.
    last_column: usize,
    /// TODO: make private.
    pub(crate) item_buffer: ItemBuffer<'a>,
    /// The token stream we are constructing.
    output: TokenStream,
    /// Currently stored cursor.
    cursor: Option<Cursor>,
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(
        receiver: &'a syn::Expr,
        span_start: Option<LineColumn>,
        span_end: Option<LineColumn>,
        last_column: usize,
    ) -> Self {
        Self {
            receiver,
            span_start,
            span_end,
            last_column,
            item_buffer: ItemBuffer::new(receiver),
            output: TokenStream::new(),
            cursor: None,
        }
    }

    /// Finalize and translate into a token stream.
    pub(crate) fn into_output(mut self) -> TokenStream {
        self.finalize();
        self.output
    }

    pub(crate) fn set_current(&mut self, cursor: Cursor) {
        self.cursor = Some(cursor);
    }

    // Reset cursor, so that registers don't count as items to be offset from.
    // This allows imports to be grouped without affecting formatting.
    pub(crate) fn reset(&mut self) {
        self.cursor = None;
    }

    pub(crate) fn step(&mut self, next: Cursor) {
        let mut next_start = next.start;

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
            if next_start.line == span_start.line {
                self.last_column = span_start.column;
                next_start = span_start;
            }
        }

        // Insert spacing if appropriate.
        self.tokenize_whitespace_until(next_start);

        // Assign the current cursor to the next item.
        // This will then be used to make future indentation decisions.
        self.cursor = Some(next);
    }

    pub(crate) fn encode_tree(&mut self, tt: TokenTree) {
        self.item_buffer.push_str(&tt.to_string());
    }

    pub(crate) fn encode_control(&mut self, control: Control) {
        let receiver = self.receiver;
        self.item_buffer.flush(&mut self.output);

        match control.kind {
            ControlKind::Space => {
                self.output
                    .extend(quote::quote_spanned!(control.span => #receiver.space();));
            }
            ControlKind::Push => {
                self.output
                    .extend(quote::quote_spanned!(control.span => #receiver.push();));
            }
            ControlKind::Line => {
                self.output
                    .extend(quote::quote_spanned!(control.span => #receiver.line();));
            }
        }
    }

    pub(crate) fn encode_eval_binding(&mut self, binding: Binding, stmt: TokenTree) {
        let Binding {
            binding,
            binding_borrowed,
        } = binding;

        let receiver = self.receiver;

        self.item_buffer.flush(&mut self.output);

        // If the receiver is borrowed, we need to reborrow to
        // satisfy the borrow checker in case it's in a loop.
        let binding = if binding_borrowed {
            quote::quote_spanned!(binding.span() => let #binding = &mut *#receiver;)
        } else {
            quote::quote_spanned!(binding.span() => let #binding = &mut #receiver;)
        };

        self.output.extend(quote::quote! {{
            #binding
            #stmt
        }});
    }

    /// Encode an evaluation of the given expression.
    pub(crate) fn encode_eval(&mut self, stmt: TokenTree) {
        let receiver = self.receiver;
        self.item_buffer.flush(&mut self.output);
        self.output.extend(quote::quote! {
            #receiver.append(#stmt);
        });
    }

    pub(crate) fn encode_repeat(
        &mut self,
        pattern: syn::Pat,
        expr: syn::Expr,
        join: Option<TokenStream>,
        stream: TokenStream,
    ) {
        self.item_buffer.flush(&mut self.output);

        if let Some(join) = join {
            self.output.extend(quote::quote! {
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
            self.output.extend(quote::quote! {
                for #pattern in #expr {
                    #stream
                }
            });
        }
    }

    /// Finalize the encoder.
    fn finalize(&mut self) {
        // evaluate whitespace in case we have an explicit end span.
        while let Some(end) = self.span_end.take() {
            if let Some(span_start) = self.span_start.take() {
                if end.line == span_start.line {
                    self.last_column = span_start.column;
                }
            }

            // Insert spacing if appropriate, up until the "fake" end.
            self.tokenize_whitespace_until(end);
        }

        self.item_buffer.flush(&mut self.output);
    }

    /// Insert indentation and spacing if appropriate in the output token stream.
    fn tokenize_whitespace_until(&mut self, next: LineColumn) {
        // Do nothing unless we have a cursor.
        let cursor = match &self.cursor {
            Some(cursor) => cursor,
            None => return,
        };

        let receiver = self.receiver;

        // Insert spacing if we are on the same line, but column has changed.
        if cursor.end.line == next.line {
            // Same line, but next item doesn't match.
            if cursor.end.column < next.column && self.last_column != next.column {
                self.item_buffer.flush(&mut self.output);
                self.output.extend(quote::quote!(#receiver.space();));
            }

            return;
        }

        // Line changed. Determine whether to indent, unindent, or hard break the
        // line.
        self.item_buffer.flush(&mut self.output);

        debug_assert!(next.line > cursor.start.line);

        let line_spaced = if next.line - cursor.end.line > 1 {
            self.output.extend(quote::quote!(#receiver.line();));
            true
        } else {
            false
        };

        if self.last_column < next.column {
            self.output.extend(quote::quote!(#receiver.indent();));
        } else if self.last_column > next.column {
            self.output.extend(quote::quote!(#receiver.unindent();));
        } else if !line_spaced {
            self.output.extend(quote::quote!(#receiver.push();));
        }

        self.last_column = next.column;
    }
}
