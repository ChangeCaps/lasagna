use crate::{Parse, ParseError, Parser};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Punctuated<T, P> {
    values: Vec<T>,
    punctuation: Vec<P>,
}

impl<Token, T, P> Parse<Token> for Punctuated<T, P>
where
    T: Parse<Token>,
    P: Parse<Token>,
{
    fn parse<Error, Par>(parser: &mut Par) -> Result<Self, Error>
    where
        Par: Parser<Token, Error> + ?Sized,
        Error: ParseError<Token>,
    {
        let mut values = Vec::new();
        let mut punctuation = Vec::new();

        while !parser.is_empty() {
            values.push(parser.parse()?);

            if parser.is_empty() {
                break;
            }

            punctuation.push(parser.parse()?);
        }

        Ok(Self {
            values,
            punctuation,
        })
    }
}

impl<T, P> IntoIterator for Punctuated<T, P> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}
