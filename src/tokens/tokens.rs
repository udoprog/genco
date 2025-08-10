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

use core::cmp;
use core::iter::FromIterator;
use core::mem;
use core::slice;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::{self, Vec};

use crate::fmt;
use crate::lang::{Lang, LangSupportsEval};
use crate::tokens::{FormatInto, Item, Register};

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
/// assert_eq!(vec![Item::Space::<()>], tokens);
///
/// let mut tokens = Tokens::<()>::new();
///
/// tokens.space();
/// tokens.push();
/// tokens.push();
///
/// assert_eq!(vec![Item::Push::<()>], tokens);
///
/// let mut tokens = Tokens::<()>::new();
///
/// // The first space and push tokens are "overriden" by a line.
/// tokens.space();
/// tokens.push();
/// tokens.line();
///
/// assert_eq!(vec![Item::Line::<()>], tokens);
/// ```
///
/// [`space`]: Self::space
/// [`push`]: Self::push
/// [`line`]: Self::line
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tokens<L = ()>
where
    L: Lang,
{
    items: Vec<Item<L>>,
    /// The last position at which we observed a language item.
    ///
    /// This references the `position + 1` in the items vector. A position of
    /// 0 means that there are no more items.
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

    /// Append the given tokens.
    ///
    /// This append function takes anything implementing [FormatInto] making the
    /// argument's behavior customizable. Most primitive types have built-in
    /// implementations of [FormatInto] treating them as raw tokens.
    ///
    /// Most notabley, things implementing [FormatInto] can be used as arguments
    /// for [interpolation] in the [quote!] macro.
    ///
    /// [quote!]: macro.quote.html
    /// [interpolation]: macro.quote.html#interpolation
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

    /// Walk over all imports.
    ///
    /// The order in which the imports are returned is *not* defined. So if you
    /// need them in some particular order you need to sort them.
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
    /// for import in tokens.walk_imports() {
    ///     println!("{:?}", import);
    /// }
    /// ```
    pub fn walk_imports(&self) -> WalkImports<'_, L> {
        WalkImports {
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
    ///
    /// [quote!]: macro.quote.html
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
            match self.items.pop() {
                // NB: never reconfigure a line into a push.
                Some(Item::Line) => {
                    self.items.push(Item::Line);
                    return;
                }
                Some(Item::Space | Item::Push) => continue,
                item => break item,
            }
        };

        self.items.extend(item);
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
            match self.items.pop() {
                Some(Item::Line) | Some(Item::Push) => continue,
                item => break item,
            }
        };

        self.items.extend(item);
        self.items.push(Item::Line);
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
    /// tokens.append(ItemStr::Static("foo"));
    /// tokens.space();
    /// tokens.space(); // Note: second space ignored
    /// tokens.append(ItemStr::Static("bar"));
    ///
    /// assert_eq!(tokens, quote!(foo bar));
    /// ```
    pub(crate) fn item(&mut self, item: Item<L>) {
        match item {
            Item::Push => self.push(),
            Item::Line => self.line(),
            Item::Space => self.space(),
            Item::Indentation(n) => self.indentation(n),
            Item::Lang(_, item) => self.lang_item(item),
            Item::Register(_, item) => self.lang_item_register(item),
            other => self.items.push(other),
        }
    }

    /// Add a language item directly.
    pub(crate) fn lang_item(&mut self, item: Box<L::Item>) {
        // NB: recorded position needs to be adjusted.
        self.items
            .push(crate::tokens::Item::Lang(self.last_lang_item, item));
        self.last_lang_item = self.items.len();
    }

    /// Register a language item directly.
    pub(crate) fn lang_item_register(&mut self, item: Box<L::Item>) {
        // NB: recorded position needs to be adjusted.
        self.items
            .push(crate::tokens::Item::Register(self.last_lang_item, item));
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
            match self.items.pop() {
                Some(Item::Push) => continue,
                Some(Item::Space) => continue,
                Some(Item::Line) => continue,
                Some(Item::Indentation(u)) => n += u,
                item => break item,
            }
        };

        self.items.extend(item);

        if n != 0 {
            self.items.push(Item::Indentation(n));
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

impl<L> cmp::PartialEq<Vec<Item<L>>> for Tokens<L>
where
    L: Lang,
{
    #[inline]
    fn eq(&self, other: &Vec<Item<L>>) -> bool {
        self.items == *other
    }
}

impl<L> cmp::PartialEq<Tokens<L>> for Vec<Item<L>>
where
    L: Lang,
{
    fn eq(&self, other: &Tokens<L>) -> bool {
        *self == other.items
    }
}

impl<L> cmp::PartialEq<[Item<L>]> for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &[Item<L>]) -> bool {
        &*self.items == other
    }
}

impl<L> cmp::PartialEq<Tokens<L>> for [Item<L>]
where
    L: Lang,
{
    fn eq(&self, other: &Tokens<L>) -> bool {
        self == &*other.items
    }
}

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
/// assert_eq!(Some(Item::Literal(ItemStr::Static("foo"))), it.next());
/// assert_eq!(Some(Item::Space), it.next());
/// assert_eq!(Some(Item::Literal(ItemStr::Static("bar"))), it.next());
/// assert_eq!(Some(Item::Space), it.next());
/// assert_eq!(Some(Item::Literal(ItemStr::Static("baz"))), it.next());
/// assert_eq!(None, it.next());
/// ```
impl<L> IntoIterator for Tokens<L>
where
    L: Lang,
{
    type Item = Item<L>;
    type IntoIter = IntoIter<L>;

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
    iter: slice::Iter<'a, Item<L>>,
}

impl<'a, L> Iterator for Iter<'a, L>
where
    L: Lang,
{
    type Item = &'a Item<L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

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

/// An iterator over language-specific imported items.
///
/// Constructed using the [Tokens::walk_imports] method.
pub struct WalkImports<'a, L>
where
    L: Lang,
{
    items: &'a [Item<L>],
    pos: usize,
}

impl<'a, L> Iterator for WalkImports<'a, L>
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
        let item = self.items.get(pos - 1)?;

        let (prev, item) = match item {
            Item::Lang(prev, item) => (prev, item),
            Item::Register(prev, item) => (prev, item),
            _ => return None,
        };

        self.pos = *prev;
        Some(item)
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

        Import {
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

        let mut output: Vec<_> = toks.walk_imports().cloned().collect();
        output.sort();

        let expected = vec![Any::Import(Import(1)), Any::Import(Import(2))];

        assert_eq!(expected, output);
    }
}
