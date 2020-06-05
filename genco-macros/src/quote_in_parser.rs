use proc_macro2::TokenStream;
use syn::parse::ParseStream;
use syn::{Ident, Result, Token};

use crate::quote_parser;

pub(crate) struct QuoteInParser;

impl QuoteInParser {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        let (span_start, receiver_borrowed) =
            if input.peek(Token![&]) && input.peek2(Token![mut]) && input.peek3(Token![*]) {
                let first = input.parse::<Token![&]>()?;
                input.parse::<Token![mut]>()?;
                input.parse::<Token![*]>()?;
                (Some(first.span.start()), true)
            } else {
                (None, false)
            };

        let ident = input.parse::<Ident>()?;

        let span_start = span_start.unwrap_or_else(|| ident.span().start());

        input.parse::<Token![=>]>()?;
        let parser = quote_parser::QuoteParser {
            receiver: &ident,
            span_start: Some(span_start),
            receiver_borrowed,
        };
        parser.parse(input)
    }
}
