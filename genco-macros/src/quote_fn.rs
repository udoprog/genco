use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::Result;

pub(crate) struct QuoteFn {
    pub(crate) stream: TokenStream,
}

impl Parse for QuoteFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let receiver = &syn::Ident::new("__genco_macros_toks", input.span());
        let parser = crate::quote::Quote::new(receiver);
        let (req, output) = parser.parse(input)?;

        let check = req.into_check(&receiver);

        let stream = q::quote! {
            genco::tokens::from_fn(move |#receiver| {
                #output
                #check
            })
        };

        Ok(Self { stream })
    }
}
