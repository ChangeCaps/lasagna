use std::ops::{Deref, DerefMut};

use crate::{DynParser, Parse, ParseError, Parser};

pub(crate) enum MutOrOwned<'a, T: ?Sized> {
    Owned(Box<T>),
    Mut(&'a mut T),
}

impl<'a, T: ?Sized> Deref for MutOrOwned<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(v) => v,
            Self::Mut(v) => &**v,
        }
    }
}

impl<'a, T: ?Sized> DerefMut for MutOrOwned<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(v) => v,
            Self::Mut(v) => &mut **v,
        }
    }
}

pub struct TokenParser<'a, Out, P: ?Sized> {
    pub(crate) parser: MutOrOwned<'a, P>,
    pub(crate) peek: Option<Out>,
}

impl<'a, Out, P: ?Sized> TokenParser<'a, Out, P> {
    #[inline]
    pub fn new(parser: P) -> Self
    where
        P: Sized,
    {
        Self {
            parser: MutOrOwned::Owned(Box::new(parser)),
            peek: None,
        }
    }
}

impl<'a, Out, P: ?Sized> From<&'a mut P> for TokenParser<'a, Out, P> {
    #[inline]
    fn from(parser: &'a mut P) -> Self {
        Self {
            parser: MutOrOwned::Mut(parser),
            peek: None,
        }
    }
}

impl<'a, In, Out, Error> Parser<Out, Error> for TokenParser<'a, Out, dyn DynParser<In, Error> + 'a>
where
    Out: Parse<In> + PartialEq<Out> + Clone,
    Error: ParseError<In> + ParseError<Out>,
{
    fn next(&mut self) -> Result<Option<Out>, Error> {
        if self.parser.is_empty()? {
            Ok(None)
        } else {
            self.peek = None;

            Ok(Some(self.parser.parse()?))
        }
    }

    fn peek(&mut self) -> Result<Option<&Out>, Error> {
        if let Some(ref peek) = self.peek {
            Ok(Some(peek))
        } else {
            self.peek = Parser::next(self)?;

            Ok(self.peek.as_ref())
        }
    }

    fn expect(&mut self, token: Out) -> Result<(), Error> {
        let found = Parser::<Out, Error>::next(self)?;

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

    fn try_parse_with<T>(
        &mut self,
        parser: impl FnOnce(&mut dyn DynParser<Out, Error>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        Parser::try_parse_with(&mut *self.parser, |par| {
            let mut par = TokenParser::from(par);

            parser(&mut par)
        })
    }
}
