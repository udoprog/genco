use crate::{Dart, FormatTokens, ItemStr, Tokens};

/// Format a doc comment where each line is preceeded by `///`.
pub struct DocComment(pub Vec<ItemStr>);

impl FormatTokens<Dart> for DocComment {
    fn format_tokens(self, tokens: &mut Tokens<Dart>) {
        for line in self.0 {
            tokens.append("/// ");
            tokens.append(line);
            tokens.push_line();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as genco;
    use genco::quote;

    #[test]
    fn test_doc_comment() {
        let toks = quote! {
            #(DocComment(vec![ItemStr::from("Foo")])),
            #(DocComment(vec![])),
            #(DocComment(vec![])),
            #(DocComment(vec![])),
            #(DocComment(vec![ItemStr::from("Bar")])),
        };

        let expected = vec!["/// Foo", "/// Bar", ""];

        assert_eq!(expected, toks.to_file_vec().unwrap());
    }
}
