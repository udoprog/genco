use proc_macro2::{LineColumn, Punct, Spacing, Span, TokenStream, TokenTree};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token;
use syn::{Result, Token};

use crate::ast::{Ast, Control, Delimiter, MatchArm};
use crate::cursor::Cursor;
use crate::encoder::Encoder;
use crate::requirements::Requirements;
use crate::string_parser::StringParser;

pub(crate) struct Quote<'a> {
    /// Used to set the receiver identifier which is being modified by this
    /// macro.
    receiver: &'a syn::Ident,
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
}

impl<'a> Quote<'a> {
    /// Construct a new quote parser.
    pub(crate) fn new(receiver: &'a syn::Ident) -> Self {
        Self {
            receiver,
            span_start: None,
            span_end: None,
            until_comma: false,
        }
    }

    /// Construct a new quote parser that will only parse until the given token.
    pub(crate) fn new_until_comma(receiver: &'a syn::Ident) -> Self {
        Self {
            receiver,
            span_start: None,
            span_end: None,
            until_comma: true,
        }
    }

    /// Override the default starting span.
    pub(crate) fn with_span(self, span: Span) -> Self {
        return Self {
            span_start: Some(adjust_start(span.start())),
            span_end: Some(adjust_end(span.end())),
            ..self
        };

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
    }

    /// Parse until end of stream.
    pub(crate) fn parse(self, input: ParseStream) -> Result<(Requirements, TokenStream)> {
        let mut encoder = Encoder::new(self.receiver, self.span_start, self.span_end);
        self.parse_inner(&mut encoder, input, 0)?;
        encoder.into_output()
    }

