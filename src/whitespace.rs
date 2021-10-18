use std::ops::{Deref, DerefMut};

use crate::{DynParser, Parse, ParseError, Parser, Punctuated};

crate::token!(Whitespace);

impl Parse<char> for Whitespace {
    #[inline]
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        P: Parser<char, Error> + ?Sized,
        Error: ParseError<char>,
    {
        while let Some(tok) = parser.peek()? {
            if tok.is_whitespace() {
                parser.consume()?;
            } else {
                break;
            }
        }

        Ok(Self)
    }
}

pub type SeparateWhitespace<T> = Punctuated<T, Whitespace>;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WhitespacePadded<T> {
    pub left_padding: Whitespace,
    pub value: T,
    pub right_padding: Whitespace,
}

impl<Token, T> Parse<Token> for WhitespacePadded<T>
where
    Whitespace: Parse<Token>,
    T: Parse<Token>,
{
    #[inline]
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        Error: ParseError<Token>,
        P: Parser<Token, Error> + ?Sized,
    {
        Ok(Self {
            left_padding: parser.parse()?,
            value: parser.parse()?,
            right_padding: parser.parse()?,
        })
    }
}

impl<T> Deref for WhitespacePadded<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for WhitespacePadded<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Clone, Debug)]
pub struct WhitespaceParser<Token, P> {
    parser: P,
    peek: Option<Token>,
}

impl<Token, P> WhitespaceParser<Token, P> {
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser, peek: None }
    }
}

impl<In, Out, Error> Parser<Out, Error>
    for WhitespaceParser<Out, Box<dyn DynParser<In, Error> + '_>>
where
    Error: ParseError<In> + ParseError<Out>,
    Whitespace: Parse<In>,
    Out: Parse<In> + PartialEq<Out>,
{
    fn next(&mut self) -> Result<Option<Out>, Error> {
        if self.parser.is_empty()? {
            return Ok(None);
        }

        self.peek = None;

        self.parser.parse::<Whitespace>()?;

        if self.parser.is_empty()? {
            return Ok(None);
        }

        let out = self.parser.parse()?;

        self.parser.parse::<Whitespace>()?;

        Ok(Some(out))
    }

    fn peek(&mut self) -> Result<Option<&Out>, Error> {
        if let Some(ref token) = self.peek {
            Ok(Some(token))
        } else {
            self.peek = Parser::next(self)?;

            Ok(self.peek.as_ref())
        }
    }

    fn expect(&mut self, token: Out) -> Result<(), Error> {
        let found = Parser::next(self)?;

        if let Some(ref tok) = found {
            if *tok == token {
                Ok(())
            } else {
                Err(Error::expected(found, token.into()))
            }
        } else {
            Err(ParseError::<Out>::unexpected_eof())
        }
    }

    fn try_parse_with<P>(
        &mut self,
        parser: impl FnOnce(&mut dyn DynParser<Out, Error>) -> Result<P, Error>,
    ) -> Result<P, Error> {
        Parser::try_parse_with(self.parser.as_mut(), |par| {
            let mut par = WhitespaceParser::new(par);

            parser(&mut par)
        })
    }
}

impl<In, Out, Error> Parser<Out, Error>
    for WhitespaceParser<Out, &mut (dyn DynParser<In, Error> + '_)>
where
    Error: ParseError<In> + ParseError<Out>,
    Whitespace: Parse<In>,
    Out: Parse<In> + PartialEq<Out>,
{
    fn next(&mut self) -> Result<Option<Out>, Error> {
        self.peek = None;

        if self.parser.is_empty()? {
            return Ok(None);
        }

        Ok(Some(self.parser.parse()?))
    }

    fn peek(&mut self) -> Result<Option<&Out>, Error> {
        if let Some(ref token) = self.peek {
            Ok(Some(token))
        } else {
            self.peek = Parser::next(self)?;

            Ok(self.peek.as_ref())
        }
    }

    fn expect(&mut self, token: Out) -> Result<(), Error> {
        let found = Parser::next(self)?;

        if let Some(ref tok) = found {
            if *tok == token {
                Ok(())
            } else {
                Err(Error::expected(found, token.into()))
            }
        } else {
            Err(ParseError::<Out>::unexpected_eof())
        }
    }

    fn try_parse_with<P>(
        &mut self,
        parser: impl FnOnce(&mut dyn DynParser<Out, Error>) -> Result<P, Error>,
    ) -> Result<P, Error> {
        Parser::try_parse_with(self.parser, |par| {
            let mut par = WhitespaceParser::new(par);

            parser(&mut par)
        })
    }
}
