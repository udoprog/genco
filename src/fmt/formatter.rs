use core::mem;

use alloc::string::String;

use crate::fmt;
use crate::fmt::config::{Config, Indentation};
use crate::fmt::cursor;
use crate::lang::Lang;
use crate::tokens::Item;

/// Buffer used as indentation source.
static SPACES: &str = "                                                                                                    ";

static TABS: &str =
    "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t";

#[derive(Debug, Clone, Copy)]
enum Whitespace {
    Initial,
    None,
    Push,
    Line,
}

impl Whitespace {
    /// Convert into an indentation level.
    ///
    /// If we return `None`, no indentation nor lines should be written since we
    /// are at the initial stage of the file.
    fn into_indent(self) -> Option<usize> {
        match self {
            Self::Initial => Some(0),
            Self::Push => Some(1),
            Self::Line => Some(2),
            Self::None => None,
        }
    }
}

impl Default for Whitespace {
    fn default() -> Self {
        Self::None
    }
}

/// Token stream formatter. Keeps track of everything we need to know in order
/// to enforce genco's indentation and whitespace rules.
pub struct Formatter<'a> {
    write: &'a mut (dyn fmt::Write + 'a),
    /// Formatter configuration.
    config: &'a Config,
    /// How many lines we want to add to the output stream.
    ///
    /// This will only be realized if we push non-whitespace.
    line: Whitespace,
    /// How many spaces we want to add to the output stream.
    ///
    /// This will only be realized if we push non-whitespace, and will be reset
    /// if a new line is pushed or indentation changes.
    spaces: usize,
    /// Current indentation level.
    indent: i16,
}

impl<'a> Formatter<'a> {
    /// Construct a new formatter.
    pub(crate) fn new(write: &'a mut (dyn fmt::Write + 'a), config: &'a Config) -> Formatter<'a> {
        Formatter {
            write,
            line: Whitespace::Initial,
            spaces: 0usize,
            indent: 0i16,
            config,
        }
    }

    /// Format the given stream of tokens.
    pub(crate) fn format_items<L>(
        &mut self,
        items: &[Item<L>],
        config: &L::Config,
        format: &L::Format,
    ) -> fmt::Result<()>
    where
        L: Lang,
    {
        let mut cursor = cursor::Cursor::new(items);
        self.format_cursor(&mut cursor, config, format, false)
    }

    /// Forcibly write a line ending, at the end of a file.
    ///
    /// This will also reset any whitespace we have pending.
    pub(crate) fn write_trailing_line(&mut self) -> fmt::Result {
        self.line = Whitespace::default();
        self.spaces = 0;
        self.write.write_trailing_line(self.config)?;
        Ok(())
    }

