#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Host,
    TagErrorStartWithLetter,
    Empty,
    NomError,
}

#[derive(Debug, PartialEq)]
pub struct IRCv3TagsError<I> {
    pub input: I,
    pub code: nom::error::ErrorKind,
    pub error: ErrorKind,
    pub reason: &'static str,
}
impl<I> IRCv3TagsError<I> {
    pub fn new(
        input: I,
        code: nom::error::ErrorKind,
        error: ErrorKind,
        reason: &'static str,
    ) -> Self {
        Self {
            input,
            code,
            error,
            reason,
        }
    }
}

impl<I> nom::error::ParseError<I> for IRCv3TagsError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        IRCv3TagsError {
            input,
            code: kind,
            error: ErrorKind::NomError,
            reason: "failed to parse IRCv3 message tags",
        }
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
