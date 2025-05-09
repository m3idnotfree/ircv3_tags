//! From RFC 952:
//! - A "name" (hostname) is a text string up to 24 characters drawn from the alphabet (A-Z),
//!   digits (0-9), minus sign (-), and period (.).
//! - Names must start with a letter, end with a letter or digit, and have as interior
//!   characters only letters, digits, and hyphen.
//! - Segments are separated by periods.
//! - A valid hostname can have multiple segments.
//!
//! - Cannot contain consecutive hyphens
//!
//! For more information, see the [RFC 952](https://datatracker.ietf.org/doc/html/rfc952) \[DNS:4\]
//!
//! # Example
//! ```
//! let input = "example.com";
//! let (remain, messages) = ircv3_tags::host(input).unwrap();
//! assert_eq!(messages, "example.com");
//! ```
//! For more information, see the
use nom::{
    branch::alt,
    character::complete::{alpha1, alphanumeric1, char},
    combinator::recognize,
    error::{Error, ErrorKind as NrrKind},
    multi::many0,
    Err::Error as Nrr,
    IResult, Parser,
};

/// RFC 952 (host) parser
pub fn host(input: &str) -> IResult<&str, &str> {
    if input.is_empty() {
        return Err(Nrr(Error::new(input, NrrKind::Char)));
    }

    if !input.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(Nrr(Error::new(input, NrrKind::Char)));
    }

    let (remain, label_str) = label(input)?;

    if label_str.is_empty() || label_str.ends_with('-') || label_str.contains("--") {
        return Err(Nrr(Error::new(input, NrrKind::Char)));
    }

    if remain.starts_with('.') {
        let mut current_input = remain;
        let mut position = label_str.len();

        while let Ok((remain2, _)) = dot(current_input) {
            let (remain2, label_str2) = label(remain2)?;

            if label_str2.is_empty() || label_str2.ends_with("-") || label_str2.contains("--") {
                return Err(Nrr(Error::new(input, NrrKind::Char)));
            }

            current_input = remain2;
            position += label_str2.len() + 1;
        }
        Ok((current_input, &input[0..position]))
    } else {
        Ok((remain, label_str))
    }
}
/// RFC 978 (host) parser with helpful error messages
/// ```toml
/// ircv3_tags = { version = "2", features = ["debug"]}
/// ````
///
/// # Example
/// ```
/// # #[cfg(feature = "debug")]
/// let input = "example.com";
/// let (remain, messages) = ircv3_tags::debug_host(input).unwrap();
/// assert_eq!(messages, "example.com");
///
/// assert_eq!(
///     ircv3_tags::debug_host("invalid-"),
///     Err(nom::Err::Error(ircv3_tags::HostError {
///         input: "invalid-",
///         code: nom::error::ErrorKind::Char,
///         error: ircv3_tags::ErrorKind::EndsWithLetterOrDigit,
///         reason: "end with an ascii alphabet or ascii digit",
///     }))
/// );
/// ```
#[cfg(feature = "debug")]
pub fn debug_host(input: &str) -> IResult<&str, &str, HostError<&str>> {
    if input.is_empty() {
        return Err(invalid_empty_label(input));
    }

    if !input.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(invalid_start_with_letter(input));
    }

    let (remain, label_str) =
        label(input).map_err(|err| err.map(|e| invalid_label_input(e.input, e.code)))?;

    if label_str.ends_with('-') {
        return Err(invalid_ends_with(input));
    }

    if label_str.contains("--") {
        return Err(invalid_consecutive_hiphens(input));
    }

    if remain.starts_with('.') {
        let mut current_input = remain;
        let mut position = label_str.len();

        while let Ok((remain2, _)) = dot(current_input) {
            let (remain2, label_str2) =
                label(remain2).map_err(|err| err.map(|e| invalid_label_input(e.input, e.code)))?;

            if label_str2.ends_with("-") {
                return Err(invalid_ends_with(input));
            }

            if label_str2.contains("--") {
                return Err(invalid_consecutive_hiphens(input));
            }

            current_input = remain2;
            position += label_str2.len() + 1;
        }
        Ok((current_input, &input[0..position]))
    } else {
        Ok((remain, label_str))
    }
}

fn label(input: &str) -> IResult<&str, &str> {
    recognize((alpha1, many0(alt((alphanumeric1, recognize(char('-'))))))).parse(input)
}

