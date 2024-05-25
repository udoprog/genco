use proc_macro2::{Punct, Spacing, Span, TokenStream, TokenTree};
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::{token, Result, Token};

use crate::ast::{Ast, Control, Delimiter, LiteralName, MatchArm, Name};
use crate::encoder::Encoder;
use crate::fake::Buf;
use crate::fake::LineColumn;
use crate::requirements::Requirements;
use crate::string_parser::StringParser;
use crate::Ctxt;

pub(crate) struct Quote<'a> {
    /// Context variables.
    cx: &'a Ctxt,
    /// Use to modify the initial line/column in case something was processed
    /// before the input was handed off to the quote parser.
    ///
    /// See [QuoteInParser].
    span_start: Option<LineColumn>,
    /// Override the end span of the quote parser.
    ///
    /// This causes encoder to be emitted at the tail of the expression,
    /// unless it specifically reached the end of the span.
    span_end: Option<LineColumn>,
    /// If true, only parse until a comma (`,`) is encountered.
    until_comma: bool,
    /// Buffer,
    buf: Buf,
}

impl<'a> Quote<'a> {
    /// Construct a new quote parser.
    pub(crate) fn new(cx: &'a Ctxt) -> Self {
        Self {
            cx,
            span_start: None,
            span_end: None,
            until_comma: false,
            buf: Buf::default(),
        }
    }

    /// Construct a new quote parser that will only parse until the given token.
    pub(crate) fn new_until_comma(cx: &'a Ctxt) -> Self {
        Self {
            cx,
            span_start: None,
            span_end: None,
            until_comma: true,
            buf: Buf::default(),
        }
    }

    /// Override the default starting span.
    pub(crate) fn with_span(mut self, span: Span) -> syn::Result<Self> {
        return Ok(Self {
            span_start: Some(adjust_start(self.buf.start(span)?)),
            span_end: Some(adjust_end(self.buf.end(span)?)),
            ..self
        });

        fn adjust_start(start: LineColumn) -> LineColumn {
            LineColumn {
                line: start.line,
                column: start.column.saturating_add(1),
            }
        }

        fn adjust_end(end: LineColumn) -> LineColumn {
            LineColumn {
                line: end.line,
                column: end.column.saturating_sub(1),
            }
        }
    }

    /// Parse until end of stream.
    pub(crate) fn parse(mut self, input: ParseStream) -> Result<(Requirements, TokenStream)> {
        let mut encoder = Encoder::new(self.cx, self.span_start, self.span_end);
        self.parse_inner(&mut encoder, input, 0)?;
        encoder.into_output()
    }

    /// Parse `if <condition> { <quoted> } [else { <quoted> }]`.
    fn parse_condition(&self, input: ParseStream) -> Result<(Requirements, Ast)> {
        input.parse::<Token![if]>()?;
        let condition = syn::Expr::parse_without_eager_brace(input)?;

        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let (req, then_branch) = Quote::new(self.cx).parse(input)?;

            return Ok((
                req,
                Ast::Condition {
                    condition,
                    then_branch,
                    else_branch: None,
                },
            ));
        }

        let mut req = Requirements::default();

        let content;
        syn::braced!(content in input);

        let (r, then_branch) = Quote::new(self.cx).parse(&content)?;
        req.merge_with(r);

        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;

            let content;
            syn::braced!(content in input);

            let (r, else_branch) = Quote::new(self.cx).parse(&content)?;
            req.merge_with(r);

