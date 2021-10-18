use crate::{Parse, ParseError, Parser};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer(pub i64);

impl Parse<char> for Integer {
    #[inline]
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        P: Parser<char, Error> + ?Sized,
        Error: ParseError<char>,
    {
        let mut negative = false;

        if parser.peek()? == Some(&'-') {
            parser.consume()?;

            negative = true;
        }

        let radix = 10;
        let mut digits = String::new();

        loop {
            let tok = parser.peek()?.cloned();

            if let Some(tok) = tok {
                if tok.is_digit(radix) {
                    parser.consume()?;

                    digits.push(tok);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if !digits.is_empty() {
            if negative {
                Ok(Integer(-digits.parse::<i64>().unwrap()))
            } else {
                Ok(Integer(digits.parse().unwrap()))
            }
        } else {
            Err(Error::message("integers must contain at least one digit"))
        }
    }
}
