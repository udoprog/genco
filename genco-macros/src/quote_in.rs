use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Result, Token};

pub(crate) struct QuoteIn {
    pub(crate) stream: TokenStream,
}

impl Parse for QuoteIn {
    fn parse(input: ParseStream) -> Result<Self> {
        // Input expression, assign to a variable.
        let expr = input.parse::<syn::Expr>()?;
        input.parse::<Token![=>]>()?;

        let receiver = &syn::Ident::new("__genco_macros_toks", expr.span());
        let parser = crate::quote::Quote::new(receiver);
        let (req, output) = parser.parse(input)?;

        let check = req.into_check(&receiver);

        // Give the assignment its own span to improve diagnostics.
        let assign_mut = q::quote_spanned! { expr.span() =>
            let #receiver: &mut genco::tokens::Tokens<_> = &mut #expr
        };

        Ok(Self {
            stream: q::quote! {
                #assign_mut;
                #output
                #check
            },
        })
    }
}
