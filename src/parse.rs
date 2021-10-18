use crate::ParseError;

pub trait Parser<Token, Error>
where
    Error: ParseError<Token>,
{
    fn next(&mut self) -> Option<Token>;

    fn peek(&mut self) -> Option<&Token>;

    #[inline]
    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }

    #[inline]
    fn consume(&mut self) -> Result<(), Error> {
        if self.next().is_some() {
            Ok(())
        } else {
            Err(Error::unexpected_eof())
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), Error>;

    #[inline]
    fn parse<T: Parse<Token>>(&mut self) -> Result<T, Error> {
        T::parse(self)
    }

    #[inline]
    fn try_parse<T: Parse<Token>>(&mut self) -> Option<T> {
        self.try_parse_with(|parser| T::parse(parser)).ok()
    }

    fn try_parse_with<P>(
        &mut self,
        parser: impl FnOnce(&mut dyn DynParser<Token, Error>) -> Result<P, Error>,
    ) -> Result<P, Error>;
}

pub trait DynParser<Token, Error>
where
    Error: ParseError<Token>,
{
    fn next(&mut self) -> Option<Token>;

    fn peek(&mut self) -> Option<&Token>;

    fn consume(&mut self) -> Result<(), Error> {
        if self.next().is_some() {
            Ok(())
        } else {
            Err(Error::unexpected_eof())
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), Error>;

    fn try_parse_with(
        &mut self,
        parser: Box<dyn FnOnce(&mut dyn DynParser<Token, Error>) -> Result<(), Error> + '_>,
    ) -> Result<(), Error>;
}

impl<Token, Error, P: Parser<Token, Error>> DynParser<Token, Error> for P
where
    Error: ParseError<Token>,
{
    fn next(&mut self) -> Option<Token> {
        self.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.peek()
    }

    fn expect(&mut self, token: Token) -> Result<(), Error> {
        self.expect(token)
    }

    fn try_parse_with(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn DynParser<Token, Error>) -> Result<(), Error> + '_>,
    ) -> Result<(), Error> {
        self.try_parse_with(|parser| {
            f(parser)?;
            Ok(())
        })?;

        Ok(())
    }
}

impl<Token, Error> Parser<Token, Error> for dyn DynParser<Token, Error> + '_
where
    Error: ParseError<Token>,
{
    fn next(&mut self) -> Option<Token> {
        self.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.peek()
    }

    fn expect(&mut self, token: Token) -> Result<(), Error> {
        self.expect(token)
    }

    fn try_parse_with<P>(
        &mut self,
        f: impl FnOnce(&mut dyn DynParser<Token, Error>) -> Result<P, Error>,
    ) -> Result<P, Error> {
        let mut out = None;

        self.try_parse_with(Box::new(|parser| {
            out = Some(f(parser)?);
            Ok(())
        }))?;

        Ok(out.unwrap())
    }
}

pub trait Parse<Token>: Sized {
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        Error: ParseError<Token>,
        P: Parser<Token, Error> + ?Sized;
}