            Some(else_branch)
        } else {
            None
        };

        Ok((
            req,
            Ast::Condition {
                condition,
                then_branch,
                else_branch,
            },
        ))
    }

    /// Parse `for <expr> in <iter> [join (<quoted>)] => <quoted>`.
    fn parse_loop(&self, input: ParseStream) -> Result<(Requirements, Ast)> {
        syn::custom_keyword!(join);

        let mut req = Requirements::default();

        input.parse::<Token![for]>()?;
        let pattern = syn::Pat::parse_single(input)?;
        input.parse::<Token![in]>()?;
        let expr = syn::Expr::parse_without_eager_brace(input)?;

        let join = if input.peek(join) {
            input.parse::<join>()?;

            let content;
            let paren = syn::parenthesized!(content in input);

            let (r, join) = Quote::new(self.cx)
                .with_span(paren.span.span())?
                .parse(&content)?;
            req.merge_with(r);

            Some(join)
        } else {
            None
        };

        let content;

        let input = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            input
        } else {
            syn::braced!(content in input);
            &content
        };

        let parser = Quote::new(self.cx);
        let (r, stream) = parser.parse(input)?;
        req.merge_with(r);

        let ast = Ast::Loop {
            pattern: Box::new(pattern),
            join,
            expr: Box::new(expr),
            stream,
        };

        Ok((req, ast))
    }

    fn parse_match(&self, input: ParseStream) -> Result<(Requirements, Ast)> {
        input.parse::<Token![match]>()?;
        let condition = syn::Expr::parse_without_eager_brace(input)?;

        let body;
        syn::braced!(body in input);

        let mut req = Requirements::default();
        let mut arms = Vec::new();

        while !body.is_empty() {
            let attr = input.call(syn::Attribute::parse_outer)?;
            let pattern = syn::Pat::parse_multi_with_leading_vert(&body)?;

            let condition = if body.peek(Token![if]) {
                body.parse::<Token![if]>()?;
                let condition = body.parse::<syn::Expr>()?;
                Some(condition)
            } else {
                None
            };

            body.parse::<Token![=>]>()?;

            let (r, block) = if body.peek(token::Brace) {
                let block;
                syn::braced!(block in body);

                let parser = Quote::new(self.cx);
                parser.parse(&block)?
            } else if body.peek(token::Paren) {
                let block;
                let paren = syn::parenthesized!(block in body);

                Quote::new(self.cx)
                    .with_span(paren.span.span())?
                    .parse(&block)?
            } else {
                let parser = Quote::new_until_comma(self.cx);
                parser.parse(&body)?
            };

            req.merge_with(r);

            arms.push(MatchArm {
                attr,
                pattern,
                condition,
                block,
            });

            if body.peek(Token![,]) {
                body.parse::<Token![,]>()?;
            }
        }

        Ok((req, Ast::Match { condition, arms }))
    }

    fn parse_let(&self, input: ParseStream) -> Result<(Requirements, Ast)> {
        input.parse::<Token![let]>()?;

        let req = Requirements::default();

        let name = syn::Pat::parse_single(input)?;
        input.parse::<Token![=]>()?;
        let expr = syn::Expr::parse_without_eager_brace(input)?;

        let ast = Ast::Let { name, expr };

        Ok((req, ast))
    }

    /// Parse evaluation: `[*]<binding> => <expr>`.
    fn parse_scope(&self, input: ParseStream) -> Result<Ast> {
        input.parse::<Token![ref]>()?;

        let binding = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            None
        } else {
            Some(input.parse()?)
        };

        let content;

        let content = if input.peek(token::Brace) {
            syn::braced!(content in input);
            &content
        } else {
            input.parse::<Token![=>]>()?;
            input
        };

        Ok(Ast::Scope {
            binding,
            content: content.parse()?,
        })
    }

    fn parse_expression(&mut self, encoder: &mut Encoder, input: ParseStream) -> Result<()> {
        let start = input.parse::<Token![$]>()?.span();

        // Single identifier without quoting.
        if !input.peek(token::Paren) {
            let ident = input.parse::<syn::Ident>()?;
            let cursor = self.buf.join(start, ident.span())?;

            encoder.encode(cursor, Ast::EvalIdent { ident })?;
            return Ok(());
        }

        let scope;
        let outer = syn::parenthesized!(scope in input);

        let cursor = self.buf.join(start, outer.span.span())?;

        let ast = if scope.peek(Token![if]) {
            let (req, ast) = self.parse_condition(&scope)?;
            encoder.requirements.merge_with(req);
            ast
        } else if scope.peek(Token![for]) {
            let (req, ast) = self.parse_loop(&scope)?;
            encoder.requirements.merge_with(req);
            ast
        } else if scope.peek(Token![match]) {
            let (req, ast) = self.parse_match(&scope)?;
            encoder.requirements.merge_with(req);
            ast
        } else if scope.peek(Token![let]) {
            let (req, ast) = self.parse_let(&scope)?;
            encoder.requirements.merge_with(req);
            ast
        } else if scope.peek(Token![ref]) {
            self.parse_scope(&scope)?
        } else if crate::string_parser::is_lit_str_opt(scope.fork())? {
            let string = scope.parse::<syn::LitStr>()?.value();
            Ast::Literal { string }
        } else {
            Ast::Eval {
                expr: scope.parse()?,
            }
        };

        encoder.encode(cursor, ast)?;
        Ok(())
    }

    fn parse_inner(
        &mut self,
        encoder: &mut Encoder,
        input: ParseStream,
        group_depth: usize,
    ) -> Result<()> {
        while !input.is_empty() {
            if group_depth == 0 && self.until_comma && input.peek(Token![,]) {
                break;
            }

            // Escape sequence.
            if input.peek(Token![$]) && input.peek2(Token![$]) {
                let [a] = input.parse::<Token![$]>()?.spans;
                let [b] = input.parse::<Token![$]>()?.spans;

                let cursor = self.buf.join(a, b)?;
                let mut punct = Punct::new('$', Spacing::Joint);
                punct.set_span(cursor.span);
                encoder.encode(cursor, Ast::Tree { tt: punct.into() })?;
                continue;
            }

            if let Some((name, content, [start, end])) = parse_internal_function(input)? {
                match (name.as_literal_name(), content) {
                    (literal_name @ LiteralName::Ident("str"), None) => {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("Function `{literal_name}` expects content, like: $[{literal_name}](<content>)"),
                        ));
                    }
                    (LiteralName::Ident("str"), Some(content)) => {
                        let parser = StringParser::new(self.cx, &self.buf, end)?;

                        let (options, r, stream) = parser.parse(&content)?;
                        encoder.requirements.merge_with(r);

                        let cursor = self.buf.join(start, end)?;

                        encoder.encode(
                            cursor,
                            Ast::String {
                                has_eval: options.has_eval.get(),
                                stream,
                            },
                        )?;
                    }
                    (LiteralName::Char(c), content) => {
                        let control = match Control::from_char(name.span(), c) {
                            Some(control) => control,
                            None => {
                                return Err(syn::Error::new(name.span(), format!("Unsupported control {c:?}, expected one of: '\\n', '\r', ' '")));
                            }
                        };

                        if let Some(content) = content {
                            return Err(syn::Error::new(
                                content.span(),
                                format!("Control {c:?} does not expect an argument"),
                            ));
                        }

                        let cursor = self.buf.join(start.span(), end.span())?;
                        encoder.encode(cursor, Ast::Control { control })?;
                    }
                    (LiteralName::Ident(string), _) => {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("Unsupported function `{string}`, expected one of: str"),
                        ));
                    }
                }

                continue;
            }

            let start_expression = input.peek2(token::Paren) || input.peek2(syn::Ident);

            if input.peek(Token![$]) && start_expression {
                self.parse_expression(encoder, input)?;
                continue;
            }

            if input.peek(syn::LitStr) {
                let s = input.parse::<syn::LitStr>()?;
                let cursor = self.buf.cursor(s.span())?;
                encoder.encode(cursor, Ast::Quoted { s })?;
                continue;
            }

            // Test for different forms of groups and recurse if necessary.
            if input.peek(token::Brace) {
                let content;
                let braces = syn::braced!(content in input);
                self.parse_group(
                    encoder,
                    Delimiter::Brace,
                    braces.span.span(),
                    &content,
                    group_depth,
                )?;
                continue;
            }

            if input.peek(token::Paren) {
                let content;
                let braces = syn::parenthesized!(content in input);
                self.parse_group(
                    encoder,
                    Delimiter::Parenthesis,
                    braces.span.span(),
                    &content,
                    group_depth,
                )?;
                continue;
            }

            if input.peek(token::Bracket) {
                let content;
                let braces = syn::bracketed!(content in input);
                self.parse_group(
                    encoder,
                    Delimiter::Bracket,
                    braces.span.span(),
                    &content,
                    group_depth,
                )?;
                continue;
            }

            let tt: TokenTree = input.parse()?;
            let cursor = self.buf.cursor(tt.span())?;
            encoder.encode(cursor, Ast::Tree { tt })?;
        }

        Ok(())
    }

    fn parse_group(
        &mut self,
        encoder: &mut Encoder,
        delimiter: Delimiter,
        span: Span,
        input: ParseStream,
        group_depth: usize,
    ) -> Result<()> {
        let cursor = self.buf.cursor(span)?;

        encoder.encode(cursor.first_character(), Ast::DelimiterOpen { delimiter })?;

        self.parse_inner(encoder, input, group_depth + 1)?;

        encoder.encode(cursor.last_character(), Ast::DelimiterClose { delimiter })?;

        Ok(())
    }
}

