// Copied from https://github.com/luser/strip-ansi-escapes
// Copyright (c) Ted Mielczarek
// MIT License
// Updated to ignore \t and \r when escaping.

use std::io::{self, Cursor, IntoInnerError, LineWriter, Write};
use vte::{Parser, Perform};

/// `Writer` wraps an underlying type that implements `Write`, stripping ANSI escape sequences
/// from bytes written to it before passing them to the underlying writer.
pub struct Writer<W>
where
    W: Write,
{
    performer: Performer<W>,
    parser: Parser,
}

/// Strip ANSI escapes from `data` and return the remaining bytes as a `Vec<u8>`.
pub fn strip<T>(data: T) -> io::Result<Vec<u8>>
where
    T: AsRef<[u8]>,
{
    let c = Cursor::new(Vec::new());
    let mut writer = Writer::new(c);
    writer.write_all(data.as_ref())?;
    Ok(writer.into_inner()?.into_inner())
}

struct Performer<W>
where
    W: Write,
{
    writer: LineWriter<W>,
    err: Option<io::Error>,
}

impl<W> Writer<W>
where
    W: Write,
{
    /// Create a new `Writer` that writes to `inner`.
    pub fn new(inner: W) -> Writer<W> {
        Writer {
            performer: Performer {
                writer: LineWriter::new(inner),
                err: None,
            },
            parser: Parser::new(),
        }
    }

    /// Unwraps this `Writer`, returning the underlying writer.
    ///
    /// The internal buffer is written out before returning the writer, which
    /// may produce an [`IntoInnerError`].
    pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> {
        self.performer.into_inner()
    }
}

impl<W> Write for Writer<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for b in buf.iter() {
            self.parser.advance(&mut self.performer, *b)
        }
        match self.performer.err.take() {
            Some(e) => Err(e),
            None => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.performer.flush()
    }
}

impl<W> Performer<W>
where
    W: Write,
{
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> {
        self.writer.into_inner()
    }
}

impl<W> Perform for Performer<W>
where
    W: Write,
{
    fn print(&mut self, c: char) {
        // Just print bytes to the inner writer.
        self.err = write!(self.writer, "{}", c).err();
    }
    fn execute(&mut self, byte: u8) {
        // We only care about executing linefeeds and tabs.
        if byte == b'\n' {
            self.err = writeln!(self.writer, "").err();
        } else if byte == b'\t' {
            self.err = write!(self.writer, "\t").err();
        } else if byte == b'\r' {
            self.err = write!(self.writer, "\r").err();
        }
    }
    // Since we're not actually implementing a terminal, we just ignore everything else.
    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]]) {}
    fn csi_dispatch(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _: char) {}
    fn esc_dispatch(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
