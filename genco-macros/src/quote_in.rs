use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Result, Token};

use crate::Ctxt;

pub(crate) struct QuoteIn {
    pub(crate) stream: TokenStream,
}

impl Parse for QuoteIn {
    fn parse(input: ParseStream) -> Result<Self> {
        // Input expression, assign to a variable.
        let expr = input.parse::<syn::Expr>()?;
        input.parse::<Token![=>]>()?;

        let cx = Ctxt::default();

        let parser = crate::quote::Quote::new(&cx);
        let (req, output) = parser.parse(input)?;

        let check = req.into_check(&cx.receiver);

        let Ctxt { receiver, module } = &cx;

        // Give the assignment its own span to improve diagnostics.
        let assign_mut = q::quote_spanned! { expr.span() =>
            let #receiver: &mut #module::tokens::Tokens<_> = &mut #expr;
        };

        let stream = q::quote! {{
            #assign_mut
            #output
            #check
        }};

        Ok(Self { stream })
    }
}
