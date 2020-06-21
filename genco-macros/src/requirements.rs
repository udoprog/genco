use proc_macro2::TokenStream;
/// Language requirements for token stream.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Requirements {
    pub(crate) lang_supports_eval: bool,
}

impl Requirements {
    /// Merge this requirements with another.
    pub fn merge_with(&mut self, other: Self) {
        self.lang_supports_eval |= other.lang_supports_eval;
    }

    /// Generate checks for requirements.
    pub fn into_check(self, receiver: &syn::Ident) -> TokenStream {
        let lang_supports_eval = if self.lang_supports_eval {
            Some(q::quote!(#receiver.lang_supports_eval();))
        } else {
            None
        };

        q::quote! {
            #lang_supports_eval
        }
    }
}
