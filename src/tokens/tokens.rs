//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use crate::fmt;
use crate::tokens::{FormatInto, Item, RegisterTokens};
use crate::{Lang, LangItem};
use std::cmp;
use std::iter::FromIterator;
use std::num::NonZeroI16;
use std::slice;
use std::vec;

/// A stream of tokens.
///
/// # Structural Guarantees
///
/// This stream of tokens provides the following structural guarantees.
///
/// * Only one [space] may occur in sequence.
/// * Only one [push] may occur in sequence.
/// * A [push] may never be preceeded by a [line], since it would have no
///   effect.
/// * Every [line] must be preceeded by a [push].
///
/// ```rust
/// use genco::Tokens;
/// use genco::tokens::Item;
///
/// let mut tokens = Tokens::<()>::new();
///
/// tokens.push();
/// tokens.push();
///
/// assert_eq!(vec![Item::Push::<()>], tokens);
/// ```
///
/// [space]: Self::space()
/// [push]: Self::push()
/// [line]: Self::line()
#[derive(Default)]
pub struct Tokens<L = ()>
where
    L: Lang,
{
    items: Vec<Item<L>>,
}

impl<L> std::fmt::Debug for Tokens<L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_list().entries(self.items.iter()).finish()
    }
}

impl<L> Tokens<L>
where
    L: Lang,
{
    /// Create a new empty stream of tokens.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let tokens = Tokens::<()>::new();
    ///
    /// assert!(tokens.is_empty());
    /// ```
    pub fn new() -> Self {
        Tokens { items: Vec::new() }
    }

    /// Construct an iterator over the token stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::tokens::{ItemStr, Item};
    ///
    /// let tokens: Tokens<()> = quote!(foo bar baz);
    /// let mut it = tokens.iter();
    ///
    /// assert_eq!(Some(&Item::Literal(ItemStr::Static("foo"))), it.next());
    /// assert_eq!(Some(&Item::Space), it.next());
    /// assert_eq!(Some(&Item::Literal(ItemStr::Static("bar"))), it.next());
    /// assert_eq!(Some(&Item::Space), it.next());
    /// assert_eq!(Some(&Item::Literal(ItemStr::Static("baz"))), it.next());
    /// assert_eq!(None, it.next());
    /// ```
    pub fn iter(&self) -> Iter<'_, L> {
        Iter {
            iter: self.items.iter(),
        }
    }

    /// Construct an owned iterator over the token stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::tokens::{ItemStr, Item};
    ///
    /// let tokens: Tokens<()> = quote!(foo bar baz);
    /// let mut it = tokens.into_iter();
    ///
    /// assert_eq!(Some(Item::Literal(ItemStr::Static("foo"))), it.next());
    /// assert_eq!(Some(Item::Space), it.next());
    /// assert_eq!(Some(Item::Literal(ItemStr::Static("bar"))), it.next());
    /// assert_eq!(Some(Item::Space), it.next());
    /// assert_eq!(Some(Item::Literal(ItemStr::Static("baz"))), it.next());
    /// assert_eq!(None, it.next());
    /// ```
    pub fn into_iter(self) -> IntoIter<L> {
        IntoIter {
            iter: self.items.into_iter(),
        }
    }

    /// Append the given tokens.
    ///
    /// This append function takes anything implementing [FormatInto] making
    /// the argument's behavior customizable. Most primitive types have built-in
    /// implementations of [FormatInto] treating them as raw tokens.
    ///
    /// Most notabley, things implementing [FormatInto] can be used as
    /// arguments for [interpolation] in the [quote!] macro.
    ///
    /// [quote!]: macro.quote.html
    /// [interpolation]: macro.quote.html#interpolation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let mut tokens = Tokens::<()>::new();
    /// tokens.append(4u32);
    ///
    /// assert_eq!(quote!(#(4u32)), tokens);
    /// ```
    pub fn append<T>(&mut self, tokens: T)
    where
        T: FormatInto<L>,
    {
        tokens.format_into(self)
    }

    /// Push a single item to the stream while checking for structural
    /// guarantees.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::tokens::{Item, ItemStr};
    ///
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.item(Item::Literal(ItemStr::Static("foo")));
    /// tokens.item(Item::Space);
    /// tokens.item(Item::Space); // Note: second space ignored
    /// tokens.item(Item::Literal(ItemStr::Static("bar")));
    ///
    /// assert_eq!(tokens, quote!(foo bar));
    /// ```
    pub fn item(&mut self, item: Item<L>) {
        match item {
            Item::Push => self.push(),
            Item::Line => self.line(),
            Item::Space => self.space(),
            other => self.items.push(other),
        }
    }

    /// Extend with another stream of tokens.
    ///
    /// This respects the structural requirements of adding one element at a
    /// time, like you would get by calling [space], [push], or [line].
    ///
    /// [space]: Self::space()
    /// [push]: Self::push()
    /// [line]: Self::line()
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::tokens::{Item, ItemStr};
    ///
    /// let mut tokens: Tokens<()> = quote!(foo bar);
    /// tokens.extend(quote!(#<space>baz));
    ///
    /// assert_eq!(tokens, quote!(foo bar baz));
    /// ```
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Item<L>>,
    {
        for item in it {
            self.item(item);
        }
    }

    /// Walk over all imports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let debug = rust::imported("std::fmt", "Debug");
    ///
    /// let ty = rust::imported("std::collections", "HashMap")
    ///     .with_arguments((rust::U32, debug.into_dyn()));
    ///
    /// let tokens = quote!(foo #ty baz);
    ///
    /// for import in tokens.walk_imports() {
    ///     println!("{:?}", import);
    /// }
    /// ```
    pub fn walk_imports(&self) -> WalkImports<'_, L> {
        WalkImports {
            queue: self.items.iter(),
        }
    }

    /// Add an registered custom element that is _not_ rendered.
    ///
    /// Registration can be used to generate imports that do not render a
    /// visible result.
    ///
    /// # Examples
    ///
    /// The `register` functionality is available through the [quote!] macro
    /// by using the [register] function.
    ///
    /// [register]: Tokens::register
    ///
    /// ```rust
    ///
    /// use genco::rust::{imported, Config};
    /// use genco::quote;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
    ///
    /// let tokens = quote!(#@(write_bytes_ext));
    ///
    /// assert_eq!("use byteorder::WriteBytesExt as _;", tokens.to_file_string()?);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [quote!]: macro.quote.html
    pub fn register<T>(&mut self, tokens: T)
    where
        T: RegisterTokens<L>,
    {
        tokens.register_tokens(self);
    }

    /// Check if tokens contain no items.
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let tokens: Tokens<()> = quote!();
    ///
    /// assert!(tokens.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Add a single spacing to the token stream.
    ///
    /// Note that due to structural guarantees two consequent spaces may not
    /// follow each other in the same token stream.
    ///
    /// A space operation has no effect unless it's followed by a non-whitespace
    /// token.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.space();
    /// tokens.append("hello");
    /// tokens.space();
    /// tokens.space(); // Note: ignored
    /// tokens.append("world");
    /// tokens.space();
    ///
    /// assert_eq!(
    ///     vec![
    ///         " hello world",
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn space(&mut self) {
        if let Some(Item::Space) = self.items.last() {
            return;
        }

        self.items.push(Item::Space);
    }

    /// Add a single push operation.
    ///
    /// Push operations ensure that any following tokens are added to their own
    /// line.
    ///
    /// A push has no effect unless it's *preceeded* or *followed* by
    /// non-whitespace tokens.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.push();
    /// tokens.append("hello");
    /// tokens.push();
    /// tokens.append("world");
    /// tokens.push();
    ///
    /// assert_eq!(
    ///     vec![
    ///         "hello",
    ///         "world"
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&mut self) {
        // Already a push or an empty line in the stream.
        // Another one will do nothing.
        match self.items.last() {
            Some(Item::Push) | Some(Item::Line) => return,
            _ => (),
        }

        self.items.push(Item::Push);
    }

    /// Add a single line operation.
    ///
    /// A line ensures that any following tokens have one line of separation
    /// between them and the preceeding tokens.
    ///
    /// A line has no effect unless it's *preceeded* and *followed* by
    /// non-whitespace tokens.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.line();
    /// tokens.append("hello");
    /// tokens.line();
    /// tokens.append("world");
    /// tokens.line();
    ///
    /// assert_eq!(
    ///     vec![
    ///         "hello",
    ///         "",
    ///         "world"
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn line(&mut self) {
        match self.items.pop() {
            Some(Item::Push) | Some(Item::Line) | None => self.items.push(Item::Line),
            Some(other) => {
                self.items.push(other);
                self.items.push(Item::Line);
            }
        }
    }

    /// Increase the indentation of the token stream.
    ///
    /// An indentation is a language-specific operation which adds whitespace to
    /// the beginning of a line preceeding any non-whitespace tokens.
    ///
    /// An indentation has no effect unless it's *followed* by non-whitespace
    /// tokens. It also acts like a [push], in that it will shift any tokens to
    /// a new line.
    ///
    /// [push]: Self::push
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.indent();
    /// tokens.append("hello");
    /// tokens.indent();
    /// tokens.append("world");
    /// tokens.indent();
    /// tokens.append("😀");
    ///
    /// assert_eq!(
    ///     vec![
    ///         "    hello",
    ///         "        world",
    ///         "            😀",
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn indent(&mut self) {
        let n = match self.items.pop() {
            None => NonZeroI16::new(1),
            Some(Item::Indentation(level)) => NonZeroI16::new(level.get() + 1),
            Some(item) => {
                self.items.push(item);
                NonZeroI16::new(1)
            }
        };

        if let Some(n) = n {
            self.items.push(Item::Indentation(n));
        }
    }

    /// Decrease the indentation of the token stream.
    ///
    /// An indentation is a language-specific operation which adds whitespace to
    /// the beginning of a line preceeding any non-whitespace tokens.
    ///
    /// An indentation has no effect unless it's *followed* by non-whitespace
    /// tokens. It also acts like a [push], in that it will shift any tokens to
    /// a new line.
    ///
    /// Indentation can never go below zero, and will just be ignored if that
    /// were to happen. However, negative indentation is stored in the token
    /// stream, so any negative indentation in place will have to be countered
    /// before indentation starts again.
    ///
    /// [push]: Self::push
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.indent();
    /// tokens.append("hello");
    /// tokens.unindent();
    /// tokens.append("world");
    /// tokens.unindent();
    /// tokens.append("😀");
    /// tokens.indent();
    /// tokens.append("😁");
    /// tokens.indent();
    /// tokens.append("😂");
    ///
    /// assert_eq!(
    ///     vec![
    ///         "    hello",
    ///         "world",
    ///         "😀",
    ///         "😁",
    ///         "    😂",
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn unindent(&mut self) {
        let n = match self.items.pop() {
            None => NonZeroI16::new(-1),
            Some(Item::Indentation(level)) => NonZeroI16::new(level.get() - 1),
            Some(item) => {
                self.items.push(item);
                NonZeroI16::new(-1)
            }
        };

        if let Some(n) = n {
            self.items.push(Item::Indentation(n));
        }
    }

    /// Formatting function for token streams that gives full control over the
    /// formatting environment.
    ///
    /// The configurations and `format` arguments will be provided to all
    /// registered language items as well, and can be used to customize
    /// formatting through [LangItem::format].
    ///
    /// The `format` argument is primarily used internally by
    /// [Lang::format_file] to provide intermediate state that can be affect how
    /// language items are formatter. So formatting something as a file might
    /// yield different results than using this raw formatting function.
    ///
    /// Available formatters:
    ///
    /// * [fmt::VecWriter] - To write result into a vector.
    /// * [fmt::FmtWriter] - To write the result into something implementing
    ///   [fmt::Write][std::fmt::Write].
    /// * [fmt::IoWriter]- To write the result into something implementing
    ///   [io::Write][std::io::Write].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// # fn main() -> fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = #map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// let stdout = std::io::stdout();
    /// let mut w = fmt::IoWriter::new(stdout.lock());
    ///
    /// let fmt_config = fmt::Config::from_lang::<Rust>().with_indentation(2);
    /// let mut formatter = w.as_formatter(fmt_config);
    /// let config = rust::Config::default();
    ///
    /// // Default format state for Rust.
    /// let format = rust::Format::default();
    ///
    /// tokens.format(&mut formatter, &config, &format)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn format(
        &self,
        out: &mut fmt::Formatter<'_>,
        config: &L::Config,
        format: &L::Format,
    ) -> fmt::Result {
        for element in &self.items {
            element.format(out, config, format)?;
        }

        Ok(())
    }

    /// File formatting function for token streams that gives full control over the
    /// formatting environment.
    ///
    /// File formatting will render preambles like namespace declarations and
    /// imports.
    ///
    /// Available formatters:
    ///
    /// * [fmt::VecWriter] - To write result into a vector.
    /// * [fmt::FmtWriter] - To write the result into something implementing
    ///   [fmt::Write][std::fmt::Write].
    /// * [fmt::IoWriter]- To write the result into something implementing
    ///   [io::Write][std::io::Write].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = #map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// let stdout = std::io::stdout();
    /// let mut w = fmt::IoWriter::new(stdout.lock());
    ///
    /// let fmt_config = fmt::Config::from_lang::<Rust>().with_indentation(2);
    /// let mut formatter = w.as_formatter(fmt_config);
    /// let config = rust::Config::default();
    ///
    /// tokens.format_file(&mut formatter, &config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn format_file(&self, out: &mut fmt::Formatter<'_>, config: &L::Config) -> fmt::Result {
        L::format_file(self, out, &config)?;
        Ok(())
    }
}

