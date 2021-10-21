use crate::{Span, Spanned};

#[derive(Clone, Debug)]
pub enum ParseError {
    ExpectedOne {
        span: Span,
        expected: Vec<String>,
    },
    Expected {
        found: Spanned<String>,
        expected: String,
    },
    UnexpectedEof {
        span: Span,
        expected: String,
    },
    Message(Spanned<String>),
}

impl ParseError {
    #[inline]
    pub fn eof(span: Span, expected: impl Into<String>) -> Self {
        Self::UnexpectedEof {
            span,
            expected: expected.into(),
        }
    }

    #[inline]
    pub fn msg(span: Span, msg: impl Into<String>) -> Self {
        Self::Message(Spanned::new(msg.into(), span))
    }
}

impl std::fmt::Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedOne { span, expected } => {
                write!(f, "expected one of the following '")?;

                for (i, e) in expected.iter().enumerate() {
                    if i < expected.len() - 1 {
                        write!(f, "{}, ", e)?;
                    } else {
                        write!(f, "{}", e)?;
                    }
                }

                write!(f, "' at line: {} column: {}", span.line, span.column)
            }
            Self::Expected { found, expected } => {
                write!(f, "found {}, expected '{}'", found, expected)
            }
            Self::UnexpectedEof { span, expected } => {
                write!(
                    f,
                    "found <eof> at line: {} column: {}, expected '{}'",
                    span.line, span.column, expected
                )
            }
            Self::Message(message) => {
                write!(f, "{}", message)
            }
        }
    }
}