fn dot(input: &str) -> IResult<&str, char> {
    char('.').parse(input)
}

pub fn validate_host(input: &str) -> bool {
    if input.is_empty()
        || !input.chars().next().unwrap().is_ascii_alphabetic()
        || input.ends_with('-')
        || input.contains("--")
    {
        return false;
    }

    input
        .split('.')
        .collect::<Vec<_>>()
        .iter()
        .all(|segment| match label(segment) {
            Err(_) => false,
            Ok((remain, _)) => {
                remain.is_empty()
                    && segment.chars().next().unwrap().is_ascii_alphabetic()
                    && !segment.ends_with('-')
            }
        })
}

pub fn validate_label(input: &str) -> bool {
    if input.is_empty()
        || !input.chars().next().unwrap().is_ascii_alphabetic()
        || input.ends_with('-')
        || input.contains("--")
    {
        return false;
    }
    match label(input) {
        Err(_) => false,
        Ok((remain, _)) => {
            remain.is_empty()
                && input.chars().next().unwrap().is_ascii_alphabetic()
                && !input.ends_with('-')
        }
    }
}

#[cfg(feature = "debug")]
#[derive(Debug, PartialEq)]
pub struct HostError<I> {
    pub input: I,
    pub code: NrrKind,
    pub error: ErrorKind,
    pub reason: &'static str,
}

#[cfg(feature = "debug")]
impl<I> nom::error::ParseError<I> for HostError<I> {
    fn from_error_kind(input: I, kind: NrrKind) -> Self {
        HostError {
            input,
            code: kind,
            error: ErrorKind::NomError,
            reason: "characters only letters, digits, and hyphen",
        }
    }

    fn append(_: I, _: NrrKind, other: Self) -> Self {
        other
    }
}

#[cfg(feature = "debug")]
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    StartWithLetter,
    EndsWithLetterOrDigit,
    NoConsecutiveHyphens,
    Empty,
    InvalidLabel,
    NomError,
}

fn invalid_empty_label<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Nrr(HostError {
        input,
        code: NrrKind::Alpha,
        error: ErrorKind::Empty,
        reason: "label must start with the ascii alphabet",
    })
}

fn invalid_label_input<I>(input: I, code: NrrKind) -> HostError<I>
where
    I: std::fmt::Display + Copy,
{
    HostError {
        input,
        code,
        error: ErrorKind::InvalidLabel,
        reason: "failed to parse label",
    }
}

fn invalid_start_with_letter<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Nrr(HostError {
        input,
        code: NrrKind::Alpha,
        error: ErrorKind::StartWithLetter,
        reason: "label must start with the ascii alphabet",
    })
}

fn invalid_ends_with<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Nrr(HostError {
        input,
        code: NrrKind::Char,
        error: ErrorKind::EndsWithLetterOrDigit,
        reason: "end with an ascii alphabet or ascii digit",
    })
}
fn invalid_consecutive_hiphens<I>(input: I) -> nom::Err<HostError<I>>
where
    I: std::fmt::Display + Copy,
{
    Nrr(HostError {
        input,
        code: NrrKind::Char,
        error: ErrorKind::NoConsecutiveHyphens,
        reason: "cannot contain consecutive hyphens",
    })
}
#[cfg(test)]
mod tests {
    use crate::host::validate_label;

    macro_rules! fn_test {
        ($name:ident, $fn:expr, [$($test:literal),+$(,)?]) => {
            #[test]
            fn $name(){
                let inputs = [$($test),+];

                for input in inputs {
                    assert!($fn(input).is_ok(), "{}", input);
                    let (remain, _) = $fn(input).unwrap();
                    assert!(remain.is_empty());
                }
            }
        };

        (err, $name:ident, $fn:expr, [$($test:literal),+$(,)?]) => {
            #[test]
            fn $name(){
                let inputs = [$($test),+];

                for input in inputs {
                    assert!($fn(input).is_err(), "'{}'", input);
                }
            }
        };

    }

