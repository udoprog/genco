use proc_macro2::TokenStream;
use syn::parse::ParseStream;
use syn::{Ident, Result, Token};

use crate::quote_parser;

pub(crate) struct QuoteInParser;

impl QuoteInParser {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        let content;
        let _ = syn::braced!(content in input);

        let parser = quote_parser::QuoteParser { receiver: &ident };

        parser.parse(&content)
    }
}
