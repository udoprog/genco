use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::Result;

use crate::Ctxt;

pub(crate) struct QuoteFn {
    pub(crate) stream: TokenStream,
}

impl Parse for QuoteFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let cx = Ctxt::default();

        let parser = crate::quote::Quote::new(&cx);
        let (req, output) = parser.parse(input)?;

        let check = req.into_check(&cx.receiver);

        let Ctxt { receiver, module } = &cx;

        let stream = q::quote! {
            #module::tokens::from_fn(move |#receiver| {
                #output
                #check
            })
        };

        Ok(Self { stream })
    }
}
