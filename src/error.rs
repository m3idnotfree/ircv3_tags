use nom::{Err::Error, IResult};

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    HostErrorStartWithLetter,
    HostErrorEndsWithLetterOrDigit,
    HostErrorNoConsecutiveHyphens,
    HostErrorInvalidLabel,

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

#[derive(Debug, PartialEq)]
pub struct HostError<I> {
    pub input: I,
    pub code: nom::error::ErrorKind,
    pub error: ErrorKind,
    pub reason: &'static str,
}

impl<I> nom::error::ParseError<I> for HostError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        HostError {
            input,
            code: kind,
            error: ErrorKind::NomError,
            reason: "characters only letters, digits, and hyphen",
        }
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

pub(crate) fn invalid_label_hyphens(input: &str) -> IResult<(), (), HostError<&str>> {
    if input.ends_with(|e| crate::host::HYPHEN.contains(e)) {
        return Err(invalid_ends_with(input));
    }

    if input.contains("--") {
        return Err(invalid_consecutive_hiphens(input));
    }
    Ok(((), ()))
}

pub(crate) fn invalid_empty_label<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Error(HostError {
        input,
        code: nom::error::ErrorKind::Alpha,
        error: ErrorKind::Empty,
        reason: "label must start with the ascii alphabet",
    })
}

pub(crate) fn invalid_start_with_letter<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Error(HostError {
        input,
        code: nom::error::ErrorKind::Alpha,
        error: ErrorKind::HostErrorStartWithLetter,
        reason: "label must start with the ascii alphabet",
    })
}

pub(crate) fn invalid_ends_with<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Error(HostError {
        input,
        code: nom::error::ErrorKind::Char,
        error: ErrorKind::HostErrorEndsWithLetterOrDigit,
        reason: "end with an ascii alphabet or ascii digit",
    })
}

pub(crate) fn invalid_consecutive_hiphens<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Error(HostError {
        input,
        code: nom::error::ErrorKind::Char,
        error: ErrorKind::HostErrorNoConsecutiveHyphens,
        reason: "cannot contain consecutive hyphens",
    })
}

pub(crate) fn check_starts_ascii_alph(input: &str) -> bool {
    input.starts_with(|c: char| c.is_ascii_alphabetic())
}
