//! Helper to parse quoted strings.

use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token;
use syn::Result;

/// More detailed information on the quoted section.
#[derive(Default)]
pub(crate) struct Options {
    /// If the section has any evaluated sections.
    pub(crate) has_eval: bool,
}

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

struct Encoder<'a> {
    receiver: &'a syn::Ident,
    cursor: Option<LineColumn>,
    span: Span,
    count: usize,
    buffer: String,
    stream: TokenStream,
    options: Options,
}

impl<'a> Encoder<'a> {
    pub fn new(receiver: &'a syn::Ident, cursor: LineColumn, span: Span) -> Self {
        Self {
            receiver,
            cursor: Some(cursor),
            span,
            count: 0,
            buffer: String::new(),
            stream: TokenStream::new(),
            options: Options::default(),
        }
    }

    pub(crate) fn finalize(mut self, end: LineColumn) -> Result<(Options, TokenStream)> {
        self.flush(Some(end), None)?;
        Ok((self.options, self.stream))
    }

    /// Encode a single character and replace the cursor with the given
    /// location.
    pub(crate) fn encode_char(&mut self, c: char, from: LineColumn, to: LineColumn) -> Result<()> {
        self.flush_whitespace(Some(from), Some(to))?;
        self.buffer.push(c);
        self.cursor = Some(to);
        Ok(())
    }

    /// Encode a string directly to the static buffer as an optimization.
    pub(crate) fn encode_str(
        &mut self,
        s: &str,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush_whitespace(Some(from), to)?;
        self.buffer.push_str(s);
        Ok(())
    }

    /// Eval the given identifier.
    pub(crate) fn eval_ident(
        &mut self,
        ident: &syn::Ident,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush(Some(from), to)?;
        let receiver = self.receiver;

        let ident = syn::LitStr::new(&ident.to_string(), ident.span());

        self.stream.extend(q::quote! {
            #receiver.item(genco::tokens::Item::OpenEval);
            #receiver.item(genco::tokens::Item::Literal(genco::tokens::ItemStr::Static(#ident)));
            #receiver.item(genco::tokens::Item::CloseEval);
        });

        self.options.has_eval = true;
        Ok(())
    }

    /// Eval the given expression.
    pub(crate) fn eval_stream(
        &mut self,
        expr: TokenStream,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush(Some(from), to)?;
        let receiver = self.receiver;

        self.stream.extend(q::quote! {
            #receiver.item(genco::tokens::Item::OpenEval);
            #expr
            #receiver.item(genco::tokens::Item::CloseEval);
        });

        self.options.has_eval = true;
        Ok(())
    }

    /// Extend the content of the string with the given raw stream.
    pub(crate) fn raw_expr(
        &mut self,
        expr: &syn::Expr,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush(Some(from), to)?;
        let receiver = self.receiver;
        self.stream.extend(q::quote! {
            #receiver.append(#expr);
        });
        Ok(())
    }

    /// Extend the content of the string with the given raw identifier.
    pub(crate) fn raw_ident(
        &mut self,
        ident: &syn::Ident,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush(Some(from), to)?;
        let receiver = self.receiver;
        self.stream.extend(q::quote! {
            #receiver.append(#ident);
        });
        Ok(())
    }

    pub(crate) fn extend_tt(
        &mut self,
        tt: &TokenTree,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush_whitespace(Some(from), to)?;
        self.buffer.push_str(&tt.to_string());
        Ok(())
    }

    /// Flush the outgoing buffer.
    pub fn flush(&mut self, from: Option<LineColumn>, to: Option<LineColumn>) -> Result<()> {
        self.flush_whitespace(from, to)?;
        let receiver = self.receiver;

        if self.buffer.is_empty() {
            return Ok(());
        }

        let lit = syn::LitStr::new(&self.buffer, self.span);

        self.count += 1;

        self.stream.extend(q::quote! {
            #receiver.append(genco::tokens::ItemStr::Static(#lit));
        });

        self.buffer.clear();
        Ok(())
    }

    /// Flush the outgoing buffer.
    pub(crate) fn flush_whitespace(
        &mut self,
        from: Option<LineColumn>,
        to: Option<LineColumn>,
    ) -> Result<()> {
        if let (Some(from), Some(cursor)) = (from, self.cursor) {
            if cursor.line != from.line {
                return Err(syn::Error::new(
                    self.span,
                    "string interpolations may not contain line breaks",
                ));
            }

            for _ in 0..from.column.saturating_sub(cursor.column) {
                self.buffer.push(' ');
            }
        }

        self.cursor = to;
        Ok(())
    }
}

pub struct StringParser<'a> {
    receiver: &'a syn::Ident,
    start: LineColumn,
    end: LineColumn,
    span: Span,
}

impl<'a> StringParser<'a> {
    pub(crate) fn new(receiver: &'a syn::Ident, span: Span) -> Self {
        Self {
            receiver,
            // Note: adjusting span since we expect the quoted string to be
            // withing a block, where the interior span is one character pulled
            // in in each direction.
            start: adjust_start(span.start()),
            end: adjust_end(span.end()),
            span,
        }
    }

    pub(crate) fn parse(self, input: ParseStream) -> Result<(Options, TokenStream)> {
        let mut encoder = Encoder::new(self.receiver, self.start, self.span);

        while !input.is_empty() {
            if input.peek(syn::Token![$]) && input.peek2(syn::Token![$]) {
                let start = input.parse::<syn::Token![$]>()?;
                let escape = input.parse::<syn::Token![$]>()?;
                encoder.encode_char('$', start.span().start(), escape.span().end())?;
                continue;
            }

            if input.peek(syn::Token![#]) && input.peek2(syn::Token![#]) {
                let start = input.parse::<syn::Token![#]>()?;
                let escape = input.parse::<syn::Token![#]>()?;
                encoder.encode_char('#', start.span().start(), escape.span().end())?;
                continue;
            }

            if input.peek(syn::Token![$]) {
                let hash = input.parse::<syn::Token![$]>()?;

                if input.peek(token::Paren) {
                    let content;
                    let paren = syn::parenthesized!(content in input);
                    let stream = crate::quote::Quote::new(self.receiver)
                        .with_span(paren.span)
                        .parse(&content)?;
                    encoder.eval_stream(stream, hash.span().start(), Some(paren.span.end()))?;
                } else {
                    let ident = input.parse::<syn::Ident>()?;
                    encoder.eval_ident(&ident, hash.span().start(), Some(ident.span().end()))?;
                };

                continue;
            }

            if input.peek(syn::Token![#]) {
                let hash = input.parse::<syn::Token![#]>()?;

                if !input.peek(token::Paren) {
                    let ident = input.parse::<syn::Ident>()?;
                    encoder.raw_ident(&ident, hash.span().start(), Some(ident.span().end()))?;
                    continue;
                }

                let content;
                let paren = syn::parenthesized!(content in input);

                // Literal string optimization. A single, enclosed literal string can be added to the existing static buffer.
                if content.peek(syn::LitStr) && content.peek2(crate::token::Eof) {
                    let s = content.parse::<syn::LitStr>()?;
                    encoder.encode_str(&s.value(), hash.span().start(), Some(paren.span.end()))?;
                } else {
                    let expr = content.parse::<syn::Expr>()?;
                    encoder.raw_expr(&expr, hash.span().start(), Some(paren.span.end()))?;
                }

                continue;
            }

            let tt = input.parse::<TokenTree>()?;
            encoder.extend_tt(&tt, tt.span().start(), Some(tt.span().end()))?;
        }

        Ok(encoder.finalize(self.end)?)
    }
}
