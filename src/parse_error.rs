use std::fmt::Debug;

pub trait ParseError<Token> {
    fn expected(found: Option<Token>, expected: &[Token]) -> Self;

    fn unexpected_eof() -> Self;

    fn message(message: &str) -> Self;
}

impl<Token> ParseError<Token> for String
where
    Token: Debug,
{
    fn expected(found: Option<Token>, expected: &[Token]) -> Self {
        format!("found '{:?}', expected '{:?}'", found, expected)
    }

    fn unexpected_eof() -> Self {
        String::from("unexpected eof")
    }

    fn message(message: &str) -> Self {
        String::from(message)
    }
}
