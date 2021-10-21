use std::{iter::Peekable, str::Chars};

use crate::{ParseError, Span, Spanned};

pub trait Named {
    const NAME: &'static str;
}

pub trait Token<Source = char>: Sized + Named {
    fn lex(lexer: &mut impl Lexer<Output = Source>) -> Result<Spanned<Self>, ParseError>;
}

pub trait Lexer {
    type Output;

    /// Returns a [`Span`] at the cursor of the lexer.
    fn span(&mut self, length: usize) -> Span;

    fn next(&mut self) -> Spanned<Option<Self::Output>>;

    fn peek(&mut self) -> Spanned<Option<&Self::Output>>;

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
    fn next(&mut self) -> Spanned<Option<char>> {
        if let Some(c) = self.chars.next() {
            let span = self.span(1);

            self.offset += 1;

            if c == '\n' {
                self.column = 0;
                self.line += 1;
            } else {
                self.column += 1;
            }

            Spanned::new(Some(c), span)
        } else {
            Spanned::new(None, self.span(0))
        }
    }

    #[inline]
    fn peek(&mut self) -> Spanned<Option<&char>> {
        let mut span = self.span(1);

        if let Some(c) = self.chars.peek() {
            Spanned::new(Some(c), span)
        } else {
            span.length = 0;

            Spanned::new(None, span)
        }
    }

    #[inline]
    fn expect(&mut self, expected: Self::Output) -> Result<(), ParseError> {
        let next = self.next();

        if let Some(next_char) = next.value {
            if next_char == expected {
                Ok(())
            } else {
                Err(ParseError::Expected {
                    found: Spanned::new(String::from(next_char), next.span),
                    expected: String::from(expected),
                })
            }
        } else {
            Err(ParseError::eof(next.span, expected))
        }
    }

    #[inline]
    fn fork(&mut self) -> Self {
        self.clone()
    }
}
