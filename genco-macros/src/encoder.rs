use crate::ast::{Ast, Control, ControlKind, Delimiter, MatchArm};
use crate::cursor::Cursor;
use crate::fake::LineColumn;
use crate::requirements::Requirements;
use crate::static_buffer::StaticBuffer;
use crate::Ctxt;

use proc_macro2::{Span, TokenStream};
use syn::Result;

/// Struct to deal with emitting the necessary spacing.
pub(crate) struct Encoder<'a> {
    /// Context for encoding.
    cx: &'a Ctxt,
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
    item_buffer: StaticBuffer<'a>,
    /// The token stream we are constructing.
    output: TokenStream,
    /// Currently stored cursor.
    last: Option<Cursor>,
    /// Which column the last line start on.
    last_start_column: Option<usize>,
    /// Indentation columns.
    indents: Vec<(usize, Option<Span>)>,
    /// Indicates if the encoder has encountered a string which requires eval
    /// support in the target language.
    pub(crate) requirements: Requirements,
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(
        cx: &'a Ctxt,
        span_start: Option<LineColumn>,
        span_end: Option<LineColumn>,
    ) -> Self {
        Self {
            cx,
            span_start,
            span_end,
            item_buffer: StaticBuffer::new(cx),
            output: TokenStream::new(),
            last: None,
            last_start_column: None,
            indents: Vec::new(),
            requirements: Requirements::default(),
        }
    }

    /// Encode a single item into the encoder.
    pub(crate) fn encode(&mut self, cursor: Cursor, ast: Ast) -> Result<()> {
        self.step(cursor)?;

        match ast {
            Ast::Tree { tt, .. } => {
                self.encode_literal(&tt.to_string());
            }
            Ast::String { has_eval, stream } => {
                self.requirements.lang_supports_eval |= has_eval;
                self.encode_string(has_eval, stream);
            }
            Ast::Quoted { s } => {
                self.encode_quoted(s);
            }
            Ast::Literal { string } => {
                self.encode_literal(&string);
            }
            Ast::Control { control, .. } => {
                self.encode_control(control);
            }
            Ast::Scope {
                binding, content, ..
            } => {
                self.encode_scope(binding, content);
            }
            Ast::EvalIdent { ident } => {
                self.encode_eval_ident(ident);
            }
            Ast::Eval { expr, .. } => {
                self.encode_eval(expr);
            }
            Ast::Loop {
                pattern,
                expr,
                join,
                stream,
                ..
            } => {
                self.encode_repeat(*pattern, *expr, join, stream);
            }
            Ast::DelimiterOpen { delimiter, .. } => {
                self.encode_open_delimiter(delimiter);
            }
            Ast::DelimiterClose { delimiter, .. } => {
                self.encode_close_delimiter(delimiter);
            }
            Ast::Condition {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.encode_condition(condition, then_branch, else_branch);
            }
            Ast::Match {
                condition, arms, ..
            } => {
                self.encode_match(condition, arms);
            }
            Ast::Let { name, expr } => {
                self.encode_let(name, expr);
            }
        }

        Ok(())
    }

    /// Finalize and translate into a token stream.
    pub(crate) fn into_output(mut self) -> Result<(Requirements, TokenStream)> {
        self.finalize()?;
        Ok((self.requirements, self.output))
    }

    pub(crate) fn step(&mut self, next: Cursor) -> Result<()> {
        if let Some(from) = self.from() {
            // Insert spacing if appropriate.
            self.tokenize_whitespace(from, next.start, Some(next.span))?;
        }

        // Assign the current cursor to the next item.
        // This will then be used to make future indentation decisions.
        self.last = Some(next);
        Ok(())
    }

    pub(crate) fn encode_open_delimiter(&mut self, d: Delimiter) {
        d.encode_open(&mut self.item_buffer);
    }

    pub(crate) fn encode_close_delimiter(&mut self, d: Delimiter) {
        d.encode_close(&mut self.item_buffer);
    }

    pub(crate) fn encode_literal(&mut self, string: &str) {
        self.item_buffer.push_str(string);
    }

    pub(crate) fn encode_string(&mut self, has_eval: bool, stream: TokenStream) {
        let Ctxt { receiver, module } = self.cx;

        self.item_buffer.flush(&mut self.output);

        self.output.extend(q::quote! {
            #receiver.append(#module::tokens::Item::OpenQuote(#has_eval));
            #stream
            #receiver.append(#module::tokens::Item::CloseQuote);
        });
    }

    pub(crate) fn encode_quoted(&mut self, s: syn::LitStr) {
        let Ctxt { receiver, module } = self.cx;

        self.item_buffer.flush(&mut self.output);

        self.output.extend(q::quote! {
            #receiver.append(#module::tokens::Item::OpenQuote(false));
            #receiver.append(#module::tokens::ItemStr::Static(#s));
            #receiver.append(#module::tokens::Item::CloseQuote);
        });
    }

    pub(crate) fn encode_control(&mut self, control: Control) {
        let Ctxt { receiver, .. } = self.cx;

        self.item_buffer.flush(&mut self.output);

        match control.kind {
            ControlKind::Space => {
                self.output
                    .extend(q::quote_spanned!(control.span => #receiver.space();));
            }
            ControlKind::Push => {
                self.output
                    .extend(q::quote_spanned!(control.span => #receiver.push();));
            }
            ControlKind::Line => {
                self.output
                    .extend(q::quote_spanned!(control.span => #receiver.line();));
            }
        }
    }

    pub(crate) fn encode_scope(&mut self, binding: Option<syn::Ident>, content: TokenStream) {
        let Ctxt { receiver, .. } = self.cx;

        if binding.is_some() {
            self.item_buffer.flush(&mut self.output);
        }

        let binding = binding.map(|b| q::quote_spanned!(b.span() => let #b = &mut *#receiver;));

        self.output.extend(q::quote! {{
            #binding
            #content
        }});
    }

    /// Encode an evaluation of the given expression.
    pub(crate) fn encode_eval_ident(&mut self, ident: syn::Ident) {
        let Ctxt { receiver, .. } = self.cx;

        self.item_buffer.flush(&mut self.output);
        self.output.extend(q::quote! {
            #receiver.append(#ident);
        });
    }

    /// Encode an evaluation of the given expression.
    pub(crate) fn encode_eval(&mut self, expr: syn::Expr) {
        let Ctxt { receiver, .. } = self.cx;

        self.item_buffer.flush(&mut self.output);
        self.output.extend(q::quote! {
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
            self.output.extend(q::quote! {
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
            self.output.extend(q::quote! {
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

        let else_branch = else_branch.map(|stream| q::quote!(else { #stream }));

        self.output.extend(q::quote! {
            if #condition { #then_branch } #else_branch
        });
    }

    /// Encode an if statement with an inner stream.
    pub(crate) fn encode_match(&mut self, condition: syn::Expr, arms: Vec<MatchArm>) {
        self.item_buffer.flush(&mut self.output);

        let mut stream = TokenStream::new();

        for MatchArm {
            attr,
            pattern,
            condition,
            block,
        } in arms
        {
            let condition = condition.map(|c| q::quote!(if #c));
            stream.extend(q::quote!(#(#attr)* #pattern #condition => { #block },));
        }

        let m = q::quote! {
            match #condition { #stream }
        };

        self.output.extend(m);
    }

    /// Encode a let statement
    pub(crate) fn encode_let(&mut self, name: syn::Pat, expr: syn::Expr) {
        self.item_buffer.flush(&mut self.output);

        self.output.extend(q::quote! {
            let #name = #expr;
        })
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
        let Ctxt { receiver, .. } = self.cx;

        // evaluate whitespace in case we have an explicit end span.
        while let Some(to) = self.span_end.take() {
            if let Some(from) = self.from() {
                // Insert spacing if appropriate, up until the "fake" end.
                self.tokenize_whitespace(from, to, None)?;
            }
        }

        self.item_buffer.flush(&mut self.output);

        while self.indents.pop().is_some() {
            self.output.extend(q::quote!(#receiver.unindent();));
        }

        Ok(())
    }

    /// If we are in a nightly genco, insert indentation and spacing if
    /// appropriate in the output token stream.
    fn tokenize_whitespace(
        &mut self,
        from: LineColumn,
        to: LineColumn,
        to_span: Option<Span>,
    ) -> Result<()> {
        let Ctxt { receiver: r, .. } = self.cx;

        // Do nothing if empty span.
        if from == to {
            return Ok(());
        }

        // Insert spacing if we are on the same line, but column has changed.
        if from.line == to.line {
            // Same line, but next item doesn't match.
            if from.column < to.column {
                self.item_buffer.flush(&mut self.output);
                self.output.extend(q::quote!(#r.space();));
            }

            return Ok(());
        }

        // Line changed. Determine whether to indent, unindent, or hard break the
        // line.
        self.item_buffer.flush(&mut self.output);

        debug_assert!(from.line < to.line);

        let line = to.line - from.line > 1;

        if let Some(last_start_column) = self.last_start_column.take() {
            if last_start_column < to.column {
                self.indents.push((last_start_column, to_span));
                self.output.extend(q::quote!(#r.indent();));

                if line {
                    self.output.extend(q::quote!(#r.line();));
                }
            } else if last_start_column > to.column {
                while let Some((column, _)) = self.indents.pop() {
                    if column > to.column && !self.indents.is_empty() {
                        self.output.extend(q::quote!(#r.unindent();));

                        if line {
                            self.output.extend(q::quote!(#r.line();));
                        }

                        continue;
                    } else if column == to.column {
                        self.output.extend(q::quote!(#r.unindent();));

                        if line {
                            self.output.extend(q::quote!(#r.line();));
                        }

                        break;
                    }

                    return Err(indentation_error(to.column, column, to_span));
                }
            } else if line {
                self.output.extend(q::quote!(#r.line();));
            } else {
                self.output.extend(q::quote!(#r.push();));
            }
        }

        return Ok(());

        fn indentation_error(
            to_column: usize,
            from_column: usize,
            to_span: Option<Span>,
        ) -> syn::Error {
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

            if let Some(span) = to_span {
                syn::Error::new(span, error)
            } else {
                syn::Error::new(Span::call_site(), error)
            }
        }
    }
}
