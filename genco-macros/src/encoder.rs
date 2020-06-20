use crate::{Cursor, ItemBuffer};
use proc_macro2::{LineColumn, Span, TokenStream};
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
    receiver: &'a syn::Ident,
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
    item_buffer: ItemBuffer<'a>,
    /// The token stream we are constructing.
    output: TokenStream,
    /// Currently stored cursor.
    last: Option<Cursor>,
    /// Which column the last line start on.
    last_start_column: Option<usize>,
    /// Indentation columns.
    indents: Vec<(usize, Option<Span>)>,
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(
        receiver: &'a syn::Ident,
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
            indents: Vec::new(),
        }
    }

    /// Finalize and translate into a token stream.
    pub(crate) fn into_output(mut self) -> Result<TokenStream> {
        self.finalize()?;
        Ok(self.output)
    }

    pub(crate) fn set_current(&mut self, last: Cursor) {
        self.last = Some(last);
    }

    pub(crate) fn step(&mut self, next: Cursor, to_span: Span) -> Result<()> {
        if let Some(from) = self.from() {
            // Insert spacing if appropriate.
            self.tokenize_whitespace(from, next.start, Some(to_span))?;
        }

        // Assign the current cursor to the next item.
        // This will then be used to make future indentation decisions.
        self.last = Some(next);
        Ok(())
    }

    pub(crate) fn encode_start_delimiter(&mut self, d: Delimiter) {
        d.encode_start(&mut self.item_buffer);
    }

    pub(crate) fn encode_end_delimiter(&mut self, d: Delimiter) {
        d.encode_end(&mut self.item_buffer);
    }

    pub(crate) fn encode_literal(&mut self, string: &str) {
        self.item_buffer.push_str(string);
    }

    pub(crate) fn encode_string(&mut self, has_eval: bool, stream: TokenStream) {
        self.item_buffer.flush(&mut self.output);
        let receiver = self.receiver;

        self.output.extend(quote::quote! {
            #receiver.item(genco::tokens::Item::OpenQuote(#has_eval));
            #stream
            #receiver.item(genco::tokens::Item::CloseQuote);
        });
    }

    pub(crate) fn encode_quoted(&mut self, s: syn::LitStr) {
        let receiver = self.receiver;
        self.item_buffer.flush(&mut self.output);

        self.output.extend(quote::quote! {
            #receiver.item(genco::tokens::Item::OpenQuote(false));
            #receiver.append(genco::tokens::ItemStr::Static(#s));
            #receiver.item(genco::tokens::Item::CloseQuote);
        });
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

    pub(crate) fn encode_scope(&mut self, binding: Option<syn::Ident>, content: TokenStream) {
        let receiver = self.receiver;

        if binding.is_some() {
            self.item_buffer.flush(&mut self.output);
        }

        let binding = binding.map(|b| quote::quote_spanned!(b.span() => let #b = &mut *#receiver;));

        self.output.extend(quote::quote! {{
            #binding
            #content
        }});
    }

    /// Encode an evaluation of the given expression.
    pub(crate) fn encode_eval_ident(&mut self, ident: syn::Ident) {
        let receiver = self.receiver;
        self.item_buffer.flush(&mut self.output);
        self.output.extend(quote::quote! {
            #receiver.append(#ident);
        });
    }

    /// Encode an evaluation of the given expression.
    pub(crate) fn encode_eval(&mut self, expr: syn::Expr) {
        let receiver = self.receiver;
        self.item_buffer.flush(&mut self.output);
        self.output.extend(quote::quote! {
            #receiver.append(#expr);
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
    fn finalize(&mut self) -> Result<()> {
        // evaluate whitespace in case we have an explicit end span.
        while let Some(to) = self.span_end.take() {
            if let Some(from) = self.from() {
                // Insert spacing if appropriate, up until the "fake" end.
                self.tokenize_whitespace(from, to, None)?;
            }
        }

        self.item_buffer.flush(&mut self.output);

        let receiver = self.receiver;

        while let Some(_) = self.indents.pop() {
            self.output.extend(quote::quote!(#receiver.unindent();));
        }

        Ok(())
    }

    /// Insert indentation and spacing if appropriate in the output token stream.
    fn tokenize_whitespace(
        &mut self,
        from: LineColumn,
        to: LineColumn,
        to_span: Option<Span>,
    ) -> Result<()> {
        let r = self.receiver;

        // Do nothing if empty span.
        if from == to {
            return Ok(());
        }

        // Insert spacing if we are on the same line, but column has changed.
        if from.line == to.line {
            // Same line, but next item doesn't match.
            if from.column < to.column {
                self.item_buffer.flush(&mut self.output);
                self.output.extend(quote::quote!(#r.space();));
            }

            return Ok(());
        }

        // Line changed. Determine whether to indent, unindent, or hard break the
        // line.
        self.item_buffer.flush(&mut self.output);

        debug_assert!(from.line < to.line);

        let line = if to.line - from.line > 1 { true } else { false };

        if let Some(last_start_column) = self.last_start_column.take() {
            if last_start_column < to.column {
                self.indents.push((last_start_column, to_span));
                self.output.extend(quote::quote!(#r.indent();));

                if line {
                    self.output.extend(quote::quote!(#r.line();));
                }
            } else if last_start_column > to.column {
                while let Some((column, _)) = self.indents.pop() {
                    if column > to.column && !self.indents.is_empty() {
                        self.output.extend(quote::quote!(#r.unindent();));

                        if line {
                            self.output.extend(quote::quote!(#r.line();));
                        }

                        continue;
                    } else if column == to.column {
                        self.output.extend(quote::quote!(#r.unindent();));

                        if line {
                            self.output.extend(quote::quote!(#r.line();));
                        }

                        break;
                    }

                    return Err(indentation_error(to.column, column, to_span));
                }
            } else if line {
                self.output.extend(quote::quote!(#r.line();));
            } else {
                self.output.extend(quote::quote!(#r.push();));
            }
        }

        Ok(())
    }
}

fn indentation_error(to_column: usize, from_column: usize, to_span: Option<Span>) -> syn::Error {
    let error = if to_column > from_column {
        let len = to_column.saturating_sub(from_column);

        format!(
            "expected {} less {} of indentation",
            len,
            if len == 1 { "space" } else { "spaces" }
        )
    } else {
        let len = from_column.saturating_sub(to_column);

        format!(
            "expected {} more {} of indentation",
            len,
            if len == 1 { "space" } else { "spaces" }
        )
    };

    let error = if let Some(span) = to_span {
        syn::Error::new(span, error)
    } else {
        syn::Error::new(Span::call_site(), error)
    };

    error
}
