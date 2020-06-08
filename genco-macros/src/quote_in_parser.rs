use proc_macro2::TokenStream;
use syn::parse::ParseStream;
use syn::{Expr, Result, Token};

use crate::quote_parser;

pub(crate) struct QuoteInParser;

impl QuoteInParser {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        // Input expression, assign to a variable.
        let expr = input.parse::<Expr>()?;

        input.parse::<Token![=>]>()?;

        let parser = quote_parser::QuoteParser::new(&expr);

        parser.parse(input)
    }
}
