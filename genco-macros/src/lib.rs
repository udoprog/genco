#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::Span;
use syn::parse::{ParseStream, Parser as _};
use syn::{Expr, Ident};

mod cursor;
mod encoder;
mod item_buffer;
mod quote_in_parser;
mod quote_parser;

pub(crate) use self::cursor::Cursor;
pub(crate) use self::encoder::{Binding, Control, Delimiter, Encoder, MatchArm};
pub(crate) use self::item_buffer::ItemBuffer;

/// Language neutral, whitespace sensitive quasi-quoting for GenCo.
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
/// let field_ty = rust::imported("std::collections", "HashMap")
///     .with_arguments((rust::U32, rust::U32));
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
/// Inline code can be evaluated using `#(<stmt>)`.
///
/// Note that this is evaluated in the same scope as where the macro is invoked,
/// so you can make use of keywords like `?` (try) when appropriate.
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
/// [FormatTokens]: https://docs.rs/genco/latest/genco/trait.FormatTokens.html
///
/// # Escaping Whitespace
///
/// Because this macro is whitespace sensitive, it might sometimes be necessary
/// to provide hints of where they should be inserted.
///
/// The macro trims any trailing and leading whitespace that it sees. So
/// `quote!(Hello )` is the same as `quote!(Hello)`. To include a spacing at the
/// end, we can use the special `#<space>` escape sequence: `quote!(Hello#<space>)`.
///
/// The available escape sequences are:
///
/// * `#<space>` for inserting a spacing between tokens. This corresponds to the
///   [Tokens::spacing] function.
/// * `#<push>` for inserting a push operation. Push operations makes sure that
///   any following tokens are on their own dedicated line. This corresponds to
///   the [Tokens::push] function.
/// * `#<line>` for inserting a line operation. Line operations makes sure that
///   any following tokens have an empty line separating them. This corresponds
///   to the [Tokens::line] function.
///
/// ```rust
/// use genco::prelude::*;
///
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote!(foo#<push>bar#<line>baz#<space>biz);
///
/// assert_eq!("foo\nbar\n\nbaz biz", tokens.to_string().unwrap());
/// ```
///
/// [Tokens::spacing]: https://docs.rs/genco/latest/genco/struct.Tokens.html#method.spacing
/// [Tokens::push]: https://docs.rs/genco/latest/genco/struct.Tokens.html#method.push
/// [Tokens::line]: https://docs.rs/genco/latest/genco/struct.Tokens.html#method.line
///
/// # Loops
///
/// To repeat a pattern you can use `#(for <bindings> in <expr> { <quoted> })`,
/// where <expr> is an iterator.
///
/// It is also possible to use the more compact
/// `#(for <bindings> in <expr> => <quoted>)` (note the arrow).
///
/// `<quoted>` will be treated as a quoted expression, so anything which works
/// during regular quoting will work here as well, with the addition that
/// anything defined in `<bindings>` will be made available to the statement.
///
/// ```rust
/// use genco::prelude::*;
///
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote! {
///     Your numbers are: #(for n in numbers => #n#<space>)
/// };
///
/// assert_eq!("Your numbers are: 3 4 5", tokens.to_string().unwrap());
/// ```
///
/// Note how we had to escape the tail spacing (`#<space>`) to have it included, and
/// we also got a spacing at the end that we _probably_ don't want. To avoid
/// this we can instead to a joined repetition.
///
/// # Joining Loops
///
/// It's a common need to join loops. To do this, you can add `join (<quoted>)`
/// to the end of a repitition specification.
///
/// The expression specified in `join (<quoted>)` is added _between_ each
/// `<quoted>` produced by the loop.
///
/// One difference with the `<quoted>` section with the regular [quote!] macro
/// is that it is _whitespace sensitive_ at the tail of the expression.
///
/// So `(,)` would be different from `(, )`, which would have a spacing at the
/// end.
///
/// With that in mind, let's redo the numbers example above.
///
/// ```rust
/// use genco::prelude::*;
///
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote! {
///     Your numbers are: #(for n in numbers join (, ) => #n).
/// };
///
/// assert_eq!("Your numbers are: 3, 4, 5.", tokens.to_string().unwrap());
/// ```
///
/// [quote!]: macro.quote.html
///
/// # Conditionals
///
/// You can specify a conditional with `#(if <condition> => <then>)` where
/// <condition> is an expression evaluating to a `bool`, and `<then>` and
/// `<else>` are quoted expressions.
///
/// It's also possible to specify a condition with an else branch, by using
/// `#(if <condition> { <then> } else { <else> })`. In this instance, `<else>`
/// is also a quoted expression.
///
/// ```rust
/// use genco::prelude::*;
///
/// fn greeting(hello: bool, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting: #(if hello {
///         Hello #name
///     } else {
///         Goodbye #name
///     }))
/// }
///
/// let tokens = greeting(true, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string().unwrap());
///
/// let tokens = greeting(false, "John");
/// assert_eq!("Custom Greeting: Goodbye John", tokens.to_string().unwrap());
/// ```
///
/// The `<else>` branch is optional, so the following is a valid expression that
/// if `false`, won't result in any tokens:
///
/// ```rust
/// use genco::prelude::*;
///
/// fn greeting(hello: bool, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting:#(if hello {
///         #<space>Hello #name
///     }))
/// }
///
/// let tokens = greeting(true, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string().unwrap());
///
/// let tokens = greeting(false, "John");
/// assert_eq!("Custom Greeting:", tokens.to_string().unwrap());
/// ```
///
/// # Match Statements
///
/// You can specify a match statement with
/// `#(match <condition> { [<pattern> => <quoted>,]* }`, where <condition> is an
/// evaluated expression that is match against each subsequent <pattern>. If a
/// pattern matches, the arm with the matching `<quoted>` block is evaluated.
///
/// ```rust
/// use genco::prelude::*;
///
/// enum Greeting {
///     Hello,
///     Goodbye,
/// }
///
/// fn greeting(greeting: Greeting, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting: #(match greeting {
///         Greeting::Hello => Hello #name,
///         Greeting::Goodbye => Goodbye #name,
///     }))
/// }
///
/// let tokens = greeting(Greeting::Hello, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string().unwrap());
///
/// let tokens = greeting(Greeting::Goodbye, "John");
/// assert_eq!("Custom Greeting: Goodbye John", tokens.to_string().unwrap());
/// ```
///
/// # Scopes
///
/// You can use `#(<binding> { <expr> })` to gain mutable access to the current
/// token stream. This is a great alternative if you want to do more complex
/// logic during evaluation.
///
/// For a more compact version, you can also use `#(<binding> => <expr>)`
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
///                 toks.space();
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

    let parser = quote_parser::QuoteParser::new(&toks);

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
/// [quote!]: macro.quote.html
///
/// # Example
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
/// # Use inside of [quote!]
///
/// [quote_in!] can be used inside of a [quote!] macro by using a scope.
///
/// ```rust
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn foo(v: bool) -> u32 {
///         #(out {
///             quote_in! { out =>
///                 if v {
///                     1
///                 } else {
///                     0
///                 }
///             }
///         })
///     }
/// };
/// ```
///
/// [quote!]: macro.quote.html
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
