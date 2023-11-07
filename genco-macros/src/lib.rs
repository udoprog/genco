//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/genco-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/genco)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/genco-macros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/genco-macros)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-genco--macros-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/genco-macros)

#![recursion_limit = "256"]
#![allow(clippy::type_complexity)]
#![cfg_attr(proc_macro_span, feature(proc_macro_span))]

extern crate proc_macro;

use proc_macro2::Span;
use syn::parse::{ParseStream, Parser as _};

struct Ctxt {
    receiver: syn::Ident,
    module: syn::Path,
}

impl Default for Ctxt {
    fn default() -> Self {
        let mut module = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::default(),
        };

        module
            .segments
            .push(syn::Ident::new("genco", Span::call_site()).into());

        Self {
            receiver: syn::Ident::new("__genco_macros_toks", Span::call_site()),
            module,
        }
    }
}

mod ast;
mod cursor;
mod encoder;
mod fake;
mod quote;
mod quote_fn;
mod quote_in;
mod requirements;
mod static_buffer;
mod string_parser;

#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cx = Ctxt::default();
    let parser = crate::quote::Quote::new(&cx);

    let parser = move |stream: ParseStream| parser.parse(stream);

    let (req, output) = match parser.parse(input) {
        Ok(data) => data,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let check = req.into_check(&cx.receiver);

    let Ctxt { receiver, module } = &cx;

    let gen = q::quote! {{
        let mut #receiver = #module::tokens::Tokens::new();

        {
            let mut #receiver = &mut #receiver;
            #output
        }

        #check
        #receiver
    }};

    gen.into()
}

#[proc_macro]
pub fn quote_in(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let quote_in = syn::parse_macro_input!(input as quote_in::QuoteIn);
    quote_in.stream.into()
}

#[proc_macro]
pub fn quote_fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let quote_fn = syn::parse_macro_input!(input as quote_fn::QuoteFn);
    quote_fn.stream.into()
}
