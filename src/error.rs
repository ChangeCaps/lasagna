use crate::Span;

#[derive(Clone, Debug)]
pub enum ParseError {
    ExpectedOne {
        span: Span,
        expected: Vec<String>,
    },
    Expected {
        span: Span,
        found: String,
        expected: String,
    },
    UnexpectedEof {
        span: Span,
        expected: String,
    },
    Message {
        span: Span,
        msg: String,
    },
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
        Self::Message {
            span,
            msg: msg.into(),
        }
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
            Self::Expected {
                span,
                found,
                expected,
            } => {
                write!(
                    f,
                    "found {} as line: {} column: {}, expected '{}'",
                    found, span.line, span.column, expected
                )
            }
            Self::UnexpectedEof { span, expected } => {
                write!(
                    f,
                    "found <eof> at line: {} column: {}, expected '{}'",
                    span.line, span.column, expected
                )
            }
            Self::Message { span, msg } => {
                write!(f, "{} at line: {} column {}", msg, span.line, span.column)
            }
        }
    }
}

impl std::error::Error for ParseError {}
