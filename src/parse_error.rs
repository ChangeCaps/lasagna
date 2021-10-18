use std::fmt::{Debug, Display};

#[derive(Clone, Debug)]
pub enum TokenOrMessage<'a, Token> {
    Token(Token),
    Message(&'a str),
}

impl<'a, Token> TokenOrMessage<'a, Token> {
    #[inline]
    pub fn from_str(msg: &'a str) -> Self {
        Self::Message(msg)
    }
}
impl<Token> Display for TokenOrMessage<'_, Token>
where
    Token: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(token) => token.fmt(f),
            Self::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl<Token> From<Token> for TokenOrMessage<'_, Token> {
    #[inline]
    fn from(token: Token) -> Self {
        Self::Token(token)
    }
}

pub trait ParseError<Token> {
    fn expected(found: Option<Token>, expected: TokenOrMessage<Token>) -> Self;

    fn expected_one(found: Option<Token>, expected: &[TokenOrMessage<Token>]) -> Self;

    fn unexpected_eof() -> Self;

    fn message(message: &str) -> Self;
}

impl<Token> ParseError<Token> for String
where
    Token: Debug,
{
    fn expected(found: Option<Token>, expected: TokenOrMessage<Token>) -> Self {
        format!("found '{:?}', expected '{:?}'", found, expected)
    }

    fn expected_one(found: Option<Token>, expected: &[TokenOrMessage<Token>]) -> Self {
        format!("found '{:?}', expected '{:?}'", found, expected)
    }

    fn unexpected_eof() -> Self {
        String::from("unexpected eof")
    }

    fn message(message: &str) -> Self {
        String::from(message)
    }
}
