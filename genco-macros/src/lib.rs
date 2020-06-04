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
///   [FormatTokens].
/// * `#` can be escaped by repeating it twice in case it's needed in
///   the target language. So `##` would produce a single `#`.
/// * Inline statements can be evaluated using `#(<stmt>)`. They can also be
///   suffixed with `<stmt>,*` to treat the statement as an iterator, and add
///   the specified separator (`,` here) between each element.
///   Example: `#("test".quoted())` can be used to quote a string.
/// * The [register] functionality of [Tokens] is available by prefixing an
///   expression with `#@` as `#@<stmt>`.
///   Example: `#@only_imports` will [register] the variable `only_imports`.
/// * Expressions can be repeated. It is then expected that they evaluate to an
///   iterator. Expressions are repeated by adding the `<token>*` suffix. The
///   <token> will then be used as a separator between each element, and a
///   spacing will be added after it.
///   Example: `#(var),*` will treat `var` as an iterator and add `,` and a
///   spacing between each element.
/// * Scoped expressions using `#{<binding> => { <block> }}`, giving mutable
///   and scoped access to the token stream being built. This can be used with
///   the [quote_in!] macro for improved flow control.
///   Example: `quote!(#{tokens => { quote_in!(tokens => fn foo() {}) }})`.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// let tokens: Tokens<Rust> = quote!(#[test]);
/// assert_eq!("#[test]", tokens.to_string().unwrap());
///
/// let tokens: Tokens<Rust> = quote!(#{t => { quote_in!(t => #[test]) }});
/// assert_eq!("#[test]", tokens.to_string().unwrap());
/// ```
///
/// Bigger example:
///
/// ```rust
/// use genco::prelude::*;
///
/// // Import the LittleEndian item, without referencing it through the last
/// // module component it is part of.
/// let little_endian = rust::imported("byteorder", "LittleEndian").qualified();
/// let big_endian = rust::imported("byteorder", "BigEndian");
///
/// // This is a trait, so only import it into the scope (unless we intent to
/// // implement it).
/// let write_bytes_ext = rust::imported("byteorder", "WriteBytesExt").alias("_");
///
/// let tokens: Tokens<Rust> = quote! {
///     #@write_bytes_ext
///
///     let mut wtr = vec![];
///     wtr.write_u16::<#little_endian>(517).unwrap();
///     wtr.write_u16::<#big_endian>(768).unwrap();
///     assert_eq!(wtr, vec![#(0..10),*]);
/// };
///
/// println!("{}", tokens.to_file_string().unwrap());
/// ```
///
/// [FormatTokens]: https://docs.rs/genco/latest/genco/trait.FormatTokens.html
/// [register]: https://docs.rs/genco/latest/genco/struct.Tokens.html#method.register
/// [Tokens]: https://docs.rs/genco/latest/genco/struct.Tokens.html
#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let toks = Ident::new("__toks", Span::call_site());

    let parser = quote_parser::QuoteParser {
        receiver: &toks,
        borrowed: false,
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
/// and then the code to generate. To avoid taking ownership of the parameter
/// argument you can use the syntax `&mut *<ident>`. This can prevent borrowing
/// issues you encounter (see the `Borrowing` section below).
///
/// For example: `quote_in! { &mut *tokens => fn foo() {  } }`.
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
/// 14 | |             #{tokens => {
/// 15 | |                 tokens.append("42");
/// 16 | |             }}
/// 17 | |         }
/// 18 | |     }
///    | |_____^ cannot borrow as mutable
/// ```
///
/// This is because inner scoped like `#{tokens => { <block> }}` take ownership
/// of their variable by default. To have it perform a proper reborrow, you can
/// do the following instead:
///
/// ```rust
/// use genco::prelude::*;
///
/// let mut tokens = Tokens::<Rust>::new();
/// let tokens = &mut tokens;
///
/// for name in &["foo", "bar", "baz"] {
///     quote_in! { &mut *tokens =>
///         fn #(*name)() -> u32 {
///             #{tokens => {
///                 tokens.append("42");
///             }}
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
/// let mut tokens = Tokens::<Rust>::new();
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
/// use genco::{quote_in, Rust, Tokens};
///
/// let mut tokens = Tokens::<Rust>::new();
///
/// quote_in!(tokens => fn hello() -> u32 { 42 });
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
