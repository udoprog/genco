use proc_macro2::TokenStream;
use syn::parse::ParseStream;
use syn::{spanned::Spanned as _, Expr, Result, Token};

use crate::quote_parser;

pub(crate) struct QuoteInParser;

impl QuoteInParser {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        // Input expression, assign to a variable.
        let expr = input.parse::<Expr>()?;
        let span_start = expr.span();

        input.parse::<Token![=>]>()?;

        let parser = quote_parser::QuoteParser::new(&expr).with_span_start(span_start.start());

        parser.parse(input)
    }
}
