use core::cell::{RefCell, RefMut};
use core::fmt::Arguments;

use proc_macro2::Span;

use crate::cursor::Cursor;

/// Error message raised.
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
    #[cfg(has_proc_macro_span)]
    pub(crate) fn start(span: Span) -> Option<Self> {
        let span = span.unwrap().start();

        Some(Self {
            line: span.line(),
            column: span.column(),
        })
    }

    #[cfg(has_proc_macro_span)]
    pub(crate) fn end(span: Span) -> Option<Self> {
        let span = span.unwrap().end();

        Some(Self {
            line: span.line(),
            column: span.column(),
        })
    }

    #[cfg(not(has_proc_macro_span))]
    pub(crate) fn start(_: Span) -> Option<Self> {
        None
    }

    #[cfg(not(has_proc_macro_span))]
    pub(crate) fn end(_: Span) -> Option<Self> {
        None
    }
}

#[derive(Default)]
pub(crate) struct Buf {
    buf: RefCell<String>,
}

impl Buf {
    /// Format the given arguments and return the associated string.
    fn format(&self, args: Arguments<'_>) -> RefMut<'_, str> {
        use std::fmt::Write;
        let mut buf = self.buf.borrow_mut();
        buf.clear();
        buf.write_fmt(args).unwrap();
        RefMut::map(buf, |buf| buf.as_mut_str())
    }

    /// Construct a cursor from a span.
    pub(crate) fn cursor(&self, span: Span) -> syn::Result<Cursor> {
        let start = LineColumn::start(span);
        let end = LineColumn::end(span);

        if let (Some(start), Some(end)) = (start, end) {
            return Ok(Cursor::new(span, start, end));
        }

        // Try compat.
        let (start, end) = self.find_line_column(span)?;

        Ok(Cursor::new(
            span,
            LineColumn {
                line: 1,
                column: start,
            },
            LineColumn {
                line: 1,
                column: end,
            },
        ))
    }

    /// The start of the given span.
    pub(crate) fn start(&mut self, span: Span) -> syn::Result<LineColumn> {
        if let Some(start) = LineColumn::start(span) {
            return Ok(start);
        }

        // Try compat.
        let (column, _) = self.find_line_column(span)?;
        Ok(LineColumn { line: 1, column })
    }

    /// The start of the given span.
    pub(crate) fn end(&mut self, span: Span) -> syn::Result<LineColumn> {
        if let Some(end) = LineColumn::end(span) {
            return Ok(end);
        }

        // Try compat.
        let (_, column) = self.find_line_column(span)?;
        Ok(LineColumn { line: 1, column })
    }

    /// Join two spans.
    pub(crate) fn join(&mut self, a: Span, b: Span) -> syn::Result<Cursor> {
        Ok(Cursor::new(
            a.join(b).unwrap_or(a),
            self.start(a)?,
            self.end(b)?,
        ))
    }

    /// Try to decode line and column information using the debug implementation of
    /// a `span` which leaks the byte offset of a thing.
    fn find_line_column(&self, span: Span) -> syn::Result<(usize, usize)> {
        match self.find_line_column_inner(span) {
            Some((start, end)) => Ok((start, end)),
            None => Err(syn::Error::new(span, ERROR)),
        }
    }

    fn find_line_column_inner(&self, span: Span) -> Option<(usize, usize)> {
        let text = self.format(format_args!("{span:?}"));
        let start = text.find('(')?;
        let (start, end) = text
            .get(start.checked_add(1)?..text.len().checked_sub(1)?)?
            .split_once("..")?;
        Some((str::parse(start).ok()?, str::parse(end).ok()?))
    }
}
