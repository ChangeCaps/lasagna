use std::iter::FromIterator;

use crate::{Parse, ParseError, Parser};

#[derive(Clone, Debug)]
pub struct ParseBuffer<Token> {
    pub(crate) tokens: Vec<Token>,
}

impl<Token> ParseBuffer<Token> {
    #[inline]
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

impl<Token> Default for ParseBuffer<Token> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Token> FromIterator<Token> for ParseBuffer<Token> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Token>>(iter: T) -> Self {
        Self {
            tokens: iter.into_iter().collect(),
        }
    }
}

impl<Token, Error> Parser<Token, Error> for ParseBuffer<Token>
where
    Token: Clone + PartialEq<Token>,
    Error: ParseError<Token>,
{
    fn next(&mut self) -> Option<Token> {
        if self.tokens.is_empty() {
            None
        } else {
            Some(self.tokens.remove(0))
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        if self.tokens.is_empty() {
            None
        } else {
            self.tokens.first()
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), Error> {
        let found = Parser::<Token, Error>::next(self);

        if let Some(ref tok) = found {
            if *tok == token {
                Ok(())
            } else {
                Err(Error::expected(found, &[token]))
            }
        } else {
            Err(Error::unexpected_eof())
        }
    }

    fn try_parse_with<P>(
        &mut self,
        parser: impl FnOnce(&mut dyn crate::DynParser<Token, Error>) -> Result<P, Error>,
    ) -> Result<P, Error> {
        let mut fork = self.clone();

        match parser(&mut fork) {
            Ok(v) => {
                *self = fork;

                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
}

impl<Token, T> Parse<T> for ParseBuffer<Token>
where
    Token: Parse<T>,
{
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        P: Parser<T, Error> + ?Sized,
        Error: ParseError<T>,
    {
        let mut tokens = Vec::new();

        while parser.peek().is_some() {
            tokens.push(Token::parse(parser)?);
        }

        Ok(Self { tokens })
    }
}
