use std::{
    fmt::{Debug, Display},
    panic::Location,
};

use crate::Span;

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    span: Option<Span>,
    hints: Vec<ErrorHint>,
    location: &'static Location<'static>,
}

impl Error {
    #[track_caller]
    pub fn new(msg: impl Display) -> Self {
        Self {
            message: msg.to_string(),
            source: None,
            span: None,
            hints: Vec::new(),
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn spanned(span: Span, msg: impl Display) -> Self {
        Self {
            message: msg.to_string(),
            source: None,
            span: Some(span),
            hints: Vec::new(),
            location: Location::caller(),
        }
    }

    pub fn msg(&self) -> &str {
        &self.message
    }

    pub fn source(&self) -> Option<&Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.source.as_ref()
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }

    pub fn hints(&self) -> &[ErrorHint] {
        &self.hints
    }

    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }

    #[track_caller]
    pub fn expected(span: Span, expected: impl Display, found: impl Display) -> Self {
        Self::new(format!("expected '{}'", expected))
            .with_hint(ErrorHint::spanned(span, format!("found '{}'", found)))
    }

    #[track_caller]
    pub fn expected_one(span: Span, expected: &[impl Debug], found: impl Display) -> Self {
        Self::new(format!("expected '{:?}'", expected))
            .with_hint(ErrorHint::spanned(span, format!("found '{}'", found)))
    }

    pub fn with_hint(mut self, hint: ErrorHint) -> Self {
        self.add_hint(hint);
        self
    }

    pub fn add_hint(&mut self, hint: ErrorHint) {
        self.hints.push(hint);
    }
}

#[derive(Debug)]
pub struct ErrorHint {
    msg: String,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    spans: Vec<Span>,
}

impl ErrorHint {
    pub fn new(msg: impl Display) -> Self {
        Self {
            msg: msg.to_string(),
            source: None,
            spans: Vec::new(),
        }
    }

    pub fn spanned(span: Span, msg: impl Display) -> Self {
        Self {
            msg: msg.to_string(),
            source: None,
            spans: vec![span],
        }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn source(&self) -> Option<&Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.source.as_ref()
    }

    pub fn span(&self) -> Option<Span> {
        self.spans.first().cloned()
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }
}

impl std::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.msg())?;

        for hint in self.hints() {
            writeln!(f, "{}", hint.msg())?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}