impl<C: Default, L: Lang<Config = C>> Tokens<L> {
    /// Format the token stream as a file for the given target language to a
    /// string using the default configuration.
    ///
    /// This is a shorthand to using [FmtWriter][fmt::FmtWriter] directly in
    /// combination with [format][Self::format_file].
    ///
    /// This function will render imports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = #map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// assert_eq!(
    ///     "use std::collections::HashMap;\n\nlet mut m = HashMap::new();\nm.insert(1u32, 2u32);",
    ///     tokens.to_file_string()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_file_string(&self) -> fmt::Result<String> {
        let mut w = fmt::FmtWriter::new(String::new());
        let mut formatter = fmt::Formatter::new(&mut w, fmt::Config::from_lang::<L>());
        let config = L::Config::default();
        self.format_file(&mut formatter, &config)?;
        Ok(w.into_inner())
    }

    /// Format only the current token stream as a string using the default
    /// configuration.
    ///
    /// This is a shorthand to using [FmtWriter][fmt::FmtWriter] directly in
    /// combination with [format][Self::format].
    ///
    /// This function _will not_ render imports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = #map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// assert_eq!(
    ///     "let mut m = HashMap::new();\nm.insert(1u32, 2u32);",
    ///     tokens.to_string()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_string(self) -> fmt::Result<String> {
        let mut w = fmt::FmtWriter::new(String::new());
        let mut formatter = fmt::Formatter::new(&mut w, fmt::Config::from_lang::<L>());
        let config = L::Config::default();
        let format = L::Format::default();
        self.format(&mut formatter, &config, &format)?;
        Ok(w.into_inner())
    }

    /// Format tokens into a vector, where each entry equals a line in the
    /// resulting file using the default configuration.
    ///
    /// This is a shorthand to using [VecWriter][fmt::VecWriter] directly in
    /// combination with [format][Self::format_file].
    ///
    /// This function will render imports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = #map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::collections::HashMap;",
    ///         "",
    ///         "let mut m = HashMap::new();",
    ///         "m.insert(1u32, 2u32);"
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_file_vec(&self) -> fmt::Result<Vec<String>> {
        let mut w = fmt::VecWriter::new();
        let mut formatter = fmt::Formatter::new(&mut w, fmt::Config::from_lang::<L>());
        let config = L::Config::default();
        self.format_file(&mut formatter, &config)?;
        Ok(w.into_vec())
    }

    /// Helper function to format tokens into a vector, where each entry equals
    /// a line using the default configuration.
    ///
    /// This is a shorthand to using [VecWriter][fmt::VecWriter] directly in
    /// combination with [format][Self::format].
    ///
    /// This function _will not_ render imports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = #map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "let mut m = HashMap::new();",
    ///         "m.insert(1u32, 2u32);"
    ///     ],
    ///     tokens.to_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_vec(self) -> fmt::Result<Vec<String>> {
        let mut w = fmt::VecWriter::new();
        let mut formatter = fmt::Formatter::new(&mut w, fmt::Config::from_lang::<L>());
        let config = L::Config::default();
        let format = L::Format::default();
        self.format(&mut formatter, &config, &format)?;
        Ok(w.into_vec())
    }
}

