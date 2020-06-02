//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use crate::{Con, Config, Element, FormatTokens, Formatter, Lang, WriteTokens};
use std::collections::LinkedList;
use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;
use std::result;
use std::vec;

/// A set of tokens.
#[derive(Debug, Clone, Default)]
pub struct Tokens<'el, C: 'el> {
    pub(crate) elements: Vec<Element<'el, C>>,
}

/// Generic methods.
impl<'el, C: 'el> Tokens<'el, C> {
    /// Create a new set of tokens.
    pub fn new() -> Tokens<'el, C> {
        Tokens {
            elements: Vec::new(),
        }
    }

    /// Push a nested definition.
    pub fn nested<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, C>,
    {
        self.elements.push(Element::Indent);
        tokens.into_tokens(self);
        self.elements.push(Element::Unindent);
    }

    /// Push a nested definition.
    pub fn nested_into<B>(&mut self, builder: B) -> ()
    where
        B: FnOnce(&mut Tokens<'el, C>) -> (),
    {
        let mut t = Tokens::new();
        builder(&mut t);
        self.nested(t);
    }

    /// Push a nested definition.
    ///
    /// This is a fallible version that expected the builder to return a result.
    pub fn try_nested_into<E, B>(&mut self, builder: B) -> Result<(), E>
    where
        B: FnOnce(&mut Tokens<'el, C>) -> Result<(), E>,
    {
        let mut t = Tokens::new();
        builder(&mut t)?;
        self.nested(t);
        Ok(())
    }

    /// Push a nested reference to a definition.
    pub fn nested_ref(&mut self, tokens: &'el Tokens<'el, C>) {
        self.elements.push(Element::Indent);
        self.elements
            .extend(tokens.elements.iter().map(Element::Borrowed));
        self.elements.push(Element::Unindent);
    }

    /// Push a definition, guaranteed to be preceded with one newline.
    pub fn push<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, C>,
    {
        self.elements.push(Element::PushSpacing);
        tokens.into_tokens(self);
    }

    /// Push a new created definition, guaranteed to be preceded with one newline.
    pub fn push_into<B>(&mut self, builder: B) -> ()
    where
        B: FnOnce(&mut Tokens<'el, C>) -> (),
    {
        let mut t = Tokens::new();
        builder(&mut t);
        self.push(t);
    }

    /// Push a new created definition, guaranteed to be preceded with one newline.
    ///
    /// This is a fallible version that expected the builder to return a result.
    pub fn try_push_into<E, B>(&mut self, builder: B) -> Result<(), E>
    where
        B: FnOnce(&mut Tokens<'el, C>) -> Result<(), E>,
    {
        let mut t = Tokens::new();
        builder(&mut t)?;
        self.push(t);
        Ok(())
    }

    /// Push the given set of tokens, unless it is empty.
    ///
    /// This is useful when you wish to preserve the structure of nested and joined tokens.
    pub fn push_unless_empty<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, C>,
    {
        if !tokens.is_empty() {
            self.elements.push(Element::PushSpacing);
            tokens.into_tokens(self);
        }
    }

    /// Insert the given element.
    pub fn insert<E>(&mut self, pos: usize, element: E)
    where
        E: Into<Element<'el, C>>,
    {
        self.elements.insert(pos, element.into());
    }

    /// Append the given element.
    pub fn append<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, C>,
    {
        tokens.into_tokens(self)
    }

    /// Append a reference to a definition.
    pub fn append_ref(&mut self, element: &'el Element<'el, C>) {
        self.elements.push(Element::Borrowed(element));
    }

    /// Append the given set of tokens, unless it is empty.
    ///
    /// This is useful when you wish to preserve the structure of nested and joined tokens.
    pub fn append_unless_empty<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, C>,
    {
        if tokens.is_empty() {
            return;
        }

        tokens.into_tokens(self);
    }

    /// Extend with another set of tokens.
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Element<'el, C>>,
    {
        self.elements.extend(it.into_iter());
    }

    /// Walk over all elements.
    pub fn walk_custom(&self) -> WalkCustom<C> {
        let mut queue = LinkedList::new();
        queue.extend(self.elements.iter());
        WalkCustom { queue: queue }
    }

    /// Add an registered custom element that is _not_ rendered.
    pub fn register(&mut self, custom: C) {
        self.elements
            .push(Element::Registered(Con::Rc(Rc::new(custom))));
    }

    /// Check if tokens contain no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Add a single spacing to the token stream.
    pub fn spacing(&mut self) {
        self.elements.push(Element::Spacing);
    }

    /// Add a single line spacing to the token stream.
    pub fn line_spacing(&mut self) {
        self.elements.push(Element::LineSpacing);
    }

    /// Add a single push spacing operation.
    pub fn push_spacing(&mut self) {
        self.elements.push(Element::PushSpacing);
    }

    /// Add a single indentation to the token stream.
    pub fn indent(&mut self) {
        self.elements.push(Element::Indent);
    }

    /// Add a single unindentation to the token stream.
    pub fn unindent(&mut self) {
        self.elements.push(Element::Unindent);
    }
}

impl<'el, C> IntoIterator for Tokens<'el, C> {
    type Item = Element<'el, C>;
    type IntoIter = vec::IntoIter<Element<'el, C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'el, C: Lang<'el>> Tokens<'el, C> {
    /// Format the tokens.
    pub fn format(&self, out: &mut Formatter, config: &mut C::Config, level: usize) -> fmt::Result {
        for element in &self.elements {
            element.format(out, config, level)?;
        }

        Ok(())
    }

    /// Format token as file with the given configuration.
    pub fn to_file_with(self, mut config: C::Config) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        output.write_file(self, &mut config)?;
        Ok(output)
    }

    /// Format the tokens with the given configuration.
    pub fn to_string_with(self, mut config: C::Config) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        output.write_tokens(self, &mut config)?;
        Ok(output)
    }
}

impl<'el, E: Config + Default, C: Lang<'el, Config = E>> Tokens<'el, C> {
    /// Format token as file.
    pub fn to_file(self) -> result::Result<String, fmt::Error> {
        self.to_file_with(C::Config::default())
    }

    /// Format the tokens.
    pub fn to_string(self) -> result::Result<String, fmt::Error> {
        self.to_string_with(C::Config::default())
    }
}

impl<'el, C> FromIterator<&'el Element<'el, C>> for Tokens<'el, C> {
    fn from_iter<I: IntoIterator<Item = &'el Element<'el, C>>>(iter: I) -> Tokens<'el, C> {
        Tokens {
            elements: iter.into_iter().map(|e| Element::Borrowed(e)).collect(),
        }
    }
}

