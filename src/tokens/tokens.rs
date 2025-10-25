//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let mut toks = java::Tokens::new();
//! toks.append("foo");
//! ```
#![allow(clippy::module_inception)]

use core::cmp::Ordering;
use core::hash;
use core::iter::FromIterator;
use core::mem;
use core::slice;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::{self, Vec};

use crate::fmt;
use crate::lang::{Lang, LangSupportsEval};
use crate::tokens::ItemStr;
use crate::tokens::{FormatInto, Item, Kind, Register};

/// A stream of tokens.
///
/// # Structural Guarantees
///
/// This stream of tokens provides the following structural guarantees.
///
/// * Only one [`space`] occurs in sequence and indicates spacing between
///   tokens.
/// * Only one [`push`] occurs in sequence and indicates that the next token
///   should be spaced onto a new line.
/// * A [`line`] is never by a [`push`] since it would have no effect. A line
///   ensures an empty line between two tokens.
///
/// ```
/// use genco::Tokens;
/// use genco::tokens::Item;
///
/// let mut tokens = Tokens::<()>::new();
///
/// // The first push token is "overriden" by a line.
/// tokens.space();
/// tokens.space();
///
/// assert_eq!(tokens, [Item::space()]);
///
/// let mut tokens = Tokens::<()>::new();
///
/// tokens.space();
/// tokens.push();
/// tokens.push();
///
/// assert_eq!(tokens, [Item::push()]);
///
/// let mut tokens = Tokens::<()>::new();
///
/// // The first space and push tokens are "overriden" by a line.
/// tokens.space();
/// tokens.push();
/// tokens.line();
///
/// assert_eq!(tokens, [Item::line()]);
/// ```
///
/// [`space`]: Self::space
/// [`push`]: Self::push
/// [`line`]: Self::line
pub struct Tokens<L = ()>
where
    L: Lang,
{
    items: Vec<(usize, Item<L>)>,
    /// The last position at which we observed a language item.
    ///
    /// This references the `position + 1` in the items vector. A position of 0
    /// means that there are no more items.
    ///
    /// This makes up a singly-linked list over all language items that you can
    /// follow.
    last_lang_item: usize,
}

