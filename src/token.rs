use std::{iter::Peekable, str::Chars};

use crate::{string_allocator::static_str, Error, SourcePath, Span};

pub trait Token<Source = char>: Lex<Source> {
    type Kind: TokenKind;

    fn kind(&self) -> Self::Kind;
}

pub trait TokenKind: Copy + PartialEq + Eq + 'static {
    fn name(&self) -> &str;
}

pub trait Lex<Source = char>: Sized {
    fn lex(lexer: &mut impl Lexer<Output = Source>) -> Result<Self, Error>;
}

pub trait Lexer {
    type Output;

    /// Returns a [`Span`] at the cursor of the lexer.
    fn span(&mut self, length: usize) -> Span;

    fn next(&mut self) -> Option<Self::Output>;

    fn peek(&mut self) -> Option<&Self::Output>;

    #[inline]
    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }

    fn expect(&mut self, expected: Self::Output) -> Result<(), Error>;

    #[inline]
    fn consume(&mut self) {
        self.next();
    }

    fn fork(&mut self) -> Self;
}

#[derive(Clone)]
pub struct CharsLexer<'a> {
    line: usize,
    column: usize,
    offset: usize,
    path: SourcePath,
    source: &'a str,
    chars: Peekable<Chars<'a>>,
}

impl<'a> CharsLexer<'a> {
    #[inline]
    pub fn new(source: &'a str, path: SourcePath) -> Self {
        Self {
            line: 0,
            column: 0,
            offset: 0,
            path,
            source,
            chars: source.chars().peekable(),
        }
    }
}

impl<'a> Lexer for CharsLexer<'a> {
    type Output = char;

    #[inline]
    fn span(&mut self, length: usize) -> Span {
        Span {
            path: self.path,
            source: static_str(self.source),
            line: self.line,
            column: self.column,
            offset: self.offset,
            length,
        }
    }

    #[inline]
    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.chars.next() {
            self.offset += 1;

            if c == '\n' {
                self.column = 0;
                self.line += 1;
            } else {
                self.column += 1;
            }

            Some(c)
        } else {
            None
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<&char> {
        if let Some(c) = self.chars.peek() {
            Some(c)
        } else {
            None
        }
    }

    #[inline]
    fn expect(&mut self, expected: Self::Output) -> Result<(), Error> {
        if let Some(next_char) = self.next() {
            if next_char == expected {
                Ok(())
            } else {
                Err(Error::expected(self.span(1), expected, next_char))
            }
        } else {
            Err(Error::expected(self.span(0), expected, "eof"))
        }
    }

    #[inline]
    fn fork(&mut self) -> Self {
        self.clone()
    }
}
