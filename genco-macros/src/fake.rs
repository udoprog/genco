use proc_macro2::Span;

/// Erro rmessage raise.d
const ERROR: &str = "Your compiler does not support spans which are required by genco and compat doesn't work, see: https://github.com/rust-lang/rust/issues/54725";

/// Internal line-column abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineColumn {
    /// The line.
    pub(crate) line: usize,
    /// The column.
    pub(crate) column: usize,
}

impl LineColumn {
    fn new(line_column: proc_macro2::LineColumn) -> Self {
        Self {
            line: line_column.line,
            column: line_column.column,
        }
    }

    pub(crate) fn pair(span: Span) -> syn::Result<(Self, Self)> {
        let start = span.start();
        let end = span.end();

        if (start.line == 0 && start.column == 0) || (end.line == 0 && end.column == 0) {
            // Try compat.
            let (start, end) = find_line_column(span)?;

            Ok((
                Self {
                    line: 1,
                    column: start,
                },
                Self {
                    line: 1,
                    column: end,
                },
            ))
        } else {
            Ok((Self::new(start), Self::new(end)))
        }
    }

    /// The start of the given span.
    pub(crate) fn start(span: Span) -> syn::Result<Self> {
        let start = span.start();

        // Try to use compat layer.
        if start.line == 0 && start.column == 0 {
            // Try compat.
            let (column, _) = find_line_column(span)?;
            Ok(Self { line: 1, column })
        } else {
            Ok(Self::new(start))
        }
    }

    /// The start of the given span.
    pub(crate) fn end(span: Span) -> syn::Result<Self> {
        let end = span.end();

        // Try to use compat layer.
        if end.line == 0 && end.column == 0 {
            // Try compat.
            let (_, column) = find_line_column(span)?;
            Ok(Self { line: 1, column })
        } else {
            Ok(Self::new(end))
        }
    }
}

/// Try to decode line and column information using the debug implementation of
/// a `span` which leaks the byte offset of a thing.
fn find_line_column(span: Span) -> syn::Result<(usize, usize)> {
    match find_line_column_inner(span) {
        Some((start, end)) => Ok((start, end)),
        None => Err(syn::Error::new(span, ERROR)),
    }
}

fn find_line_column_inner(span: Span) -> Option<(usize, usize)> {
    let text = format!("{:?}", span);
    let start = text.find('(')?;
    let (start, end) = text
        .get(start.checked_add(1)?..text.len().checked_sub(1)?)?
        .split_once("..")?;
    Some((str::parse(start).ok()?, str::parse(end).ok()?))
}