impl<L> Tokens<L>
where
    L: Lang,
{
    /// Create a new empty stream of tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let tokens = Tokens::<()>::new();
    ///
    /// assert!(tokens.is_empty());
    /// ```
    pub fn new() -> Self {
        Tokens {
            items: Vec::new(),
            last_lang_item: 0,
        }
    }

    /// Create a new empty stream of tokens with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let tokens = Tokens::<()>::with_capacity(10);
    ///
    /// assert!(tokens.is_empty());
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        Tokens {
            items: Vec::with_capacity(cap),
            last_lang_item: 0,
        }
    }

    /// Construct an iterator over the token stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    /// use genco::tokens::{ItemStr, Item};
    ///
    /// let tokens: Tokens<()> = quote!(foo bar baz);
    /// let mut it = tokens.iter();
    ///
    /// assert_eq!(Some(&Item::literal(ItemStr::static_("foo"))), it.next());
    /// assert_eq!(Some(&Item::space()), it.next());
    /// assert_eq!(Some(&Item::literal(ItemStr::static_("bar"))), it.next());
    /// assert_eq!(Some(&Item::space()), it.next());
    /// assert_eq!(Some(&Item::literal(ItemStr::static_("baz"))), it.next());
    /// assert_eq!(None, it.next());
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, L> {
        Iter {
            iter: self.items.iter(),
        }
    }

    /// Append the given tokens.
    ///
    /// This append function takes anything implementing [`FormatInto`] making
    /// the argument's behavior customizable. Most primitive types have built-in
    /// implementations of [`FormatInto`] treating them as raw tokens.
    ///
    /// Most notabley, things implementing [`FormatInto`] can be used as
    /// arguments for [interpolation] in the [`quote!`] macro.
    ///
    /// [`quote!`]: crate::quote
    /// [interpolation]: crate::quote#interpolation
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let mut tokens = Tokens::<()>::new();
    /// tokens.append(4u32);
    ///
    /// assert_eq!(quote!($(4u32)), tokens);
    /// ```
    pub fn append<T>(&mut self, tokens: T)
    where
        T: FormatInto<L>,
    {
        tokens.format_into(self)
    }

    /// Extend with another stream of tokens.
    ///
    /// This respects the structural requirements of adding one element at a
    /// time, like you would get by calling [`space`], [`push`], or [`line`].
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    /// use genco::tokens::{Item, ItemStr};
    ///
    /// let mut tokens: Tokens<()> = quote!(foo bar);
    /// tokens.extend::<Tokens<()>>(quote!($[' ']baz));
    ///
    /// assert_eq!(tokens, quote!(foo bar baz));
    /// ```
    ///
    /// [`space`]: Self::space
    /// [`push`]: Self::push
    /// [`line`]: Self::line
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Item<L>>,
    {
        let it = it.into_iter();
        let (low, high) = it.size_hint();
        self.items.reserve(high.unwrap_or(low));

        for item in it {
            self.item(item);
        }
    }

    /// Iterate over all registered [`Lang`] items.
    ///
    /// The order in which the imports are returned is *not* defined. So if you
    /// need them in some particular order you have to sort them.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let debug = rust::import("std::fmt", "Debug");
    /// let ty = rust::import("std::collections", "HashMap");
    ///
    /// let tokens = quote!(foo $ty<u32, dyn $debug> baz);
    ///
    /// for import in tokens.iter_lang() {
    ///     println!("{:?}", import);
    /// }
    /// ```
    pub fn iter_lang(&self) -> IterLang<'_, L> {
        IterLang {
            items: &self.items,
            pos: self.last_lang_item,
        }
    }

    /// Add an registered custom element that is _not_ rendered.
    ///
    /// Registration can be used to generate imports that do not render a
    /// visible result.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let write_bytes_ext = rust::import("byteorder", "WriteBytesExt").with_alias("_");
    ///
    /// let tokens = quote!($(register(write_bytes_ext)));
    ///
    /// assert_eq!("use byteorder::WriteBytesExt as _;\n", tokens.to_file_string()?);
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn register<T>(&mut self, tokens: T)
    where
        T: Register<L>,
    {
        tokens.register(self);
    }

    /// Check if tokens contain no items.
    ///
    /// ```
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
    /// ```
    /// use genco::prelude::*;
    ///
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn space(&mut self) {
        if let Some((_, Item { kind: Kind::Space })) = self.items.last() {
            return;
        }

        self.items.push((0, Item::space()));
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
    /// ```
    /// use genco::prelude::*;
    ///
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn push(&mut self) {
        let item = loop {
            let Some((o, item)) = self.items.pop() else {
                break None;
            };

            match &item.kind {
                // NB: never reconfigure a line into a push.
                Kind::Line => {
                    self.items.push((o, item));
                    return;
                }
                Kind::Space | Kind::Push => continue,
                _ => break Some((o, item)),
            }
        };

        self.items.extend(item);
        self.items.push((0, Item::push()));
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
    /// ```
    /// use genco::prelude::*;
    ///
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn line(&mut self) {
        let item = loop {
            let Some((o, item)) = self.items.pop() else {
                break None;
            };

            if matches!(item.kind, Kind::Line | Kind::Push) {
                continue;
            }

            break Some((o, item));
        };

        self.items.extend(item);
        self.items.push((0, Item::line()));
    }

    /// Increase the indentation of the token stream.
    ///
    /// An indentation is a language-specific operation which adds whitespace to
    /// the beginning of a line preceeding any non-whitespace tokens.
    ///
    /// An indentation has no effect unless it's *followed* by non-whitespace
    /// tokens. It also acts like a [`push`], in that it will shift any tokens to
    /// a new line.
    ///
    /// [`push`]: Self::push
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn indent(&mut self) {
        self.indentation(1);
    }

    /// Decrease the indentation of the token stream.
    ///
    /// An indentation is a language-specific operation which adds whitespace to
    /// the beginning of a line preceeding any non-whitespace tokens.
    ///
    /// An indentation has no effect unless it's *followed* by non-whitespace
    /// tokens. It also acts like a [`push`], in that it will shift any tokens to
    /// a new line.
    ///
    /// Indentation can never go below zero, and will just be ignored if that
    /// were to happen. However, negative indentation is stored in the token
    /// stream, so any negative indentation in place will have to be countered
    /// before indentation starts again.
    ///
    /// [`push`]: Self::push
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn unindent(&mut self) {
        self.indentation(-1);
    }

    /// Formatting function for token streams that gives full control over the
    /// formatting environment.
    ///
    /// The configurations and `format` arguments will be provided to all
    /// registered language items as well, and can be used to customize
    /// formatting through [LangItem::format()].
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
    /// ```,no_run
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// let map = rust::import("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = $map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// let stdout = std::io::stdout();
    /// let mut w = fmt::IoWriter::new(stdout.lock());
    ///
    /// let fmt = fmt::Config::from_lang::<Rust>()
    ///     .with_indentation(fmt::Indentation::Space(2));
    /// let mut formatter = w.as_formatter(&fmt);
    /// let config = rust::Config::default();
    ///
    /// // Default format state for Rust.
    /// let format = rust::Format::default();
    ///
    /// tokens.format(&mut formatter, &config, &format)?;
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    ///
    /// [LangItem::format()]: crate::lang::LangItem::format()
    pub fn format(
        &self,
        out: &mut fmt::Formatter<'_>,
        config: &L::Config,
        format: &L::Format,
    ) -> fmt::Result {
        out.format_items(&self.items, config, format)
    }

    /// Push a literal item to the stream.
    #[inline]
    pub(crate) fn literal(&mut self, lit: impl Into<ItemStr>) {
        self.items.push((0, Item::literal(lit.into())));
    }

    /// Push an open quote item to the stream.
    #[inline]
    pub(crate) fn open_quote(&mut self, is_interpolated: bool) {
        self.items.push((0, Item::open_quote(is_interpolated)));
    }

    /// Push a close quote item to the stream.
    #[inline]
    pub(crate) fn close_quote(&mut self) {
        self.items.push((0, Item::close_quote()));
    }

    /// Push a single item to the stream while checking for structural
    /// guarantees.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    /// use genco::tokens::{Item, ItemStr};
    ///
    /// let mut tokens = Tokens::<()>::new();
    ///
    /// tokens.append(ItemStr::static_("foo"));
    /// tokens.space();
    /// tokens.space(); // Note: second space ignored
    /// tokens.append(ItemStr::static_("bar"));
    ///
    /// assert_eq!(tokens, quote!(foo bar));
    /// ```
    pub(crate) fn item(&mut self, item: Item<L>) {
        match item.kind {
            Kind::Push => self.push(),
            Kind::Line => self.line(),
            Kind::Space => self.space(),
            Kind::Indentation(n) => self.indentation(n),
            Kind::Lang(item) => self.lang_item(item),
            Kind::Register(item) => self.lang_item_register(item),
            other => self.items.push((0, Item::new(other))),
        }
    }

    /// Add a language item directly.
    pub(crate) fn lang_item(&mut self, item: Box<L::Item>) {
        // NB: recorded position needs to be adjusted.
        self.items.push((self.last_lang_item, Item::lang(item)));
        self.last_lang_item = self.items.len();
    }

    /// Register a language item directly.
    pub(crate) fn lang_item_register(&mut self, item: Box<L::Item>) {
        // NB: recorded position needs to be adjusted.
        self.items.push((self.last_lang_item, Item::register(item)));
        self.last_lang_item = self.items.len();
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
    /// ```,no_run
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// let map = rust::import("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = $map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// let stdout = std::io::stdout();
    /// let mut w = fmt::IoWriter::new(stdout.lock());
    ///
    /// let fmt = fmt::Config::from_lang::<Rust>()
    ///     .with_indentation(fmt::Indentation::Space(2));
    /// let mut formatter = w.as_formatter(&fmt);
    /// let config = rust::Config::default();
    ///
    /// tokens.format_file(&mut formatter, &config)?;
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn format_file(&self, out: &mut fmt::Formatter<'_>, config: &L::Config) -> fmt::Result {
        L::format_file(self, out, config)?;
        out.write_trailing_line()?;
        Ok(())
    }

    /// Internal function to modify the indentation of the token stream.
    fn indentation(&mut self, mut n: i16) {
        let item = loop {
            // flush all whitespace preceeding the indentation change.
            let Some((o, item)) = self.items.pop() else {
                break None;
            };

            match &item.kind {
                Kind::Push => continue,
                Kind::Space => continue,
                Kind::Line => continue,
                Kind::Indentation(u) => n += u,
                _ => break Some((o, item)),
            }
        };

        self.items.extend(item);

        if n != 0 {
            self.items.push((0, Item::new(Kind::Indentation(n))));
        }
    }
}

impl<L> Default for Tokens<L>
where
    L: Lang,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<L> Tokens<L>
where
    L: LangSupportsEval,
{
    /// Helper function to determine if the token stream supports evaluation at compile time.
    #[doc(hidden)]
    #[inline]
    pub fn lang_supports_eval(&self) {}
}

impl<L> Tokens<L>
where
    L: Lang,
    L::Config: Default,
{
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
    /// ```
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// let map = rust::import("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = $map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// assert_eq!(
    ///     "use std::collections::HashMap;\n\nlet mut m = HashMap::new();\nm.insert(1u32, 2u32);\n",
    ///     tokens.to_file_string()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn to_file_string(&self) -> fmt::Result<String> {
        let mut w = fmt::FmtWriter::new(String::new());
        let fmt = fmt::Config::from_lang::<L>();
        let mut formatter = w.as_formatter(&fmt);
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
    /// ```
    /// use genco::prelude::*;
    ///
    /// let map = rust::import("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = $map::new();
    ///     m.insert(1u32, 2u32);
    /// };
    ///
    /// assert_eq!(
    ///     "let mut m = HashMap::new();\nm.insert(1u32, 2u32);",
    ///     tokens.to_string()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn to_string(&self) -> fmt::Result<String> {
        let mut w = fmt::FmtWriter::new(String::new());
        let fmt = fmt::Config::from_lang::<L>();
        let mut formatter = w.as_formatter(&fmt);
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
    /// ```
    /// use genco::prelude::*;
    ///
    /// let map = rust::import("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = $map::new();
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    ///
    /// # Example with Python indentation
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let tokens: python::Tokens = quote! {
    ///     def foo():
    ///         pass
    ///
    ///     def bar():
    ///         pass
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "def foo():",
    ///         "    pass",
    ///         "",
    ///         "def bar():",
    ///         "    pass",
    ///     ],
    ///     tokens.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn to_file_vec(&self) -> fmt::Result<Vec<String>> {
        let mut w = fmt::VecWriter::new();
        let fmt = fmt::Config::from_lang::<L>();
        let mut formatter = w.as_formatter(&fmt);
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
    /// ```
    /// use genco::prelude::*;
    ///
    /// let map = rust::import("std::collections", "HashMap");
    ///
    /// let tokens: rust::Tokens = quote! {
    ///     let mut m = $map::new();
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
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn to_vec(&self) -> fmt::Result<Vec<String>> {
        let mut w = fmt::VecWriter::new();
        let fmt = fmt::Config::from_lang::<L>();
        let mut formatter = w.as_formatter(&fmt);
        let config = L::Config::default();
        let format = L::Format::default();
        self.format(&mut formatter, &config, &format)?;
        Ok(w.into_vec())
    }
}

impl<L> PartialEq<Tokens<L>> for Tokens<L>
where
    L: Lang,
    L::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Tokens<L>) -> bool {
        self.items == other.items
    }
}

impl<L> PartialEq<Vec<Item<L>>> for Tokens<L>
where
    L: Lang,
    L::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Vec<Item<L>>) -> bool {
        self == &other[..]
    }
}

