use std::fmt::Arguments;

use proc_macro2::Span;

use crate::cursor::Cursor;

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
}

#[derive(Default)]
pub(crate) struct Buf {
    buf: Option<String>,
}

impl Buf {
    /// Format the given arguments and return the associated string.
    fn format(&mut self, args: Arguments<'_>) -> &str {
        use std::fmt::Write;
        let buf = self.buf.get_or_insert_with(String::default);
        buf.clear();
        buf.write_fmt(args).unwrap();
        buf.as_str()
    }

    pub(crate) fn pair(&mut self, span: Span) -> syn::Result<(LineColumn, LineColumn)> {
        let start = span.start();
        let end = span.end();

        if (start.line == 0 && start.column == 0) || (end.line == 0 && end.column == 0) {
            // Try compat.
            let (start, end) = self.find_line_column(span)?;

            Ok((
                LineColumn {
                    line: 1,
                    column: start,
                },
                LineColumn {
                    line: 1,
                    column: end,
                },
            ))
        } else {
            Ok((LineColumn::new(start), LineColumn::new(end)))
        }
    }

    /// The start of the given span.
    pub(crate) fn start(&mut self, span: Span) -> syn::Result<LineColumn> {
        let start = span.start();

        // Try to use compat layer.
        if start.line == 0 && start.column == 0 {
            // Try compat.
            let (column, _) = self.find_line_column(span)?;
            Ok(LineColumn { line: 1, column })
        } else {
            Ok(LineColumn::new(start))
        }
    }

    /// The start of the given span.
    pub(crate) fn end(&mut self, span: Span) -> syn::Result<LineColumn> {
        let end = span.end();

        // Try to use compat layer.
        if end.line == 0 && end.column == 0 {
            // Try compat.
            let (_, column) = self.find_line_column(span)?;
            Ok(LineColumn { line: 1, column })
        } else {
            Ok(LineColumn::new(end))
        }
    }

    /// Join two spans.
    pub(crate) fn join(&mut self, a: Span, b: Span) -> syn::Result<Cursor> {
        Ok(Cursor::new(self.start(a)?, self.end(b)?))
    }

    /// Construct a cursor from a span.
    pub(crate) fn from_span(&mut self, span: Span) -> syn::Result<Cursor> {
        let (start, end) = self.pair(span)?;
        Ok(Cursor::new(start, end))
    }

    /// Try to decode line and column information using the debug implementation of
    /// a `span` which leaks the byte offset of a thing.
    fn find_line_column(&mut self, span: Span) -> syn::Result<(usize, usize)> {
        match self.find_line_column_inner(span) {
            Some((start, end)) => Ok((start, end)),
            None => Err(syn::Error::new(span, ERROR)),
        }
    }

    fn find_line_column_inner(&mut self, span: Span) -> Option<(usize, usize)> {
        let text = self.format(format_args!("{:?}", span));
        let start = text.find('(')?;
        let (start, end) = text
            .get(start.checked_add(1)?..text.len().checked_sub(1)?)?
            .split_once("..")?;
        Some((str::parse(start).ok()?, str::parse(end).ok()?))
    }
}
