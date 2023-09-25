//! Helper to parse quoted strings.

use crate::ast::LiteralName;
use crate::fake::{Buf, LineColumn};
use crate::quote::parse_internal_function;
use crate::requirements::Requirements;

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token;
use syn::Result;

/// Options for the parsed string.
#[derive(Default, Clone, Copy)]
pub(crate) struct Options {
    /// If the parsed string has any evaluation statements in it.
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
    pub(crate) options: Options,
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
            #receiver.append(genco::tokens::Item::OpenEval);
            #receiver.append(genco::tokens::Item::Literal(genco::tokens::ItemStr::Static(#ident)));
            #receiver.append(genco::tokens::Item::CloseEval);
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
            #receiver.append(genco::tokens::Item::OpenEval);
            #expr
            #receiver.append(genco::tokens::Item::CloseEval);
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
    buf: &'a Buf,
    start: LineColumn,
    end: LineColumn,
    span: Span,
}

impl<'a> StringParser<'a> {
    pub(crate) fn new(receiver: &'a syn::Ident, buf: &'a Buf, span: Span) -> syn::Result<Self> {
        let cursor = buf.cursor(span)?;

        Ok(Self {
            receiver,
            buf,
            // Note: adjusting span since we expect the quoted string to be
            // withing a block, where the interior span is one character pulled
            // in in each direction.
            start: adjust_start(cursor.start),
            end: adjust_end(cursor.end),
            span,
        })
    }

    pub(crate) fn parse(self, input: ParseStream) -> Result<(Options, Requirements, TokenStream)> {
        let mut requirements = Requirements::default();
        let mut encoder = Encoder::new(self.receiver, self.start, self.span);

        while !input.is_empty() {
            if input.peek(syn::Token![$]) && input.peek2(syn::Token![$]) {
                let start = input.parse::<syn::Token![$]>()?;
                let escape = input.parse::<syn::Token![$]>()?;
                let start = self.buf.cursor(start.span())?;
                let escape = self.buf.cursor(escape.span())?;
                encoder.encode_char('$', start.start, escape.end)?;
                continue;
            }

            if input.peek(syn::Token![$]) {
                if let Some((name, content, [start, end])) = parse_internal_function(input)? {
                    match (name.as_literal_name(), content) {
                        (LiteralName::Ident("const"), Some(content)) => {
                            let start = self.buf.cursor(start)?;
                            let end = self.buf.cursor(end)?;

                            // Compile-time string optimization. A single,
                            // enclosed literal string can be added to the
                            // existing static buffer.
                            if content.peek(syn::LitStr) && content.peek2(crate::token::Eof) {
                                let s = content.parse::<syn::LitStr>()?;
                                encoder.encode_str(&s.value(), start.start, Some(end.end))?;
                            } else {
                                let expr = content.parse::<syn::Expr>()?;
                                encoder.raw_expr(&expr, start.start, Some(end.end))?;
                            }
                        }
                        (literal_name, _) => {
                            return Err(syn::Error::new(
                                name.span(),
                                format!(
                                    "Unsupported [str] function {literal_name}, expected one of: const"
                                ),
                            ));
                        }
                    }
                } else {
                    let dollar = input.parse::<syn::Token![$]>()?;
                    let [start] = dollar.spans;

                    if !input.peek(token::Paren) {
                        let ident = input.parse::<syn::Ident>()?;
                        let start = self.buf.cursor(start.span())?;
                        let end = self.buf.cursor(ident.span())?.end;
                        encoder.eval_ident(&ident, start.start, Some(end))?;
                        continue;
                    }

                    let content;
                    let end = syn::parenthesized!(content in input).span;

                    let (req, stream) = crate::quote::Quote::new(self.receiver)
                        .with_span(content.span())?
                        .parse(&content)?;
                    requirements.merge_with(req);
                    let start = self.buf.cursor(start.span())?;
                    let end = self.buf.cursor(end.span())?;
                    encoder.eval_stream(stream, start.start, Some(end.end))?;
                }

                continue;
            }

            let tt = input.parse::<TokenTree>()?;
            let cursor = self.buf.cursor(tt.span())?;
            encoder.extend_tt(&tt, cursor.start, Some(cursor.end))?;
        }

        let (options, stream) = encoder.finalize(self.end)?;
        Ok((options, requirements, stream))
    }
}