    /// Parse `if <condition> { <quoted> } [else { <quoted> }]`.
    fn parse_condition(&self, input: ParseStream) -> Result<(Requirements, Ast)> {
        input.parse::<Token![if]>()?;
        let condition = syn::Expr::parse_without_eager_brace(input)?;

        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let (req, then_branch) = Quote::new(self.receiver).parse(input)?;

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

        let (r, then_branch) = Quote::new(self.receiver).parse(&content)?;
        req.merge_with(r);

        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;

            let content;
            syn::braced!(content in input);

            let (r, else_branch) = Quote::new(self.receiver).parse(&content)?;
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
        let pattern = input.parse::<syn::Pat>()?;
        input.parse::<Token![in]>()?;
        let expr = syn::Expr::parse_without_eager_brace(input)?;

        let join = if input.peek(join) {
            input.parse::<join>()?;

            let content;
            let paren = syn::parenthesized!(content in input);

            let (r, join) = Quote::new(self.receiver)
                .with_span(paren.span)
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

        let parser = Quote::new(self.receiver);
        let (r, stream) = parser.parse(&input)?;
        req.merge_with(r);

        let ast = Ast::Loop {
            pattern,
            join,
            expr,
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
            let pattern = multi_pat_with_leading_vert(&body)?;

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

                let parser = Quote::new(self.receiver);
                parser.parse(&block)?
            } else if body.peek(token::Paren) {
                let block;
                let paren = syn::parenthesized!(block in body);

                Quote::new(self.receiver)
                    .with_span(paren.span)
                    .parse(&block)?
            } else {
                let parser = Quote::new_until_comma(self.receiver);
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

    fn parse_expression(&self, encoder: &mut Encoder, input: ParseStream) -> Result<()> {
        let span = input.span();
        let hash = input.parse::<Token![#]>()?;
        let start = hash.span;

        // Single identifier without quoting.
        if !input.peek(token::Paren) {
            let ident = input.parse::<syn::Ident>()?;
            let cursor = Cursor::join(start, ident.span());

            encoder.encode(span, cursor, Ast::EvalIdent { ident })?;

            return Ok(());
        }

        let scope;
        let outer = syn::parenthesized!(scope in input);

        let cursor = Cursor::join(start, outer.span);

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
        } else if scope.peek(Token![ref]) {
            let ast = self.parse_scope(&scope)?;
            ast
        } else if scope.peek(syn::LitStr) && scope.peek2(crate::token::Eof) {
            let string = scope.parse::<syn::LitStr>()?.value();

            Ast::Literal { string }
        } else {
            Ast::Eval {
                expr: scope.parse()?,
            }
        };

        encoder.encode(span, cursor, ast)?;
        Ok(())
    }

    fn parse_inner(
        &self,
        encoder: &mut Encoder,
        input: ParseStream,
        group_depth: usize,
    ) -> Result<()> {
        syn::custom_punctuation!(Escape, ##);
        syn::custom_punctuation!(ControlStart, #<);

        while !input.is_empty() {
            if group_depth == 0 && self.until_comma && input.peek(Token![,]) {
                break;
            }

            // Escape sequence.
            if input.peek(Escape) {
                let escape = input.parse::<Escape>()?;
                let cursor = Cursor::join(escape.spans[0], escape.spans[1]);
                let mut punct = Punct::new('#', Spacing::Joint);
                punct.set_span(escape.spans[1]);
                encoder.encode(escape.span(), cursor, Ast::Tree { tt: punct.into() })?;
                continue;
            }

            if input.peek(syn::Token![#])
                && input.peek2(syn::Token![_])
                && input.peek3(token::Paren)
            {
                let start = input.parse::<syn::Token![#]>()?;
                input.parse::<syn::Token![_]>()?;

                let content;
                let paren = syn::parenthesized!(content in input);

                let parser = StringParser::new(self.receiver, paren.span);

                let (options, r, stream) = parser.parse(&content)?;
                encoder.requirements.merge_with(r);

                let cursor = Cursor::join(start.span(), paren.span);

                encoder.encode(
                    content.span(),
                    cursor,
                    Ast::String {
                        has_eval: options.has_eval,
                        stream,
                    },
                )?;
                continue;
            }

            // Control sequence.
            if input.peek(ControlStart) {
                let escape = input.parse::<ControlStart>()?;
                let control = input.parse::<Control>()?;
                let gt = input.parse::<token::Gt>()?;

                let cursor = Cursor::join(escape.span(), gt.span());
                encoder.encode(escape.span(), cursor, Ast::Control { control })?;
                continue;
            }

            let start_expression = input.peek2(token::Paren) || input.peek2(syn::Ident);

            if input.peek(Token![#]) && start_expression {
                self.parse_expression(encoder, input)?;
                continue;
            }

            if input.peek(syn::LitStr) {
                let s = input.parse::<syn::LitStr>()?;
                let cursor = Cursor::from(s.span());
                let span = s.span();
                encoder.encode(span, cursor, Ast::Quoted { s })?;
                continue;
            }

            // Test for different forms of groups and recurse if necessary.
            if input.peek(token::Brace) {
                let content;
                let braces = syn::braced!(content in input);
                self.parse_group(
                    encoder,
                    Delimiter::Brace,
                    braces.span,
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
                    braces.span,
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
                    braces.span,
                    &content,
                    group_depth,
                )?;
                continue;
            }

            let tt: TokenTree = input.parse()?;
            let cursor = Cursor::from(tt.span());
            let span = tt.span();

            encoder.encode(span, cursor, Ast::Tree { tt })?;
        }

        Ok(())
    }

    fn parse_group(
        &self,
        encoder: &mut Encoder,
        delimiter: Delimiter,
        span: Span,
        input: ParseStream,
        group_depth: usize,
    ) -> Result<()> {
        let cursor = Cursor::from(span);

        encoder.encode(
            span,
            cursor.first_character(),
            Ast::DelimiterOpen { delimiter },
        )?;

        self.parse_inner(encoder, input, group_depth + 1)?;

        encoder.encode(
            span,
            cursor.last_character(),
            Ast::DelimiterClose { delimiter },
        )?;

        Ok(())
    }
}

// NB: copied from: https://github.com/dtolnay/syn/blob/80c6889684da7e0a30ef5d0b03e9d73fa6160541/src/pat.rs#L792
//
// TODO: remove once made public in syn.
fn multi_pat_with_leading_vert(input: ParseStream) -> Result<syn::Pat> {
    let leading_vert: Option<Token![|]> = input.parse()?;
    multi_pat_impl(input, leading_vert)
}

fn multi_pat_impl(input: ParseStream, leading_vert: Option<Token![|]>) -> Result<syn::Pat> {
    let mut pat: syn::Pat = input.parse()?;
    if leading_vert.is_some()
        || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
    {
        let mut cases = syn::punctuated::Punctuated::new();
        cases.push_value(pat);
        while input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=]) {
            let punct = input.parse()?;
            cases.push_punct(punct);
            let pat: syn::Pat = input.parse()?;
            cases.push_value(pat);
        }
        pat = syn::Pat::Or(syn::PatOr {
            attrs: Vec::new(),
            leading_vert,
            cases,
        });
    }
    Ok(pat)
}
