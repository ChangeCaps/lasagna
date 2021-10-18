use crate::{ParseError, Parser};
use std::{iter::Peekable, str::Chars};

#[derive(Clone, Debug)]
pub struct CharsParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> CharsParser<'a> {
    #[inline]
    pub fn new(chars: impl Into<Chars<'a>>) -> Self {
        Self {
            chars: chars.into().peekable(),
        }
    }
}

impl<'a, Error> Parser<char, Error> for CharsParser<'a>
where
    Error: ParseError<char>,
{
    fn next(&mut self) -> Result<Option<char>, Error> {
        Ok(self.chars.next())
    }

    fn peek(&mut self) -> Result<Option<&char>, Error> {
        Ok(self.chars.peek())
    }

    fn expect(&mut self, token: char) -> Result<(), Error> {
        let found = Parser::<char, Error>::next(self)?;

        if found == Some(token) {
            Ok(())
        } else {
            Err(Error::expected(found, token.into()))
        }
    }

    fn try_parse_with<P>(
        &mut self,
        parser: impl FnOnce(&mut dyn crate::DynParser<char, Error>) -> Result<P, Error>,
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

impl<'a, T: Into<Chars<'a>>> From<T> for CharsParser<'a> {
    fn from(chars: T) -> Self {
        Self::new(chars)
    }
}