impl<L> PartialEq<Tokens<L>> for Vec<Item<L>>
where
    L: Lang,
    L::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Tokens<L>) -> bool {
        other == &self[..]
    }
}

impl<L> PartialEq<[Item<L>]> for Tokens<L>
where
    L: Lang,
    L::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &[Item<L>]) -> bool {
        self.iter().eq(other)
    }
}

impl<L, const N: usize> PartialEq<[Item<L>; N]> for Tokens<L>
where
    L: Lang,
    L::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &[Item<L>; N]) -> bool {
        self == &other[..]
    }
}

impl<L> PartialEq<Tokens<L>> for [Item<L>]
where
    L: Lang,
    L::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Tokens<L>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<L> Eq for Tokens<L>
where
    L: Lang,
    L::Item: Eq,
{
}

impl<L> PartialOrd<Tokens<L>> for Tokens<L>
where
    L: Lang,
    L::Item: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Tokens<L>) -> Option<Ordering> {
        self.items.iter().partial_cmp(other.items.iter())
    }
}

impl<L> Ord for Tokens<L>
where
    L: Lang,
    L::Item: Ord,
{
    #[inline]
    fn cmp(&self, other: &Tokens<L>) -> Ordering {
        self.items.iter().cmp(other.items.iter())
    }
}

