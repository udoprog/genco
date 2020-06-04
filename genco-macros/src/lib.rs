#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::Span;
use syn::parse::{ParseStream, Parser as _};
use syn::Ident;

mod cursor;
mod item_buffer;
mod quote_in_parser;
mod quote_parser;

pub(crate) use self::cursor::Cursor;
pub(crate) use self::item_buffer::ItemBuffer;

/// Quotes the specified expression as a stream of tokens for use with genco.
///
/// # Mechanisms
///
/// * Elements are interpolated using `#`, so to include the variable `test`,
///   you could write `#test`. Returned elements must implement
///   [`FormatTokens`].
/// * Inline statements can be evaluated using `#(<stmt>)`, or `#{<stmt>}`,
///   or `#[<stmt>]`. In effect, anything that counts as a _group_ in Rust.
///   For example: `#("test".quoted())` can be used to quote a string.
/// * The [`register`] functionality of [`Tokens`] is available by prefixing an
///   expression with `@` as `@<stmt>`.
///   For example: `@only_imports`.
/// * `#` can be escaped by repeating it twice in case it's needed in
///   the target language. So `##` would produce a single `#`.
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
///
/// use genco::rust::imported;
/// use genco::{quote, Rust, Tokens};
///
/// // Import the LittleEndian item, without referencing it through the last
/// // module component it is part of.
/// let little_endian = imported("byteorder", "LittleEndian").qualified();
/// let big_endian = imported("byteorder", "BigEndian");
///
/// // This is a trait, so only import it into the scope (unless we intent to
/// // implement it).
/// let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
///
/// let tokens: Tokens<Rust> = quote! {
///     @write_bytes_ext
///
///     let mut wtr = vec![];
///     wtr.write_u16::<#little_endian>(517).unwrap();
///     wtr.write_u16::<#big_endian>(768).unwrap();
///     assert_eq!(wtr, vec![5, 2, 3, 0]);
/// };
///
/// println!("{}", tokens.to_file_string().unwrap());
/// ```
///
/// [`FormatTokens`]: https://docs.rs/genco/latest/genco/trait.FormatTokens.html
/// [`register`]: https://docs.rs/genco/latest/genco/struct.Tokens.html#method.register
#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let toks = Ident::new("__toks", Span::call_site());

    let parser = quote_parser::QuoteParser { receiver: &toks };

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

/// Same as [quote], except that it allows for quoting directly to a token
/// stream.
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
///
/// use genco::{quote_in, Rust, Tokens};
///
/// let mut tokens = Tokens::<Rust>::new();
///
/// quote_in! {
///     tokens => {
///         fn hello() -> u32 { 42 }
///     }
/// }
///
/// assert_eq!(vec!["fn hello() -> u32 { 42 }", ""], tokens.to_file_vec().unwrap());

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
