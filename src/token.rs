use std::{iter::Peekable, str::Chars};

use crate::{ParseError, Span};

pub trait Named {
    const NAME: &'static str;
}

pub trait Token<Source = char>: Sized + Named {
    fn lex(lexer: &mut impl Lexer<Output = Source>) -> Result<Self, ParseError>;
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

    fn expect(&mut self, expected: Self::Output) -> Result<(), ParseError>;

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
    chars: Peekable<Chars<'a>>,
}

impl<'a> CharsLexer<'a> {
    #[inline]
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            line: 0,
            column: 0,
            offset: 0,
            chars: chars.peekable(),
        }
    }
}

impl<'a> Lexer for CharsLexer<'a> {
    type Output = char;

    #[inline]
    fn span(&mut self, length: usize) -> Span {
        Span {
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
    fn expect(&mut self, expected: Self::Output) -> Result<(), ParseError> {
        if let Some(next_char) = self.next() {
            if next_char == expected {
                Ok(())
            } else {
                Err(ParseError::Expected {
                    span: self.span(0),
                    found: String::from(next_char),
                    expected: String::from(expected),
                })
            }
        } else {
            Err(ParseError::eof(self.span(0), expected))
        }
    }

    #[inline]
    fn fork(&mut self) -> Self {
        self.clone()
    }
}