impl<'el, C> FromIterator<Element<'el, C>> for Tokens<'el, C> {
    fn from_iter<I: IntoIterator<Item = Element<'el, C>>>(iter: I) -> Tokens<'el, C> {
        Tokens {
            elements: iter.into_iter().collect(),
        }
    }
}

pub struct WalkCustom<'el, C: 'el> {
    queue: LinkedList<&'el Element<'el, C>>,
}

impl<'el, C: 'el> Iterator for WalkCustom<'el, C> {
    type Item = &'el C;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Element::*;

        // read until custom element is encountered.
        while let Some(next) = self.queue.pop_front() {
            match *next {
                Rc(ref element) => {
                    self.queue.push_back(element.as_ref());
                }
                Borrowed(ref element) => {
                    self.queue.push_back(element);
                }
                Lang(ref custom) => return Some(custom.as_ref()),
                Registered(ref custom) => return Some(custom.as_ref()),
                _ => {}
            }
        }

        Option::None
    }
}

#[cfg(test)]
mod tests {
    use crate::Tokens;

    /// Own little custom language for this test.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Lang(u32);

    impl<'el> crate::Lang<'el> for Lang {
        type Config = ();
    }

    #[test]
    fn test_walk_custom() {
        let mut toks: Tokens<Lang> = Tokens::new();

        toks.push(toks!("1:1", Lang(1), "1:2"));

        // static string
        toks.append("bar");

        toks.nested(toks!("2:1", "2:2", toks!("3:1", "3:2"), Lang(2)));

        // owned literal
        toks.append(String::from("nope"));

        let output: Vec<_> = toks.walk_custom().cloned().collect();

        let expected = vec![Lang(1), Lang(2)];

        assert_eq!(expected, output);
    }
}
