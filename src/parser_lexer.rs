use std::fmt::Display;

use crate::{Lexer, Named, Parse, ParseError, Parser, Span, Spanned, Token};

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
    fn next<T: Token<Self::Source>>(&mut self) -> Result<Spanned<T>, ParseError> {
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
    peek: Option<Spanned<T>>,
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
    fn next(&mut self) -> Spanned<Option<Self::Output>> {
        if let Some(t) = self.peek.take() {
            return Spanned::new(Some(t.value), t.span);
        }

        if self.parser.is_empty() {
            Spanned::new(None, self.span(0))
        } else {
            if let Ok(t) = T::parse(&mut self.parser) {
                Spanned::new(Some(t.value), t.span)
            } else {
                Spanned::new(None, self.span(0))
            }
        }
    }

    #[inline]
    fn peek(&mut self) -> Spanned<Option<&Self::Output>> {
        if let Some(ref peek) = self.peek {
            Spanned::new(Some(&peek.value), peek.span)
        } else {
            let next = self.next();
            if let Some(t) = next.value {
                self.peek = Some(Spanned::new(t, next.span));

                Spanned::new(Some(&self.peek.as_ref().unwrap().value), next.span)
            } else {
                Spanned::new(None, next.span)
            }
        }
    }

    #[inline]
    fn expect(&mut self, expected: Self::Output) -> Result<(), ParseError> {
        let next = self.next();

        if let Some(t) = next.value {
            if t == expected {
                Ok(())
            } else {
                Err(ParseError::Expected {
                    found: Spanned::new(t.to_string(), next.span),
                    expected: expected.to_string(),
                })
            }
        } else {
            Err(ParseError::eof(next.span, expected.to_string()))
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
