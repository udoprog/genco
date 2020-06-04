use proc_macro2::TokenStream;
use syn::parse::ParseStream;
use syn::{Ident, Result, Token};

use crate::quote_parser;

pub(crate) struct QuoteInParser;

impl QuoteInParser {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        let borrowed = if input.peek(Token![&]) && input.peek2(Token![mut]) {
            input.parse::<Token![&]>()?;
            input.parse::<Token![mut]>()?;
            true
        } else {
            false
        };

        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        let parser = quote_parser::QuoteParser {
            receiver: &ident,
            borrowed,
        };
        parser.parse(input)
    }
}
