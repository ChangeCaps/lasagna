use std::fmt::Display;

use crate::{Lexer, Named, Parse, ParseError, Parser, Span, Token};

pub struct LexerParser<L>(L);

impl<L> LexerParser<L> {
    #[inline]
    pub fn new(lexer: L) -> Self {
        Self(lexer)
    }
}

impl<L> Parser for LexerParser<L>
where
    L: Lexer,
{
    type Source = L::Output;

    #[inline]
    fn span(&mut self, length: usize) -> Span {
        self.0.span(length)
    }

    #[inline]
    fn next<T: Token<Self::Source>>(&mut self) -> Result<T, ParseError> {
        T::lex(&mut self.0)
    }

    #[inline]
    fn is_empty(&mut self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn fork(&mut self) -> Self {
        Self(self.0.fork())
    }
}

pub struct ParserLexer<T, P> {
    parser: P,
    peek: Option<T>,
}

impl<T, P> ParserLexer<T, P> {
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser, peek: None }
    }
}

impl<T, P> Lexer for ParserLexer<T, P>
where
    P: Parser,
    T: Parse<Source = P::Source> + PartialEq<T> + Named + Display + Clone,
{
    type Output = T;

    #[inline]
    fn span(&mut self, length: usize) -> Span {
        self.parser.span(length)
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Output> {
        if let Some(t) = self.peek.take() {
            return Some(t);
        }

        if self.parser.is_empty() {
            None
        } else {
            if let Ok(t) = T::parse(&mut self.parser) {
                Some(t)
            } else {
                None
            }
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<&Self::Output> {
        if let Some(ref peek) = self.peek {
            Some(&peek)
        } else {
            if let Some(t) = self.next() {
                self.peek = Some(t);

                self.peek.as_ref()
            } else {
                None
            }
        }
    }

    #[inline]
    fn expect(&mut self, expected: Self::Output) -> Result<(), ParseError> {
        let span = self.span(0);
        if let Some(t) = self.next() {
            if t == expected {
                Ok(())
            } else {
                Err(ParseError::Expected {
                    span: span | self.span(0),
                    found: t.to_string(),
                    expected: expected.to_string(),
                })
            }
        } else {
            Err(ParseError::eof(self.span(0), expected.to_string()))
        }
    }

    #[inline]
    fn fork(&mut self) -> Self {
        Self {
            parser: self.parser.fork(),
            peek: self.peek.clone(),
        }
    }
}
