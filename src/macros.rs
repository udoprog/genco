//! Macros in GenCo

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// ## Examples
///
/// ```rust,ignore
/// let n1: genco::Tokens<()> = toks!("var v = ", "bar".quoted(), ";");
/// ```
#[macro_export]
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
#[macro_export]
macro_rules! push {
    ($dest:ident, $($x:expr),*) => {
        $dest.push({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        })
    };

    ($dest:ident, $($x:expr,)*) => {push!($dest, $($x),*)};
}

/// Helper macro to reduce boilerplate needed with nested token expressions.
#[macro_export]
macro_rules! nested {
    ($dest:ident, $($x:expr)*) => {
        $dest.nested({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        })
    };

    ($dest:ident, $($x:expr,)*) => {nested!($dest, $($x),*)};
}

macro_rules! into_tokens_impl_from {
    ($type:ty, $custom:ty) => {
        impl<'el> From<$type> for Tokens<'el, $custom> {
            fn from(value: $type) -> Tokens<'el, $custom> {
                value.into_tokens()
            }
        }
    };
}

macro_rules! into_tokens_impl_from_generic {
    ($type:ty) => {
        impl<'el, C> From<$type> for Tokens<'el, C> {
            fn from(value: $type) -> Tokens<'el, C> {
                value.into_tokens()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use quoted::Quoted;
    use tokens::Tokens;
    use js::JavaScript;

    #[test]
    fn test_quoted() {
        let n1: Tokens<JavaScript> = toks!("var v = ", "bar".quoted(), ";");
        assert_eq!("var v = \"bar\";", n1.to_string().unwrap().as_str());
    }
}
