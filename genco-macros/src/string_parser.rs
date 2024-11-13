//! Helper to parse quoted strings.

use core::cell::{Cell, RefCell};
use core::fmt::Write;

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token;
use syn::Result;

use crate::ast::LiteralName;
use crate::fake::{Buf, LineColumn};
use crate::quote::parse_internal_function;
use crate::requirements::Requirements;
use crate::Ctxt;

/// Options for the parsed string.
#[derive(Default)]
pub(crate) struct Options {
    /// If the parsed string has any evaluation statements in it.
    pub(crate) has_eval: Cell<bool>,
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
    cx: &'a Ctxt,
    span: Span,
    cursor: Cell<Option<LineColumn>>,
    count: Cell<usize>,
    buf: RefCell<String>,
    stream: RefCell<TokenStream>,
    pub(crate) options: Options,
}

impl<'a> Encoder<'a> {
    pub fn new(cx: &'a Ctxt, cursor: LineColumn, span: Span) -> Self {
        Self {
            cx,
            span,
            cursor: Cell::new(Some(cursor)),
            count: Cell::new(0),
            buf: RefCell::new(String::new()),
            stream: RefCell::new(TokenStream::new()),
            options: Options::default(),
        }
    }

    pub(crate) fn finalize(self, end: LineColumn) -> Result<(Options, TokenStream)> {
        self.flush(Some(end), None)?;
        Ok((self.options, self.stream.into_inner()))
    }

    /// Encode a single character and replace the cursor with the given
    /// location.
    pub(crate) fn encode_char(&self, c: char, from: LineColumn, to: LineColumn) -> Result<()> {
        self.flush_whitespace(Some(from), Some(to))?;
        self.buf.borrow_mut().push(c);
        self.cursor.set(Some(to));
        Ok(())
    }

    /// Encode a string directly to the static buffer as an optimization.
    pub(crate) fn encode_str(
        &self,
        s: &str,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush_whitespace(Some(from), to)?;
        self.buf.borrow_mut().push_str(s);
        Ok(())
    }

    /// Eval the given identifier.
    pub(crate) fn eval_ident(
        &self,
        ident: &syn::Ident,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        let Ctxt { receiver, module } = self.cx;

        self.flush(Some(from), to)?;

        let ident = syn::LitStr::new(&ident.to_string(), ident.span());

        self.stream.borrow_mut().extend(q::quote! {
            #receiver.append(#module::tokens::Item::OpenEval);
            #receiver.append(#module::tokens::Item::Literal(#module::tokens::ItemStr::Static(#ident)));
            #receiver.append(#module::tokens::Item::CloseEval);
        });

        self.options.has_eval.set(true);
        Ok(())
    }

    /// Eval the given expression.
    pub(crate) fn eval_stream(
        &self,
        expr: TokenStream,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush(Some(from), to)?;

        let Ctxt { receiver, module } = self.cx;

        self.stream.borrow_mut().extend(q::quote! {
            #receiver.append(#module::tokens::Item::OpenEval);
            #expr
            #receiver.append(#module::tokens::Item::CloseEval);
        });

        self.options.has_eval.set(true);
        Ok(())
    }

    /// Extend the content of the string with the given raw stream.
    pub(crate) fn raw_expr(
        &self,
        expr: &syn::Expr,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush(Some(from), to)?;

        let Ctxt { receiver, .. } = self.cx;

        self.stream.borrow_mut().extend(q::quote! {
            #receiver.append(#expr);
        });
        Ok(())
    }

    pub(crate) fn extend_tt(
        &self,
        tt: &TokenTree,
        from: LineColumn,
        to: Option<LineColumn>,
    ) -> Result<()> {
        self.flush_whitespace(Some(from), to)?;
        write!(self.buf.borrow_mut(), "{tt}").unwrap();
        Ok(())
    }

    /// Flush the outgoing buffer.
    pub fn flush(&self, from: Option<LineColumn>, to: Option<LineColumn>) -> Result<()> {
        let Ctxt { receiver, module } = self.cx;

        self.flush_whitespace(from, to)?;

        let lit = {
            let buf = self.buf.borrow();

            if buf.is_empty() {
                return Ok(());
            }

            syn::LitStr::new(buf.as_str(), self.span)
        };

        self.count.set(self.count.get().wrapping_add(1));

        self.stream.borrow_mut().extend(q::quote! {
            #receiver.append(#module::tokens::ItemStr::Static(#lit));
        });

        self.buf.borrow_mut().clear();
        Ok(())
    }

    /// Flush the outgoing buffer.
    pub(crate) fn flush_whitespace(
        &self,
        from: Option<LineColumn>,
        to: Option<LineColumn>,
    ) -> Result<()> {
        if let (Some(from), Some(cursor)) = (from, self.cursor.get()) {
            if cursor.line != from.line {
                return Err(syn::Error::new(
                    self.span,
                    "string interpolations may not contain line breaks",
                ));
            }

            for _ in 0..from.column.saturating_sub(cursor.column) {
                self.buf.borrow_mut().push(' ');
            }
        }

        self.cursor.set(to);
        Ok(())
    }
}

pub struct StringParser<'a> {
    cx: &'a Ctxt,
    buf: &'a Buf,
    start: LineColumn,
    end: LineColumn,
    span: Span,
}

impl<'a> StringParser<'a> {
    pub(crate) fn new(cx: &'a Ctxt, buf: &'a Buf, span: Span) -> syn::Result<Self> {
        let cursor = buf.cursor(span)?;

        Ok(Self {
            cx,
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
        let encoder = Encoder::new(self.cx, self.start, self.span);

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
                            if is_lit_str_opt(content.fork())? {
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

                    let (req, stream) = crate::quote::Quote::new(self.cx)
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

pub(crate) fn is_lit_str_opt(content: ParseBuffer<'_>) -> syn::Result<bool> {
    if content.parse::<Option<syn::LitStr>>()?.is_none() {
        return Ok(false);
    }

    Ok(content.is_empty())
}