/// Iterator over [Tokens].
///
/// This is created using [Tokens::into_iter()].
pub struct IntoIter<L>
where
    L: Lang,
{
    iter: vec::IntoIter<(usize, Item<L>)>,
}

impl<L> Iterator for IntoIter<L>
where
    L: Lang,
{
    type Item = Item<L>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Construct an owned iterator over the token stream.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use genco::tokens::{ItemStr, Item};
///
/// let tokens: Tokens<()> = quote!(foo bar baz);
/// let mut it = tokens.into_iter();
///
/// assert_eq!(Some(Item::literal(ItemStr::static_("foo"))), it.next());
/// assert_eq!(Some(Item::space()), it.next());
/// assert_eq!(Some(Item::literal(ItemStr::static_("bar"))), it.next());
/// assert_eq!(Some(Item::space()), it.next());
/// assert_eq!(Some(Item::literal(ItemStr::static_("baz"))), it.next());
/// assert_eq!(None, it.next());
/// ```
impl<L> IntoIterator for Tokens<L>
where
    L: Lang,
{
    type Item = Item<L>;
    type IntoIter = IntoIter<L>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.items.into_iter(),
        }
    }
}

/// Iterator over [Tokens].
///
/// This is created using [Tokens::iter()].
pub struct Iter<'a, L>
where
    L: Lang,
{
    iter: slice::Iter<'a, (usize, Item<L>)>,
}

