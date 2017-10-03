//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use super::formatter::Formatter;
use super::element::Element;
use super::write_tokens::WriteTokens;
use std::vec;
use std::collections::LinkedList;
use super::custom::Custom;
use std::fmt;
use std::result;
use std::slice;

/// A set of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens<'element, C> {
    elements: Vec<Element<'element, C>>,
}

/// Generic methods.
impl<'element, C> Tokens<'element, C> {
    /// Create a new set of tokens.
    pub fn new() -> Tokens<'element, C> {
        Tokens { elements: Vec::new() }
    }

    /// Push a nested definition.
    pub fn nested<T>(&mut self, tokens: T)
    where
        T: Into<Tokens<'element, C>>,
    {
        self.elements.push(Element::Nested(tokens.into()));
    }

    /// Push a definition, guaranteed to be preceeded with one newline.
    pub fn push<T>(&mut self, tokens: T)
    where
        T: Into<Tokens<'element, C>>,
    {
        self.elements.push(Element::Push(tokens.into()));
    }

    /// Append the given element.
    pub fn append<E>(&mut self, element: E)
    where
        E: Into<Element<'element, C>>,
    {
        self.elements.push(element.into());
    }

    /// Extend with another set of tokens.
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Element<'element, C>>,
    {
        self.elements.extend(it);
    }

    /// Walk over all elements.
    pub fn walk_custom(&'element self) -> WalkCustomIter<'element, C> {
        let mut queue = LinkedList::new();
        queue.push_back(self);

        WalkCustomIter {
            queue: queue,
            current: None,
        }
    }
}

impl<'element, C: Custom> Tokens<'element, C> {
    /// Format the tokens.
    pub fn format(&self, out: &mut Formatter, extra: &mut C::Extra, level: usize) -> fmt::Result {
        for element in &self.elements {
            element.format(out, extra, level)?;
        }

        Ok(())
    }
}

impl<'element, C: Custom<Extra = ()>> Tokens<'element, C> {
    /// Format the tokens.
    pub fn to_string(self) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        output.write_tokens(self, &mut ()).unwrap();
        Ok(output)
    }
}

/// Methods only available for clonable elements.
impl<'element, C: Clone> Tokens<'element, C> {
    /// Join the set of tokens on the given element.
    pub fn join<E>(self, element: E) -> Tokens<'element, C>
    where
        E: Into<Element<'element, C>>,
    {
        let element = element.into();

        let len = self.elements.len();
        let mut it = self.elements.into_iter();
        let mut out = Vec::with_capacity(usize::max(0, len + len - 1));

        if let Some(first) = it.next() {
            out.push(first);
        } else {
            return Tokens { elements: out };
        }

        while let Some(next) = it.next() {
            out.push(element.clone());
            out.push(next);
        }

        Tokens { elements: out }
    }

    /// Join with spacing.
    pub fn join_spacing(self) -> Tokens<'element, C> {
        self.join(Element::Spacing)
    }

    /// Join with line spacing.
    pub fn join_line_spacing(self) -> Tokens<'element, C> {
        self.join(Element::LineSpacing)
    }
}

/// Permit iterating over elements in tokens.
impl<'element, C: 'element> IntoIterator for Tokens<'element, C> {
    type Item = Element<'element, C>;
    type IntoIter = vec::IntoIter<Element<'element, C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

/// Convert custom elements.
impl<'element, C: Custom> From<C> for Tokens<'element, C> {
    fn from(value: C) -> Self {
        Tokens { elements: vec![Element::Custom(value)] }
    }
}

/// Convert strings.
impl<'element, C> From<&'element str> for Tokens<'element, C> {
    fn from(value: &'element str) -> Self {
        Tokens { elements: vec![Element::BorrowedLiteral(value)] }
    }
}

/// Convert already elements.
impl<'element, C> From<Element<'element, C>> for Tokens<'element, C> {
    fn from(value: Element<'element, C>) -> Self {
        Tokens { elements: vec![value] }
    }
}

pub struct WalkCustomIter<'element, C: 'static> {
    queue: LinkedList<&'element Tokens<'element, C>>,
    current: Option<slice::Iter<'element, Element<'element, C>>>,
}

impl<'element, C: 'static> Iterator for WalkCustomIter<'element, C> {
    type Item = &'element C;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Element::*;

        loop {
            match self.current {
                Some(ref mut current) => {
                    // read all nested tokens and queue them
                    // return any elements encountered.
                    while let Some(next) = current.next() {
                        match next {
                            &Append(ref tokens) |
                            &Push(ref tokens) |
                            &Nested(ref tokens) => {
                                self.queue.push_back(tokens);
                                continue;
                            }
                            &Custom(ref custom) => return Some(custom),
                            _ => continue,
                        }
                    }

                    // fall through to set self.current to None
                }
                None => {
                    if let Some(next) = self.queue.pop_front() {
                        self.current = Some(next.elements.iter());
                    }

                    match self.current {
                        Some(_) => continue,
                        None => return None,
                    }
                }
            }

            self.current = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tokens;
    use custom::Custom;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Lang(u32);

    impl Custom for Lang {
        type Extra = ();
    }

    #[test]
    fn test_join() {
        let s = String::from("foo");
        let mut toks: Tokens<()> = Tokens::new();

        // locally borrowed string
        toks.append(s.as_str());
        // static string
        toks.append("bar");
        // owned literal
        toks.append(String::from("nope"));

        let toks = toks.join_spacing();

        assert_eq!("foo bar nope", toks.to_string().unwrap().as_str());
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