    macro_rules! fn_tests {
        ($fn:expr) => {
            fn_test!(
                simple,
                $fn,
                [
                    "example.com",
                    "host.a.z",
                    "server.host1",
                    "a-b-c.x-y-z123",
                ]
            );
            fn_test!(
                normal,
                $fn,
                [
                    "my-host-name",
                    "my-host-name.server-01",
                    "web-server-prod",
                    "app-server-1.database-cluster-a"
                ]
            );
            fn_test!(edge_cases,$fn,[
                "a-very-long-hostname-that-is-still-valid-according-to-rfc952-specs-with-multiple-hyphens",
                "a-b",
                "x1",
                "a-1"
            ]);
            fn_test!(
                max_length,
                $fn,
                ["abcdefghijklmnopqrstuvwxyz-0123456789-abcdefghijklmnopqrstuvwxyz"]
            );
        };
        ($fn:expr, err) => {
            fn_test!(
                err,
                starts_with_digit,
                $fn,
                [
                    "1host",
                    "host.1host",
                    "9server",
                    "123-invalid",
                    "0invalid"
                ]
            );
            fn_test!(
                err,
                starts_with_hyphen,
                $fn,
                [
                    "-host",
                    "host.-host",
                    "-server-1",
                    "-invalid-name"
                ]
            );
            fn_test!(
                err,
                ends_with_hyphen,
                $fn,
                [
                    "host-",
                    "host.host-",
                    "server-name-",
                    "server.server-name-",
                    "invalid-"
                ]
            );
            fn_test!(
                err,
                consecutive_hyphens,
                $fn,
                [
                    "host--name",
                    "host.host--name",
                    "server--01",
                    "double--hyphen--test",
                    "triple---hyphen",
                    "quadruple----hyphen"
                ]
            );
            fn_test!(err, empty, $fn, [""]);
            fn_test!(err, whitespace_only, $fn, [" ", "  ", "\t", "\n"]);
        }
    }
    fn_tests!(crate::host::host);
    fn_tests!(crate::host::host, err);

    // fn_tests!(crate::host::debug_host);
    // fn_tests!(crate::host::debug_host, err);

    #[test]
    fn validate_hostnames() {
        let inputs_ok = [
            "a",
            "my-host-name",
            "server-01",
            "web-server-prod",
            "database-cluster-a",
            "app-server-1",
            "a-host",
            "ho-st",
            "hosta",
            "host1",
            "hostZ",
            "ahost",
            "Zhost",
        ];
        let inputs_err = [
            "host--name",
            "server--01",
            "double--hyphen--test",
            "triple---hyphen",
            "host_name",
            "terver.01",
            "web@server",
            "database$cluster",
            "app server",
            "test#host",
            "invalid!",
            "not_valid",
            "special&chars",
            "spaces are invalid",
            "uppercase+lowercase",
            "",
            " ",
            "  ",
            "\t",
            "\n",
            "almost-valid-but-has-a-space at-the-end",
            "valid-prefix-but-has.dot",
            "valid-start-invalid@end",
            "a-b-c-_-y-z",
            "1host",
            "-host",
            "host-",
            "-host",
            "host-",
            "ho--st",
        ];
        for ok in inputs_ok {
            assert!(validate_label(ok), "{}", ok);
        }

        for err in inputs_err {
            assert!(!validate_label(err), "{}", err);
        }
    }

    #[cfg(feature = "debug")]
    #[test]
    fn host_debug() {
        use crate::{debug_host, HostError};

        assert_eq!(
            debug_host(""),
            Err(nom::Err::Error(HostError {
                input: "",
                code: nom::error::ErrorKind::Alpha,
                error: crate::ErrorKind::Empty,
                reason: "label must start with the ascii alphabet",
            }))
        );

        let inputs = ["-", "0", " "];

        for input in inputs {
            assert_eq!(
                debug_host(input),
                Err(nom::Err::Error(HostError {
                    input,
                    code: nom::error::ErrorKind::Alpha,
                    error: crate::ErrorKind::StartWithLetter,
                    reason: "label must start with the ascii alphabet",
                }))
            );
        }

        assert_eq!(
            debug_host("a-"),
            Err(nom::Err::Error(HostError {
                input: "a-",
                code: nom::error::ErrorKind::Char,
                error: crate::ErrorKind::EndsWithLetterOrDigit,
                reason: "end with an ascii alphabet or ascii digit",
            }))
        );

        assert_eq!(
            debug_host("a--b"),
            Err(nom::Err::Error(HostError {
                input: "a--b",
                code: nom::error::ErrorKind::Char,
                error: crate::ErrorKind::NoConsecutiveHyphens,
                reason: "cannot contain consecutive hyphens",
            }))
        );
    }
}
