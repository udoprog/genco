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

pub(crate) struct MatchArm {
    pub(crate) pattern: syn::Pat,
    pub(crate) condition: Option<syn::Expr>,
    pub(crate) block: TokenStream,
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
    /// TODO: make private.
    pub(crate) item_buffer: ItemBuffer<'a>,
    /// The token stream we are constructing.
    output: TokenStream,
    /// Currently stored cursor.
    last: Option<Cursor>,
    /// Which column the last line start on.
    last_start_column: Option<usize>,
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(
        receiver: &'a syn::Expr,
        span_start: Option<LineColumn>,
        span_end: Option<LineColumn>,
    ) -> Self {
        Self {
            receiver,
            span_start,
            span_end,
            item_buffer: ItemBuffer::new(receiver),
            output: TokenStream::new(),
            last: None,
            last_start_column: None,
        }
    }

    /// Finalize and translate into a token stream.
    pub(crate) fn into_output(mut self) -> TokenStream {
        self.finalize();
        self.output
    }

    pub(crate) fn set_current(&mut self, last: Cursor) {
        self.last = Some(last);
    }

    // Reset cursor, so that registers don't count as items to be offset from.
    // This allows imports to be grouped without affecting formatting.
    pub(crate) fn reset(&mut self) {
        self.last = None;
    }

    pub(crate) fn step(&mut self, next: Cursor) {
        if let Some(from) = self.from() {
            // Insert spacing if appropriate.
            self.tokenize_whitespace(from, next.start);
        }

        // Assign the current cursor to the next item.
        // This will then be used to make future indentation decisions.
        self.last = Some(next);
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

    /// Encode an if statement with an inner stream.
    pub(crate) fn encode_condition(
        &mut self,
        condition: syn::Expr,
        then_branch: TokenStream,
        else_branch: Option<TokenStream>,
    ) {
        self.item_buffer.flush(&mut self.output);

        let else_branch = else_branch.map(|stream| quote::quote!(else { #stream }));

        self.output.extend(quote::quote! {
            if #condition { #then_branch } #else_branch
        });
    }

    /// Encode an if statement with an inner stream.
    pub(crate) fn encode_match(&mut self, condition: syn::Expr, arms: Vec<MatchArm>) {
        self.item_buffer.flush(&mut self.output);

        let mut stream = TokenStream::new();

        for MatchArm {
            pattern,
            condition,
            block,
        } in arms
        {
            let condition = condition.map(|c| quote::quote!(if #c));
            stream.extend(quote::quote!(#pattern #condition => { #block },));
        }

        let m = quote::quote! {
            match #condition { #stream }
        };

        self.output.extend(m);
    }

    fn from(&mut self) -> Option<LineColumn> {
        // So we've (potentially) encountered the first ever token, while we
        // have a spanned start like `quote_in! { out => foo }`, `foo` is now
        // `next`.
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
            self.last_start_column = Some(span_start.column);
            return Some(span_start);
        }

        if let Some(last) = self.last {
            if self.last_start_column.is_none() {
                self.last_start_column = Some(last.start.column);
            }

            return Some(last.end);
        }

        None
    }

    /// Finalize the encoder.
    fn finalize(&mut self) {
        // evaluate whitespace in case we have an explicit end span.
        while let Some(to) = self.span_end.take() {
            if let Some(from) = self.from() {
                // Insert spacing if appropriate, up until the "fake" end.
                self.tokenize_whitespace(from, to);
            }
        }

        self.item_buffer.flush(&mut self.output);
    }

    /// Insert indentation and spacing if appropriate in the output token stream.
    fn tokenize_whitespace(&mut self, from: LineColumn, to: LineColumn) {
        let receiver = self.receiver;

        // Do nothing if empty span.
        if from == to {
            return;
        }

        // Insert spacing if we are on the same line, but column has changed.
        if from.line == to.line {
            // Same line, but next item doesn't match.
            if from.column < to.column {
                self.item_buffer.flush(&mut self.output);
                self.output.extend(quote::quote!(#receiver.space();));
            }

            return;
        }

        // Line changed. Determine whether to indent, unindent, or hard break the
        // line.
        self.item_buffer.flush(&mut self.output);

        debug_assert!(from.line < to.line);

        let line_spaced = if to.line - from.line > 1 {
            self.output.extend(quote::quote!(#receiver.line();));
            true
        } else {
            false
        };

        if let Some(last_start_column) = self.last_start_column.take() {
            if last_start_column < to.column {
                self.output.extend(quote::quote!(#receiver.indent();));
            } else if last_start_column > to.column {
                self.output.extend(quote::quote!(#receiver.unindent();));
            } else if !line_spaced {
                self.output.extend(quote::quote!(#receiver.push();));
            }
        }
    }
}
