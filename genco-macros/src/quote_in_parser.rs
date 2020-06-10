use proc_macro2::TokenStream;
use syn::parse::ParseStream;
use syn::spanned::Spanned as _;
use syn::{Result, Token};

use crate::quote_parser;

pub(crate) struct QuoteInParser;

impl QuoteInParser {
    pub(crate) fn parse(self, input: ParseStream) -> Result<TokenStream> {
        // Input expression, assign to a variable.
        let expr = input.parse::<syn::Expr>()?;
        input.parse::<Token![=>]>()?;

        let receiver = &syn::Ident::new("__genco_macros_toks", expr.span());
        let parser = quote_parser::QuoteParser::new(receiver);
        let output = parser.parse(input)?;

        // Give the assignment its own span to improve diagnostics.
        let assign_mut = quote::quote_spanned! { expr.span() => &mut #expr };

        Ok(quote::quote! {
            let #receiver = #assign_mut;
            #output
        })
    }
}
