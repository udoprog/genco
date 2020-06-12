#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::Span;
use syn::parse::{ParseStream, Parser as _};

mod cursor;
mod encoder;
mod item_buffer;
mod quote_in_parser;
mod quote_parser;
mod token;

pub(crate) use self::cursor::Cursor;
pub(crate) use self::encoder::{Control, Delimiter, Encoder, MatchArm};
pub(crate) use self::item_buffer::ItemBuffer;

/// Language neutral whitespace sensitive quasi-quoting.
///
/// ```rust
/// # use genco::prelude::*;
///
/// # fn generate() -> Tokens<()> {
/// quote!(hello world)
/// # }
///
/// # fn generate_multiline() -> Tokens<()> {
/// quote! {
///     hello...
///     world!
/// }
/// # }
/// ```
///
/// # String Quoting
///
/// Literal strings like `quote!("hello")` are automatically quoted for the
/// target language according to its [Lang::quote_string] implementation.
///
/// To avoid this, you would have to produce a string literal. This can be done
/// through interpolation as shown below in the third example.
///
/// [Lang::quote_string]: https://docs.rs/genco/0/genco/trait.Lang.html#method.quote_string
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let tokens: rust::Tokens = quote! {
///     "hello world"
///     #("hello world".quoted())
///     #("\"hello world\"")
/// };
///
/// assert_eq!(
///     vec![
///         "\"hello world\"",
///         "\"hello world\"",
///         "\"hello world\"",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
///
/// The items produced in the token stream above are however subtly different.
///
/// The first one is a static *quoted string*. The second one is a boxed *quoted
/// string*, who's content is stored on the heap. And the third one is a static
/// *literal* which bypasses language quoting entirely.
///
/// ```rust
/// # use genco::prelude::*;
/// # fn main() -> genco::fmt::Result {
/// # let tokens: rust::Tokens = quote! {
/// #     "hello world"
/// #     #("hello world".quoted())
/// #     #("\"hello world\"")
/// # };
/// #
/// use genco::tokens::{Item, ItemStr};
///
/// assert_eq!(
///     vec![
///         Item::Quoted(ItemStr::Static("hello world")),
///         Item::Push,
///         Item::Quoted(ItemStr::Box("hello world".into())),
///         Item::Push,
///         Item::Literal(ItemStr::Static("\"hello world\"")),
///     ],
///     tokens,
/// );
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// # Interpolation
///
/// Elements are interpolated using `#`, so to include the variable `test`,
/// you could write `#test`. Returned elements must implement [FormatInto].
///
/// **Note:** `#` can be escaped by repeating it twice. So `##` would produce a
/// single `#` token.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
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
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// Inline code can be evaluated through `#(<expr>)`.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let world = "world";
///
/// let tokens: genco::Tokens = quote!(hello #(world.to_uppercase()));
///
/// assert_eq!("hello WORLD", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// Interpolations are evaluated in the same scope as where the macro is
/// invoked, so you can make use of keywords like `?` (try) when appropriate.
///
/// ```rust
/// use std::error::Error;
///
/// use genco::prelude::*;
///
/// fn age_fn(age: &str) -> Result<rust::Tokens, Box<dyn Error>> {
///     Ok(quote! {
///         fn age() {
///             println!("You are {} years old!", #(str::parse::<u32>(age)?));
///         }
///     })
/// }
/// ```
///
/// [FormatInto]: https://docs.rs/genco/0/genco/tokens/trait.FormatInto.html
///
/// # Escape Sequences
///
/// Because this macro is _whitespace sensitive_, it might sometimes be
/// necessary to provide hints of where they should be inserted.
///
/// `quote!` trims any trailing and leading whitespace that it sees. So
/// `quote!(Hello )` is the same as `quote!(Hello)`. To include a space at the
/// end, we can use the special `#<space>` escape sequence: `quote!(Hello#<space>)`.
///
/// The available escape sequences are:
///
/// * `#<space>` — Inserts a space between tokens. This corresponds to the
///   [Tokens::space] function.
///
/// * `#<push>` — Inserts a push operation. Push operations makes sure that
///   any following tokens are on their own dedicated line. This corresponds to
///   the [Tokens::push] function.
///
/// * `#<line>` — Inserts a forced line. Line operations makes sure that
///   any following tokens have an empty line separating them. This corresponds
///   to the [Tokens::line] function.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote!(foo#<push>bar#<line>baz#<space>biz);
///
/// assert_eq!("foo\nbar\n\nbaz biz", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// [Tokens::space]: https://docs.rs/genco/0/genco/struct.Tokens.html#method.space
/// [Tokens::push]: https://docs.rs/genco/0/genco/struct.Tokens.html#method.push
/// [Tokens::line]: https://docs.rs/genco/0/genco/struct.Tokens.html#method.line
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
/// # fn main() -> genco::fmt::Result {
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote! {
///     Your numbers are: #(for n in numbers => #n#<space>)
/// };
///
/// assert_eq!("Your numbers are: 3 4 5", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// # Joining Loops
///
/// You can add `join (<quoted>)` to the end of a repitition specification.
///
/// The expression specified in `join (<quoted>)` is added _between_ each
/// element produced by the loop.
///
/// **Note:** The argument to `join` us *whitespace sensitive*, so leading and
/// trailing is preserved. `join (,)` and `join (, )` would therefore produce
/// different results.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let numbers = 3..=5;
///
/// let tokens: Tokens<()> = quote! {
///     Your numbers are: #(for n in numbers join (, ) => #n).
/// };
///
/// assert_eq!("Your numbers are: 3, 4, 5.", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
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
/// # fn main() -> genco::fmt::Result {
/// fn greeting(hello: bool, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting: #(if hello {
///         Hello #name
///     } else {
///         Goodbye #name
///     }))
/// }
///
/// let tokens = greeting(true, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string()?);
///
/// let tokens = greeting(false, "John");
/// assert_eq!("Custom Greeting: Goodbye John", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// The `<else>` branch is optional, so the following is a valid expression that
/// if `false`, won't result in any tokens:
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// fn greeting(hello: bool, name: &str) -> Tokens<()> {
///     quote!(Custom Greeting:#(if hello {
///         #<space>Hello #name
///     }))
/// }
///
/// let tokens = greeting(true, "John");
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string()?);
///
/// let tokens = greeting(false, "John");
/// assert_eq!("Custom Greeting:", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
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
/// # fn main() -> genco::fmt::Result {
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
/// assert_eq!("Custom Greeting: Hello John", tokens.to_string()?);
///
/// let tokens = greeting(Greeting::Goodbye, "John");
/// assert_eq!("Custom Greeting: Goodbye John", tokens.to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// # Scopes
///
/// You can use `#(ref <binding> { <quoted> })` to gain mutable access to the current
/// token stream. This is an alternative to existing control flow operators if
/// you want to execute more complex logic during evaluation.
///
/// For a more compact version, you can also omit the braces by doing
/// `#(ref <binding> => <quoted>)`.
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
/// # fn main() -> genco::fmt::Result {
/// fn quote_greeting(surname: &str, lastname: Option<&str>) -> rust::Tokens {
///     quote! {
///         Hello #surname#(ref toks {
///             if let Some(lastname) = lastname {
///                 toks.space();
///                 toks.append(lastname);
///             }
///         })
///     }
/// }
///
/// assert_eq!("Hello John", quote_greeting("John", None).to_string()?);
/// assert_eq!("Hello John Doe", quote_greeting("John", Some("Doe")).to_string()?);
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// ## Whitespace Detection
///
/// The [quote!] macro has the following rules for dealing with indentation and
/// spacing.
///
/// **Spaces** — Two tokens that are separated, are spaced. Regardless of how
/// many spaces there are between them. This can also be controlled manually by
/// inserting the [`#<space>`] escape in the token stream.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let tokens: rust::Tokens = quote! {
///     fn     test()     {
///         println!("Hello... ");
///
///         println!("World!");
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "fn test() {",
///         "    println!(\"Hello... \");",
///         "",
///         "    println!(\"World!\");",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// **Line breaking** — Line breaks are detected by leaving two empty lines
/// between two tokens. This can also be controlled manually by inserting the
/// [`#<line>`] escape in the token stream.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let tokens: rust::Tokens = quote! {
///     fn test() {
///         println!("Hello... ");
///
///
///
///         println!("World!");
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "fn test() {",
///         "    println!(\"Hello... \");",
///         "",
///         "    println!(\"World!\");",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
///
/// <br>
///
/// **Indentation** — Indentation is determined on a row-by-row basis. If a
/// column is further in than the one on the preceeding row, it is indented
/// *one level* deeper.
///
/// If a column starts shallower than a previous row, it will be matched against
/// previously known indentation levels.
///
/// All indentations inserted during the macro will be unrolled at the end of
/// it. So any trailing indentations will be matched by unindentations.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let tokens: rust::Tokens = quote! {
///     fn test() {
///             println!("Hello... ");
///
///             println!("World!");
///     }
/// };
///
/// assert_eq!(
///     vec![
///         "fn test() {",
///         "    println!(\"Hello... \");",
///         "",
///         "    println!(\"World!\");",
///         "}",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
///
/// A mismatched indentation would result in an error:
///
/// ```rust,compile_fail
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn test() {
///             println!("Hello... ");
///
///         println!("World!");
///     }
/// };
/// ```
///
/// ```text
/// ---- src\lib.rs -  (line 150) stdout ----
/// error: expected 4 less spaces of indentation
/// --> src\lib.rs:157:9
///    |
/// 10 |         println!("World!");
///    |         ^^^^^^^
/// ```
///
/// [`#<space>`]: #escape-sequences
/// [`#<line>`]: #escape-sequences
#[proc_macro]
pub fn quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let receiver = &syn::Ident::new("__genco_macros_toks", Span::call_site());

    let parser = quote_parser::QuoteParser::new(receiver);

    let parser = move |stream: ParseStream| parser.parse(stream);

    let output = match parser.parse(input) {
        Ok(data) => data,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let gen = quote::quote! {{
        let mut #receiver = genco::Tokens::new();

        {
            let mut #receiver = &mut #receiver;
            #output
        }

        #receiver
    }};

    gen.into()
}

/// Behaves the same as [quote!] while quoting into an existing token stream
/// with `<target> => <quoted>`.
///
/// This macro takes a destination stream followed by an `=>` and the tokens to
/// extend that stream with.
///
/// ```rust
/// # use genco::prelude::*;
///
/// # fn generate() -> Tokens<()> {
/// let mut tokens = Tokens::new();
/// quote_in!(tokens => hello world);
/// # tokens
/// # }
///
/// # fn generate_into(tokens: &mut Tokens<()>) {
/// quote_in! { *tokens =>
///     hello...
///     world!
/// }
/// # }
/// ```
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
/// # Example use inside of [quote!]
///
/// [quote_in!] can be used inside of a [quote!] macro by using a scope.
///
/// ```rust
/// use genco::prelude::*;
///
/// let tokens: rust::Tokens = quote! {
///     fn foo(v: bool) -> u32 {
///         #(ref out {
///             quote_in! { *out =>
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
/// [quote_in!] macro.quote_in.html
/// [quote!]: macro.quote.html
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