    /// Write the given string.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_empty() {
            self.flush_whitespace()?;
            self.write.write_str(s)?;
        }

        Ok(())
    }

    fn push(&mut self) {
        self.line = match self.line {
            Whitespace::Initial => return,
            Whitespace::Line => return,
            _ => Whitespace::Push,
        };

        self.spaces = 0;
    }

    /// Push a new line.
    fn line(&mut self) {
        self.line = match self.line {
            Whitespace::Initial => return,
            _ => Whitespace::Line,
        };

        self.spaces = 0;
    }

    /// Push a space.
    fn space(&mut self) {
        self.spaces += 1;
    }

    /// Increase indentation level.
    fn indentation(&mut self, n: i16) {
        self.push();
        self.indent += n;
    }

    /// Internal function for formatting.
    fn format_cursor<L>(
        &mut self,
        cursor: &mut cursor::Cursor<'_, L>,
        config: &L::Config,
        format: &L::Format,
        end_on_close_quote: bool,
    ) -> fmt::Result
    where
        L: Lang,
    {
        use crate::lang::LangItem as _;

        let mut buf = String::new();
        let mut stack = smallvec::SmallVec::<[Frame; 4]>::new();

        stack.push(Frame::default());

        while let (Some(item), Some(head)) = (cursor.next(), stack.last_mut()) {
            let Frame {
                in_quote,
                has_eval,
                end_on_eval,
            } = head;

            match item {
                Item::Register(..) => (),
                Item::Indentation(0) => (),
                Item::Literal(literal) => {
                    if *in_quote {
                        L::write_quoted(self, literal)?;
                    } else {
                        self.write_str(literal)?;
                    }
                }
                Item::OpenQuote(e) if !*in_quote => {
                    *has_eval = *e;
                    *in_quote = true;
                    L::open_quote(self, config, format, *has_eval)?;
                }
                // Warning: slow path which will buffer a string internally.
                // This is used for expressions like: `$[str](Hello $(quoted(world)))`.
                //
                // Evaluating quotes are not supported.
                Item::OpenQuote(false) if *in_quote => {
                    self.quoted_quote(cursor, &mut buf, config, format)?;
                    L::write_quoted(self, &buf)?;
                    buf.clear();
                }
                Item::CloseQuote if end_on_close_quote => {
                    return Ok(());
                }
                Item::CloseQuote if *in_quote => {
                    *in_quote = false;
                    L::close_quote(self, config, format, mem::take(has_eval))?;
                }
                Item::Lang(_, lang) => {
                    lang.format(self, config, format)?;
                }
                // whitespace below
                Item::Push => {
                    self.push();
                }
                Item::Line => {
                    self.line();
                }
                Item::Space => {
                    self.space();
                }
                Item::Indentation(n) => {
                    self.indentation(*n);
                }
                Item::OpenEval if *in_quote => {
                    if cursor.peek::<cursor::Literal>() && cursor.peek1::<cursor::CloseEval>() {
                        let literal = cursor.parse::<cursor::Literal>()?;
                        L::string_eval_literal(self, config, format, literal)?;
                        cursor.parse::<cursor::CloseEval>()?;
                    } else {
                        L::start_string_eval(self, config, format)?;

                        stack.push(Frame {
                            in_quote: false,
                            has_eval: false,
                            end_on_eval: true,
                        });
                    }
                }
                // Eval are only allowed within quotes.
                Item::CloseEval if *end_on_eval => {
                    L::end_string_eval(self, config, format)?;
                    stack.pop();
                }
                _ => {
                    // Anything else is an illegal state for formatting.
                    return Err(core::fmt::Error);
                }
            }
        }

        return Ok(());

        #[derive(Default, Clone)]
        struct Frame {
            in_quote: bool,
            has_eval: bool,
            end_on_eval: bool,
        }
    }

    /// Support for evaluating an interior quote and returning it as a string.
    fn quoted_quote<L>(
        &mut self,
        cursor: &mut cursor::Cursor<'_, L>,
        buf: &mut String,
        config: &L::Config,
        format: &L::Format,
    ) -> fmt::Result<()>
    where
        L: Lang,
    {
        use crate::fmt::FmtWriter;

        let mut w = FmtWriter::new(buf);
        let out = &mut Formatter::new(&mut w, self.config);
        L::open_quote(out, config, format, false)?;
        out.format_cursor(cursor, config, format, true)?;
        L::close_quote(out, config, format, false)?;
        Ok(())
    }

    // Realize any pending whitespace just prior to writing a non-whitespace
    // item.
    fn flush_whitespace(&mut self) -> fmt::Result {
        let mut spaces = mem::take(&mut self.spaces);

        if let Some(lines) = mem::take(&mut self.line).into_indent() {
            for _ in 0..lines {
                self.write.write_line(self.config)?;
            }

            let level = i16::max(self.indent, 0) as usize;

            match self.config.indentation {
                Indentation::Space(n) => {
                    spaces += level * n;
                }
                Indentation::Tab => {
                    let mut tabs = level;

                    while tabs > 0 {
                        let len = usize::min(tabs, TABS.len());
                        self.write.write_str(&TABS[0..len])?;
                        tabs -= len;
                    }
                }
            }
        }

        while spaces > 0 {
            let len = usize::min(spaces, SPACES.len());
            self.write.write_str(&SPACES[0..len])?;
            spaces -= len;
        }

        Ok(())
    }
}

impl core::fmt::Write for Formatter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_empty() {
            Formatter::write_str(self, s)?;
        }

        Ok(())
    }
}

impl core::fmt::Debug for Formatter<'_> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.debug_struct("Formatter")
            .field("line", &self.line)
            .field("spaces", &self.spaces)
            .field("indent", &self.indent)
            .field("config", self.config)
            .finish()
    }
}