/// Parse an internal function of the form:
///
/// ```text
/// $[<name>](<content>)
/// ```
///
/// The `(<content>)` part is optional, and if absent the internal function is
/// known as a "control function", like `$[' ']`.
pub(crate) fn parse_internal_function<'a>(
    input: &'a ParseBuffer,
) -> Result<Option<(Name, Option<ParseBuffer<'a>>, [Span; 2])>> {
    // Custom function call.
    if !(input.peek(Token![$]) && input.peek2(token::Bracket)) {
        return Ok(None);
    }

    let start = input.parse::<Token![$]>()?;

    let function;
    let brackets = syn::bracketed!(function in input);

    let name = if function.peek(Token![const]) {
        Name::Const(function.parse()?)
    } else if function.peek(syn::LitChar) {
        let c = function.parse::<syn::LitChar>()?;
        Name::Char(c.value())
    } else {
        let ident = function.parse::<syn::Ident>()?;
        Name::Ident(ident.to_string())
    };

    if !function.is_empty() {
        return Err(function.error("expected nothing after function identifier"));
    }

    let (content, end) = if input.peek(token::Paren) {
        let content;
        let paren = syn::parenthesized!(content in input);
        (Some(content), paren.span)
    } else {
        (None, brackets.span)
    };

    Ok(Some((name, content, [start.span(), end.span()])))
}