impl<L> Clone for Tokens<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl<L> cmp::PartialEq for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<'a, L> cmp::PartialEq<Vec<Item<L>>> for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &Vec<Item<L>>) -> bool {
        self.items == *other
    }
}

impl<'a, L> cmp::PartialEq<Tokens<L>> for Vec<Item<L>>
where
    L: Lang,
{
    fn eq(&self, other: &Tokens<L>) -> bool {
        *self == other.items
    }
}

impl<'a, L> cmp::PartialEq<[Item<L>]> for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &[Item<L>]) -> bool {
        &*self.items == other
    }
}

impl<'a, L> cmp::PartialEq<Tokens<L>> for [Item<L>]
where
    L: Lang,
{
    fn eq(&self, other: &Tokens<L>) -> bool {
        self == &*other.items
    }
}

impl<L> cmp::Eq for Tokens<L> where L: Lang {}

/// Iterator over [Tokens].
///
/// This is created using [Tokens::into_iter()].
pub struct IntoIter<L>
where
    L: Lang,
{
    iter: vec::IntoIter<Item<L>>,
}

impl<L> Iterator for IntoIter<L>
where
    L: Lang,
{
    type Item = Item<L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<L> IntoIterator for Tokens<L>
where
    L: Lang,
{
    type Item = Item<L>;
    type IntoIter = IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/// Iterator over [Tokens].
///
/// This is created using [Tokens::iter()].
pub struct Iter<'a, L>
where
    L: Lang,
{
    iter: slice::Iter<'a, Item<L>>,
}

impl<'a, L: 'a> Iterator for Iter<'a, L>
where
    L: Lang,
{
    type Item = &'a Item<L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, L> IntoIterator for &'a Tokens<L>
where
    L: Lang,
{
    type Item = &'a Item<L>;
    type IntoIter = Iter<'a, L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, L: 'a> FromIterator<&'a Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = &'a Item<L>>>(iter: I) -> Tokens<L> {
        let mut tokens = Tokens::new();
        tokens.extend(iter.into_iter().cloned());
        tokens
    }
}

impl<L> FromIterator<Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = Item<L>>>(iter: I) -> Tokens<L> {
        let mut tokens = Tokens::new();
        tokens.extend(iter.into_iter());
        tokens
    }
}

/// An iterator over language-specific imported items.
///
/// Constructed using the [Tokens::walk_imports] method.
pub struct WalkImports<'a, L>
where
    L: Lang,
{
    queue: std::slice::Iter<'a, Item<L>>,
}

impl<'a, L> Iterator for WalkImports<'a, L>
where
    L: Lang,
{
    type Item = &'a L::Import;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.queue.next() {
            let import = match next {
                Item::LangBox(item) => item.as_import(),
                Item::Registered(item) => item.as_import(),
                _ => continue,
            };

            if let Some(import) = import {
                return Some(import);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate as genco;
    use crate::fmt;
    use crate::{quote, Tokens};

    /// Own little custom language for this test.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Import(u32);

    impl_lang_item! {
        impl FormatInto<Lang> for Import;
        impl From<Import> for LangBox<Lang>;

        impl LangItem<Lang> for Import {
            fn format(&self, out: &mut fmt::Formatter<'_>, _: &(), _: &()) -> fmt::Result {
                use std::fmt::Write as _;
                write!(out, "{}", self.0)
            }

            fn as_import(&self) -> Option<&Self> {
                Some(self)
            }
        }
    }

    #[derive(Clone, Copy)]
    struct Lang(());

    impl crate::Lang for Lang {
        type Config = ();
        type Format = ();
        type Import = Import;
    }

    #[test]
    fn test_walk_custom() {
        let toks: Tokens<Lang> = quote! {
            1:1 #(Import(1)) 1:2
            bar
            2:1 2:2 #(quote!(3:1 3:2)) #(Import(2))
            #(String::from("nope"))
        };

        let output: Vec<_> = toks.walk_imports().cloned().collect();

        let expected = vec![Import(1), Import(2)];

        assert_eq!(expected, output);
    }
}