impl<'a, L> Iterator for Iter<'a, L>
where
    L: Lang,
{
    type Item = &'a Item<L>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(&self.iter.next()?.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
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

impl<'a, L> FromIterator<&'a Item<L>> for Tokens<L>
where
    L: Lang,
    L::Item: Clone,
{
    fn from_iter<I: IntoIterator<Item = &'a Item<L>>>(iter: I) -> Self {
        let it = iter.into_iter();
        let (low, high) = it.size_hint();
        let mut tokens = Self::with_capacity(high.unwrap_or(low));
        tokens.extend(it.cloned());
        tokens
    }
}

impl<L> FromIterator<Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = Item<L>>>(iter: I) -> Self {
        let it = iter.into_iter();
        let (low, high) = it.size_hint();
        let mut tokens = Self::with_capacity(high.unwrap_or(low));
        tokens.extend(it);
        tokens
    }
}

impl<L> core::fmt::Debug for Tokens<L>
where
    L: Lang,
    L::Item: core::fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.items).finish()
    }
}

impl<L> Clone for Tokens<L>
where
    L: Lang,
    L::Item: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            last_lang_item: self.last_lang_item,
        }
    }
}

impl<L> hash::Hash for Tokens<L>
where
    L: Lang,
    L::Item: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.items.hash(state);
        self.last_lang_item.hash(state);
    }
}

/// An iterator over language-specific imported items.
///
/// Constructed using the [`Tokens::iter_lang`] method.
pub struct IterLang<'a, L>
where
    L: Lang,
{
    items: &'a [(usize, Item<L>)],
    pos: usize,
}

impl<'a, L> Iterator for IterLang<'a, L>
where
    L: Lang,
{
    type Item = &'a L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = mem::take(&mut self.pos);

        if pos == 0 {
            return None;
        }

        // NB: recorded position needs to be adjusted.
        match self.items.get(pos - 1)? {
            (
                prev,
                Item {
                    kind: Kind::Lang(item) | Kind::Register(item),
                },
            ) => {
                self.pos = *prev;
                Some(item)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write as _;

    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate as genco;
    use crate::fmt;
    use crate::{quote, Tokens};

    /// Own little custom language for this test.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Import(u32);

    impl_lang! {
        Lang {
            type Config = ();
            type Format = ();
            type Item = Any;
        }

        Import(Import) {
            fn format(&self, out: &mut fmt::Formatter<'_>, _: &(), _: &()) -> fmt::Result {
                write!(out, "{}", self.0)
            }
        }
    }

    #[test]
    fn test_walk_custom() {
        let toks: Tokens<Lang> = quote! {
            1:1 $(Import(1)) 1:2
            bar
            2:1 2:2 $(quote!(3:1 3:2)) $(Import(2))
            $(String::from("nope"))
        };

        let mut output: Vec<_> = toks.iter_lang().cloned().collect();
        output.sort();

        let expected: Vec<Any> = vec![Import(1).into(), Import(2).into()];

        assert_eq!(expected, output);
    }
}
