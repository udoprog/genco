//! Macros in GenCo

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// ## Examples
///
/// ```rust,ignore
/// # #![allow(deprecated)]
/// let n1: genco::Tokens<()> = toks!("var v = ", "bar".quoted(), ";");
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! toks {
    ($($x:expr),*) => {
        {
            let mut _t = $crate::Tokens::new();
            $(_t.append($x);)*
            _t
        }
    };

    ($($x:expr,)*) => {toks!($($x),*)}
}

/// Helper macro to reduce boilerplate needed with pushed token expressions.
///
/// All arguments being pushed are cloned, which should be cheap for reference types.
///
/// ## Examples
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::push;
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// push!(toks, "foo ", id);
/// push!(toks, "bar ", id);
///
/// let mut out = Vec::new();
/// out.push("foo hello");
/// out.push("bar hello");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
///
/// Pushing as a block:
///
/// ```rust
/// # #[macro_use] extern crate genco;
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// push!(toks, |t| {
///   push!(t, "foo ", id);
///   push!(t, "bar ", id);
/// });
///
/// let mut out = Vec::new();
/// out.push("foo hello");
/// out.push("bar hello");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! push {
    ($dest:expr, |$t:ident| $code:block) => {
        $dest.push({
            let mut $t = $crate::Tokens::new();
            $code
            $t
        })
    };

    ($dest:expr, $($x:expr),*) => {
        $dest.push({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        })
    };

    ($dest:expr, $($x:expr,)*) => {push!($dest, $($x),*)};
}

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// All arguments being pushed are cloned, which should be cheap for reference types.
///
/// ## Examples
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::nested;
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// nested!(toks, "foo ", id);
/// nested!(toks, "bar ", id);
///
/// let mut out = Vec::new();
/// out.push("    foo hello");
/// out.push("    bar hello");
/// out.push("");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
///
/// Pushing as a block:
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::{nested, push};
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// nested!(toks, |t| {
///   push!(t, "foo ", id);
///   push!(t, "bar ", id);
/// });
///
/// let mut out = Vec::new();
/// out.push("    foo hello");
/// out.push("    bar hello");
/// out.push("");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! nested {
    ($dest:expr, |$t:ident| $code:block) => {
        $dest.nested({
            let mut $t = $crate::Tokens::new();
            $code
            $t
        })
    };

    ($dest:expr, $($x:expr),*) => {
        $dest.nested({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        })
    };

    ($dest:expr, $($x:expr,)*) => {nested!($dest, $($x),*)};
}

macro_rules! impl_lang_item {
    ($ty:ident, $lang:ty) => {
        impl<'el> crate::FormatTokens<'el, $lang> for $ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<'el, $lang>) {
                tokens.elements.push(crate::Element::LangBox(self.into()));
            }
        }

        impl<'el> crate::FormatTokens<'el, $lang> for &'el $ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<'el, $lang>) {
                tokens.elements.push(crate::Element::LangBox(self.into()));
            }
        }

        impl<'el> From<$ty> for crate::LangBox<'el, $lang> {
            fn from(value: $ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::Rc(Rc::new(value) as Rc<dyn LangItem<$lang>>)
            }
        }

        impl<'el> From<&'el $ty> for crate::LangBox<'el, $lang> {
            fn from(value: &'el $ty) -> Self {
                crate::LangBox::Ref(value)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{JavaScript, Quoted, Tokens};

    #[test]
    fn test_quoted() {
        let n1: Tokens<JavaScript> = toks!("var v = ", "bar".quoted(), ";");
        assert_eq!("var v = \"bar\";", n1.to_string().unwrap().as_str());
    }

    #[test]
    fn test_macros() {
        let mut t = Tokens::<JavaScript>::new();

        push!(t, |t| {
            push!(t, "function bar(a, b) {");
            nested!(t, |t| {
                push!(t, "var v = a + b;");
                push!(t, "return v;");
            });
            push!(t, "}");
        });
        push!(t, "var foo = bar();");

        let mut out = Vec::new();
        out.push("function bar(a, b) {");
        out.push("  var v = a + b;");
        out.push("  return v;");
        out.push("}");
        out.push("var foo = bar();");

        assert_eq!(out.join("\n").as_str(), t.to_string().unwrap().as_str());
    }
}
