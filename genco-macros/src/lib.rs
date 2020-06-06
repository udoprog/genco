#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::Span;
use syn::parse::{ParseStream, Parser as _};
use syn::{Expr, Ident};

mod cursor;
mod item_buffer;
mod quote_in_parser;
mod quote_parser;

pub(crate) use self::cursor::Cursor;
pub(crate) use self::item_buffer::ItemBuffer;

/// Quotes the specified expression as a stream of tokens for use with genco.
///
/// # Simple Interpolation
///
/// Elements are interpolated using `#`, so to include the variable `test`,
/// you could write `#test`. Returned elements must implement [FormatTokens].
///
/// `#` can be escaped by repeating it twice in case it's needed in the target
/// language. So `##` would produce a single `#`.
///
/// ```rust
/// use genco::prelude::*;
///
/// let field_ty = rust::imported("std::collections", "HashMap").with_arguments((rust::U32, rust::U32));
///
/// let tokens: rust::Tokens = quote! {
///     struct Quoted {
///         field: #field_ty,
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "use std::collections::HashMap;",
///         "",
///         "struct Quoted {",
///         "    field: HashMap<u32, u32>,",
///         "}",
///     ],
///     tokens.to_file_vec().unwrap(),
/// );
/// ```
///
/// Inline code can be evaluated using `#(<stmt>)`. Note that this is evaluated
/// in the same scope as where the macro is invoked, so you can make use of
/// keywords like `?` (try) when appropriate.
///
/// ```rust
/// use genco::prelude::*;
///
/// let world = "world";
///
/// let tokens: genco::Tokens = quote!(hello #(world.to_uppercase()));
///
/// assert_eq!("hello WORLD", tokens.to_string().unwrap());
/// ```
///
/// # Scopes
///
/// You can use `#(<binding> => <stmt>)` to gain mutable access to the current
/// token stream. This is a great alternative if you want to do more complex
/// logic during evaluation.
///
/// Note that this can cause borrowing issues if the underlying stream is
/// already a mutable reference. To work around this you can specify
/// `*<binding>` to cause it to reborrow.
///
/// For more information, see [quote_in!].
///
/// ```rust
/// use genco::prelude::*;
///
/// fn quote_greeting(surname: &str, lastname: Option<&str>) -> rust::Tokens {
///     quote! {
///         Hello #surname#(toks => {
///             if let Some(lastname) = lastname {
///                 toks.spacing();
///                 toks.append(lastname);
///             }
///         })
///     }
/// }
///
/// assert_eq!("Hello John", quote_greeting("John", None).to_string().unwrap());
/// assert_eq!("Hello John Doe", quote_greeting("John", Some("Doe")).to_string().unwrap());
/// ```
#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let toks = Ident::new("__toks", Span::call_site());
    let toks = Expr::Verbatim(quote::quote!(#toks));

    let parser = quote_parser::QuoteParser {
        receiver: &toks,
        span_start: None,
    };

    let parser = move |stream: ParseStream| parser.parse(stream);

    let output = match parser.parse(input) {
        Ok(data) => data,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let gen = quote::quote! {{
        let mut #toks = genco::Tokens::new();
        #output
        #toks
    }};

    gen.into()
}

/// Same as [quote!], except that it allows for quoting directly to a token
/// stream.
///
/// You specify the destination stream as the first argument, followed by a `=>`
/// and then the code to generate.
///
/// Note that there is a potential issue with reborrowing
///
/// # Reborrowing
///
/// In case you get a borrow issue like the following:
///
/// ```text
/// 9  |   let tokens = &mut tokens;
///    |       ------ help: consider changing this to be mutable: `mut tokens`
/// ...
/// 12 | /     quote_in! { tokens =>
/// 13 | |         fn #name() -> u32 {
/// 14 | |             #(tokens => tokens.append("42");)
/// 15 | |         }
/// 16 | |     }
///    | |_____^ cannot borrow as mutable
/// ```
///
/// This is because inner scoped like `#(tokens => <code>)` take ownership
/// of their variable by default. To have it perform a proper reborrow, you can
/// do the following instead:
///
/// ```rust
/// use genco::prelude::*;
///
/// let mut tokens = Tokens::<Rust>::new();
/// let tokens = &mut tokens;
///
/// for name in vec!["foo", "bar", "baz"] {
///     quote_in! { tokens =>
///         fn #name() -> u32 {
///             #(*tokens => tokens.append("42");)
///         }
///     }
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// let mut tokens = rust::Tokens::new();
///
/// quote_in! { tokens =>
///     fn foo() -> u32 {
///         42
///     }
/// }
/// ```
///
/// # Examples
///
/// ```rust
///
/// use genco::prelude::*;
///
/// let mut tokens = rust::Tokens::new();
///
/// quote_in!(tokens => fn hello() -> u32 { 42 });
///
/// assert_eq!(vec!["fn hello() -> u32 { 42 }"], tokens.to_file_vec().unwrap());
#[proc_macro]
pub fn quote_in(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let quote_in_parser::QuoteInParser;

    let parser = quote_in_parser::QuoteInParser;

    let parser = move |stream: ParseStream| parser.parse(stream);

    let output = match parser.parse(input) {
        Ok(data) => data,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let gen = quote::quote! {{
        #output
    }};

    gen.into()
}
