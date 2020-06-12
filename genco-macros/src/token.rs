/// Token allowing you to peek for anything.
///
/// Can be used to peek for end of stream with `peek`, `peek2`, and `peek3`.
pub(crate) struct Eof {}

#[allow(non_snake_case)]
pub(crate) fn Eof<T>(_: T) -> Eof {
    Eof {}
}

impl syn::token::CustomToken for Eof {
    fn peek(cursor: syn::buffer::Cursor<'_>) -> bool {
        cursor.eof()
    }

    fn display() -> &'static str {
        "<eof>"
    }
}
