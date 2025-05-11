use nom::{
    bytes::complete::take_till,
    character::{char, complete::space1},
    combinator::{opt, recognize},
    multi::separated_list1,
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

use crate::{
    host::{RFC952HostParser, StandardHostValidator},
    CharValidator, ErrorKind, IRCv3Tags, IRCv3TagsError,
};

pub struct IRCv3TagsParser<T: CharValidator, H: CharValidator> {
    tag_name_validator: T,
    host_validator: RFC952HostParser<H>,
}

impl Default for IRCv3TagsParser<StandardTagValidator, StandardHostValidator> {
    fn default() -> Self {
        Self {
            tag_name_validator: StandardTagValidator,
            host_validator: RFC952HostParser::new(StandardHostValidator),
        }
    }
}

impl<T> IRCv3TagsParser<T, StandardHostValidator>
where
    T: CharValidator,
{
    pub fn new(validator: T) -> Self {
        Self {
            tag_name_validator: validator,
            host_validator: RFC952HostParser::new(StandardHostValidator),
        }
    }

    pub fn parse<'a>(&self, input: &'a str) -> (&'a str, IRCv3Tags<'a>) {
        self.try_parse(input).unwrap()
    }

    pub fn try_parse<'a>(&self, input: &'a str) -> IResult<&'a str, IRCv3Tags<'a>> {
        self.debug_parse(input)
            .map_err(|err| err.map(|e| nom::error::Error::new(e.input, e.code)))
    }

    /// Parse with detailed error messages
    pub fn debug_parse<'a>(
        &self,
        input: &'a str,
    ) -> IResult<&'a str, IRCv3Tags<'a>, IRCv3TagsError<&'a str>> {
        if input.is_empty() || !input.starts_with('@') {
            return Err(nom::Err::Error(IRCv3TagsError::new(
                input,
                nom::error::ErrorKind::Char,
                ErrorKind::TagErrorStartWithLetter,
                "tag must start with an '@'",
            )));
        }

        let (remain, tags) = delimited(char('@'), |i| self.try_tags(i), space1)
            .parse(input)
            .map_err(|err| {
                err.map(|e| IRCv3TagsError {
                    input: e.input,
                    code: e.code,
                    error: e.error,
                    reason: e.reason,
                })
            })?;

        Ok((remain, IRCv3Tags(tags)))
    }

    #[allow(clippy::type_complexity)]
    pub fn try_tags<'a>(
        &self,
        input: &'a str,
    ) -> IResult<&'a str, Vec<(&'a str, Option<&'a str>)>, IRCv3TagsError<&'a str>> {
        separated_list1(char(';'), |i| self.tag(i)).parse(input)
    }

    pub(crate) fn tag<'a>(
        &self,
        input: &'a str,
    ) -> IResult<&'a str, (&'a str, Option<&'a str>), IRCv3TagsError<&'a str>> {
        (
            |i| self.key(i),
            opt(preceded(char('='), |c| self.escaped_value(c))),
        )
            .parse(input)
    }

    /// Parses a vendor part of the tag which follows the format `vendor/` where vendor
    /// must be a valid hostname as defined in RFC 952.
    ///
    /// A valid hostname consists of:
    /// - Only alphanumeric characters, dots '.', and hyphens '-'
    /// - Segments cannot start or end with a hyphen '-'
    /// - Must end with a forward slash '/'
    fn key<'a>(&self, input: &'a str) -> IResult<&'a str, &'a str, IRCv3TagsError<&'a str>> {
        recognize((
            opt(|c| self.client_prefix(c)),
            opt(terminated(
                |i| {
                    self.host_validator.try_host(i).map_err(|err| {
                        err.map(|e| IRCv3TagsError {
                            input: e.input,
                            code: e.code,
                            error: e.error,
                            reason: e.reason,
                        })
                    })
                },
                char('/'),
            )),
            |i| self.key_name(i),
        ))
        .parse(input)
        .map_err(|err| {
            err.map(|e| IRCv3TagsError {
                input: e.input,
                code: e.code,
                error: e.error,
                reason: e.reason,
            })
        })
    }

    fn key_name<'a>(&self, input: &'a str) -> IResult<&'a str, &'a str, IRCv3TagsError<&'a str>> {
        if input.is_empty() {
            return Err(nom::Err::Error(IRCv3TagsError::new(
                input,
                nom::error::ErrorKind::Char,
                ErrorKind::Empty,
                "tag key does not allow empty",
            )));
        }

        let first_char = input.chars().next().unwrap();
        if !self.tag_name_validator.is_valid_start_char(first_char) {
            return Err(nom::Err::Error(IRCv3TagsError {
                input,
                code: nom::error::ErrorKind::Char,
                error: ErrorKind::TagErrorStartWithLetter,
                reason: "tag key must start with an allowed character",
            }));
        }

        let (remain, key_name_str) = self.tag_name_validator.while_valid(input, first_char);
        if remain.is_empty() {
            return Err(nom::Err::Error(IRCv3TagsError {
                input,
                code: nom::error::ErrorKind::Char,
                error: ErrorKind::Empty,
                reason: "tag key must not be empty",
            }));
        }

        Ok((remain, key_name_str))
    }

    /// Parses an escaped value which is a sequence of zero or more UTF-8 characters
    /// except NUL, CR, LF, semicolon (`;`) and SPACE.
    fn escaped_value<'a>(
        &self,
        input: &'a str,
    ) -> IResult<&'a str, &'a str, IRCv3TagsError<&'a str>> {
        take_till(|c| c == '\0' || c == '\r' || c == '\n' || c == ';' || c == ' ').parse(input)
    }

    fn client_prefix<'a>(&self, input: &'a str) -> IResult<&'a str, char, IRCv3TagsError<&'a str>> {
        char('+').parse(input)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StandardTagValidator;
impl CharValidator for StandardTagValidator {
    fn is_valid_char(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '-'
    }

    fn is_valid_start_char(&self, c: char) -> bool {
        c.is_ascii_alphabetic()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CustomTagNameValidator {
    extra_chars: Vec<char>,
    extra_start_chars: Vec<char>,
}

impl CustomTagNameValidator {
    pub fn new() -> Self {
        Self {
            extra_chars: Vec::new(),
            extra_start_chars: Vec::new(),
        }
    }

    pub fn allow_chars(mut self, chars: &[char]) -> Self {
        self.extra_chars.extend_from_slice(chars);
        self
    }

    pub fn allow_start_chars(mut self, chars: &[char]) -> Self {
        self.extra_start_chars.extend_from_slice(chars);
        self
    }
}

impl CharValidator for CustomTagNameValidator {
    fn is_valid_char(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '-' || self.extra_chars.contains(&c)
    }

    fn is_valid_start_char(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || self.extra_start_chars.contains(&c)
    }
}
