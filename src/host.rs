//! From RFC 952:
//! - A "name" (hostname) is a text string up to 24 characters drawn from the alphabet (A-Z),
//!   digits (0-9), minus sign (-), and period (.).
//! - Names must start with a letter, end with a letter or digit, and have as interior
//!   characters only letters, digits, and hyphen.
//! - Segments are separated by periods.
//! - A valid hostname can have multiple segments.
//! - Cannot contain consecutive hyphens
//!
//! For more information, see the [RFC 952](https://datatracker.ietf.org/doc/html/rfc952) \[DNS:4\]
//!
//! # Example
//! ```
//! # use ircv3_tags::host::host;
//!
//! let input = "example.com";
//! let (remain, messages) = host(input).unwrap();
//! assert_eq!(messages, "example.com");
//! ```
use nom::IResult;

use crate::{CharValidator, ErrorKind};

/// RFC 952 (host) parser
pub fn host(input: &str) -> IResult<&str, &str> {
    try_host(input).map_err(|err| err.map(|e| nom::error::Error::new(e.input, e.code)))
}

/// RFC 978 (host) parser with helpful error messages
///
/// # Example
/// ```
/// # use ircv3_tags::host::{try_host, HostError};
///
/// let input = "example.com";
/// let (remain, messages) = try_host(input).unwrap();
/// assert_eq!(messages, "example.com");
///
/// assert_eq!(
///     try_host("invalid-"),
///     Err(nom::Err::Error(HostError::new(
///         "invalid-",
///         nom::error::ErrorKind::Char,
///         ircv3_tags::ErrorKind::Host,
///         "label contains an invalid chracter",
///     )))
/// );
/// ```
pub fn try_host(input: &str) -> IResult<&str, &str, HostError<&str>> {
    RFC952HostParser::default().try_host(input)
}

pub struct RFC952HostParser<V: CharValidator> {
    validator: V,
}

impl Default for RFC952HostParser<StandardHostValidator> {
    fn default() -> Self {
        Self {
            validator: StandardHostValidator,
        }
    }
}

impl<V: CharValidator> RFC952HostParser<V> {
    pub fn new(validator: V) -> Self {
        Self { validator }
    }

    pub fn try_host<'a>(&self, input: &'a str) -> IResult<&'a str, &'a str, HostError<&'a str>> {
        let (remain, label_str) = self.label(input)?;

        if self.validator.is_invalid_char(label_str) {
            return Err(nom::Err::Error(HostError::new(
                input,
                nom::error::ErrorKind::Char,
                ErrorKind::Host,
                "label contains an invalid chracter",
            )));
        }

        if remain.starts_with('.') {
            let mut current_input = remain;
            let mut position = label_str.len();

            while let Some(remain2) = current_input.strip_prefix('.') {
                let (remain2, label_str2) = self.label(remain2)?;

                if self.validator.is_invalid_char(label_str2) {
                    return Err(nom::Err::Error(HostError::new(
                        input,
                        nom::error::ErrorKind::Char,
                        ErrorKind::Host,
                        "label contains an invalid chracter",
                    )));
                }

                current_input = remain2;
                position += label_str2.len() + 1;
            }
            Ok((current_input, &input[0..position]))
        } else {
            Ok((remain, label_str))
        }
    }

    fn label<'a>(&self, input: &'a str) -> IResult<&'a str, &'a str, HostError<&'a str>> {
        if input.is_empty() {
            return Err(nom::Err::Error(HostError::new(
                input,
                nom::error::ErrorKind::Char,
                ErrorKind::Empty,
                "label must not be empty",
            )));
        }

        let first_char = input.chars().next().unwrap();
        if !self.validator.is_valid_start_char(first_char) {
            return Err(nom::Err::Error(HostError::new(
                input,
                nom::error::ErrorKind::Char,
                ErrorKind::Host,
                "label must start with an allowed character",
            )));
        }

        let (remain, label) = self.validator.while_valid(input, first_char);

        if label.is_empty() {
            return Err(nom::Err::Error(HostError::new(
                input,
                nom::error::ErrorKind::Char,
                ErrorKind::Empty,
                "label must not be empty",
            )));
        }

        Ok((remain, label))
    }
}

pub fn standard_host_validate(input: &str) -> bool {
    StandardHostValidator.validate_host(input)
}

#[derive(Debug, Clone, Default)]
pub struct StandardHostValidator;
impl StandardHostValidator {
    /// Validates a vendor name according to RFC 952 hostname rules.
    ///
    /// A valid vendor name follows these rules:
    /// - Must consist of one or more segments separated by dots
    /// - Each segment must contain only alphanumeric characters and hyphens
    /// - No segment can start or end with a hyphen
    /// - No segment can be empty
    ///
    /// # Examples
    ///
    /// ```
    /// use ircv3_tags::host::standard_host_validate;
    ///
    /// assert!(standard_host_validate("example.com"));
    /// assert!(standard_host_validate("sub.example.com"));
    /// assert!(!standard_host_validate("example-.com"));
    /// assert!(!standard_host_validate("example..com"));
    /// ```
    pub fn validate_host(&self, input: &str) -> bool {
        if input.is_empty()
            || !input.starts_with(|c| self.is_valid_start_char(c))
            || self.is_invalid_char(input)
        {
            return false;
        }

        input.split('.').collect::<Vec<_>>().iter().all(|segment| {
            !segment.is_empty()
                && segment.starts_with(|c| self.is_valid_start_char(c))
                && !self.is_invalid_char(segment)
        })
    }
}

impl CharValidator for StandardHostValidator {
    fn is_valid_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-'
    }
    fn is_valid_start_char(&self, c: char) -> bool {
        c.is_ascii_alphabetic()
    }
    fn is_invalid_char(&self, s: &str) -> bool {
        s.contains('_') || s.ends_with('-') || s.contains("--")
    }
}

#[derive(Debug, PartialEq)]
pub struct HostError<I> {
    pub input: I,
    pub code: nom::error::ErrorKind,
    pub error: ErrorKind,
    pub reason: &'static str,
}

impl<I> HostError<I> {
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

impl<I> nom::error::ParseError<I> for HostError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        HostError {
            input,
            code: kind,
            error: ErrorKind::NomError,
            reason: "failed to parse host",
        }
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
