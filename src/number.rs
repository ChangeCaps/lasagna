use crate::{Lexer, Named, ParseError, Spanned, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerFormat {
    Binary,
    Decimal,
    Hex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer {
    pub format: IntegerFormat,
    pub value: i64,
}

impl Named for Integer {
    const NAME: &'static str = "integer";
}

impl Token for Integer {
    #[inline]
    fn lex(lexer: &mut impl Lexer<Output = char>) -> Result<Spanned<Self>, ParseError> {
        let span = lexer.span(0);
        let mut radix_set = false;
        let mut format = IntegerFormat::Decimal;
        let mut radix = 10;

        let mut number = String::new();

        loop {
            let c = lexer.next();

            if let Some(c) = *c {
                if number == "0" && c == 'b' && !radix_set {
                    radix_set = true;

                    number.clear();

                    format = IntegerFormat::Binary;
                    radix = 2;
                } else if number == "0" && c == 'x' && !radix_set {
                    radix_set = true;

                    number.clear();

                    format = IntegerFormat::Hex;
                    radix = 16;
                } else if c.is_digit(radix) {
                    number.push(c);
                }
            } else {
                break;
            }
        }

        if number.len() > 0 {
            Ok(Spanned::new(
                Self {
                    format,
                    value: i64::from_str_radix(&number, radix).unwrap(),
                },
                span | lexer.span(0),
            ))
        } else {
            Err(ParseError::msg(
                span | lexer.span(0),
                "integers must contain at least one digit",
            ))
        }
    }
}
