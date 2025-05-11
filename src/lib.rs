//!
//! # IRCv3 Message Tags Parser
//!
//! ## Examples
//!
//! ```rust
//! let input = "@id=234AB;+example.com/key=value :nick!user@host PRIVMSG #channel :Hello";
//! let (remain, tags) = ircv3_tags::parse(input);
//!
//! assert_eq!(remain, ":nick!user@host PRIVMSG #channel :Hello");
//! assert_eq!(tags.get("id"), Some("234AB"));
//! assert_eq!(tags.get("+example.com/key"), Some("value"));
//! ```
//!
//! For more information, see the [IRCv3 Message Tags specification](https://ircv3.net/specs/extensions/message-tags.html).
//!
use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::take_till,
    character::complete::{alphanumeric1, char, one_of, space1},
    combinator::{opt, recognize},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

use error::check_starts_ascii_alph;

mod error;
mod host;

pub use error::{ErrorKind, HostError, IRCv3TagsError};
pub use host::{debug_host, host, validate_host, validate_label};

#[cfg(not(feature = "allow-underdash_key_name"))]
pub(crate) const HYPHEN: &str = "-";
#[cfg(feature = "allow-underdash_key_name")]
pub(crate) const HYPHEN: &str = "-_";

/// Parses only the tags portion of an IRC message, using an unwrapping fallback for errors
///
/// # Examples
///
/// ```
/// let input = "@id=123;time=2020-01-01T00:00:00Z :nick!user@host PRIVMSG #channel :Hello";
/// let (remain, tags) = ircv3_tags::parse(input);
///
/// assert_eq!(remain, ":nick!user@host PRIVMSG #channel :Hello");
/// assert_eq!(tags.get("id"), Some("123"));
/// ```
pub fn parse(input: &str) -> (&str, IRCv3Tags<'_>) {
    try_parse(input).unwrap()
}

/// Safely tries to parse IRC message tags.
///
/// Similar to `parse`, but returns a `Result` instead of unwrapping.
///
/// # Examples
///
/// ```
/// let input = "@id=123 :nick!user@host PRIVMSG #channel :Hello";
/// let result = ircv3_tags::try_parse(input);
/// assert!(result.is_ok());
/// ```
pub fn try_parse(input: &str) -> IResult<&str, IRCv3Tags<'_>> {
    debug_parse(input).map_err(|err| err.map(|e| nom::error::Error::new(e.input, e.code)))
}

/// Parse to IRCv2 Message tags with helpful error messages
pub fn debug_parse(input: &str) -> IResult<&str, IRCv3Tags<'_>, IRCv3TagsError<&str>> {
    if input.is_empty() || !input.starts_with('@') {
        return Err(nom::Err::Error(IRCv3TagsError {
            input,
            code: nom::error::ErrorKind::Char,
            error: ErrorKind::TagErrorStartWithLetter,
            reason: "tag must start with an '@'",
        }));
    }

    let (remain, tags) = delimited(char('@'), tags, space1)
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

#[derive(Clone, Debug, PartialEq)]
pub struct IRCv3Tags<'a>(Vec<(&'a str, Option<&'a str>)>);

impl<'a> IRCv3Tags<'a> {
    /// Gets the raw value for a key in the tag list without unescaping.
    ///
    /// * `None` if the key doesn't exist
    /// * `Some("")` if the key exists with an empty value
    /// * `Some(value)` if the key exists with a value
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.0.iter().find_map(|(k, v)| {
            if *k == key {
                Some(v.unwrap_or(""))
            } else {
                None
            }
        })
    }

    /// Gets the escaped value for a key in the tag list.
    ///
    /// This method performs the same lookup as `get()` but also escapes
    /// the value according to the IRCv3 tag specification.
    ///
    /// * `None` if the key doesn't exist
    /// * `Some("")` if the key exists with an empty value
    /// * `Some(value)` if the key exists with a value (escaped)
    ///
    /// # Examples
    ///
    /// ```
    /// use ircv3_tags::parse;
    ///
    /// let input = "@key=value\\:with\\sescapes :nick PRIVMSG #channel :Hello";
    /// let (_, tags) = parse(input);
    ///
    /// assert_eq!(tags.get("key"), Some("value\\:with\\sescapes"));
    ///
    /// assert_eq!(tags.get_escaped("key"), Some("value;with escapes".to_string()));
    /// ```
    pub fn get_escaped(&self, key: &str) -> Option<String> {
        self.get(key).map(unescaped_to_escaped)
    }

    /// Converts the tags to a HashMap where empty values are represented as empty strings.
    pub fn to_hashmap(&self) -> HashMap<&'a str, &'a str> {
        self.0.iter().map(|(k, v)| (*k, v.unwrap_or(""))).collect()
    }

    /// Consumes the tags and converts them to a HashMap where empty values are represented as empty strings.
    pub fn into_hashmap(self) -> HashMap<&'a str, &'a str> {
        self.0
            .into_iter()
            .map(|(k, v)| (k, v.unwrap_or("")))
            .collect()
    }

    /// Converts the tags to a HashMap with escaped values.
    pub fn to_hashmap_escaped(&self) -> HashMap<&'a str, String> {
        self.0
            .iter()
            .map(|(k, v)| (*k, unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    /// Consumes the tags and converts them to a HashMap with escaped values.
    pub fn into_hashmap_escaped(self) -> HashMap<&'a str, String> {
        self.0
            .into_iter()
            .map(|(k, v)| (k, unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    /// Converts the tags to an owned HashMap where the keys and values are owned Strings.
    pub fn to_map(&self) -> HashMap<String, String> {
        self.0
            .iter()
            .map(|(k, v)| (k.to_string(), v.unwrap_or("").to_string()))
            .collect()
    }

    /// Consumes the tags and converts them to an owned HashMap.
    pub fn into_map(self) -> HashMap<String, String> {
        self.0
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.unwrap_or("").to_string()))
            .collect()
    }

    /// Converts the tags to an owned HashMap with escaped values.
    pub fn to_map_escaped(&self) -> HashMap<String, String> {
        self.0
            .iter()
            .map(|(k, v)| (k.to_string(), unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    /// Consumes the tags and converts them to an owned HashMap with escaped values.
    pub fn into_map_escaped(self) -> HashMap<String, String> {
        self.0
            .into_iter()
            .map(|(k, v)| (k.to_string(), unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }
}

impl std::fmt::Display for IRCv3Tags<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter().peekable();
        while let Some((key, value)) = iter.next() {
            write!(
                f,
                "{}: {}",
                key,
                value
                    .as_ref()
                    .map_or("''", |v| if v.is_empty() { "''" } else { v })
            )?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

#[allow(clippy::type_complexity)]
fn tags(input: &str) -> IResult<&str, Vec<(&str, Option<&str>)>, IRCv3TagsError<&str>> {
    separated_list1(char(';'), tag).parse(input)
}

fn tag(input: &str) -> IResult<&str, (&str, Option<&str>), IRCv3TagsError<&str>> {
    (key, opt(preceded(char('='), escaped_value))).parse(input)
}

fn key(input: &str) -> IResult<&str, &str, IRCv3TagsError<&str>> {
    recognize((
        opt(client_prefix),
        opt(terminated(vendor, char('/'))),
        key_name,
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

fn client_prefix(input: &str) -> IResult<&str, char, IRCv3TagsError<&str>> {
    char('+').parse(input)
}

/// Parses a key name according to the IRCv3 specifications.
///
/// A key name can contain alphanumeric characters and hyphens.
///
/// Note: This implementation allows keys starting with or ending with hyphens,
/// which might need to be validated separately in strict mode. The current behavior
/// is maintained for compatibility with existing parsers.
fn key_name(input: &str) -> IResult<&str, &str, IRCv3TagsError<&str>> {
    if input.is_empty() {
        return Err(nom::Err::Error(IRCv3TagsError {
            input,
            code: nom::error::ErrorKind::Char,
            error: ErrorKind::Empty,
            reason: "tag key must start with the ascii alphabet",
        }));
    }

    if !check_starts_ascii_alph(input) || input.starts_with(HYPHEN) {
        return Err(nom::Err::Error(IRCv3TagsError {
            input,
            code: nom::error::ErrorKind::Char,
            error: ErrorKind::TagErrorStartWithLetter,
            reason: "tag key must start with the ascii alphabet",
        }));
    }

    // recognize(many1(alt((alphanumeric1, recognize(char(HYPHEN)))))).parse(input)
    recognize(many1(alt((alphanumeric1, recognize(one_of(HYPHEN)))))).parse(input)
}

/// Parses an escaped value which is a sequence of zero or more UTF-8 characters
/// except NUL, CR, LF, semicolon (`;`) and SPACE.
fn escaped_value(input: &str) -> IResult<&str, &str, IRCv3TagsError<&str>> {
    take_till(|c| c == '\0' || c == '\r' || c == '\n' || c == ';' || c == ' ').parse(input)
}

/// Parses a vendor part of the tag which follows the format `vendor/` where vendor
/// must be a valid hostname as defined in RFC 952.
///
/// A valid hostname consists of:
/// - Only alphanumeric characters, dots '.', and hyphens '-'
/// - Segments cannot start or end with a hyphen '-'
/// - Must end with a forward slash '/'
fn vendor(input: &str) -> IResult<&str, &str, IRCv3TagsError<&str>> {
    debug_host(input).map_err(|err| {
        err.map(|e| IRCv3TagsError {
            input: e.input,
            code: e.code,
            error: e.error,
            reason: e.reason,
        })
    })
}

/// Validates a key name according to strict IRCv3 specifications.
///
/// Rules:
/// - Must contain alphanumeric characters and hyphens only
/// - Cannot start with a hyphen
/// - Cannot end with a hyphen
/// - Cannot be empty
pub fn validate_key_name(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }

    if key.starts_with(HYPHEN) || key.ends_with(HYPHEN) {
        return false;
    }

    key.chars()
        .all(|c| c.is_alphanumeric() || HYPHEN.contains(c))
}

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
/// use ircv3_tags::validate_vendor;
///
/// assert!(validate_vendor("example.com"));
/// assert!(validate_vendor("sub.example.com"));
/// assert!(!validate_vendor("example-.com"));
/// assert!(!validate_vendor("example..com"));
/// ```
pub fn validate_vendor(input: &str) -> bool {
    let segments: Vec<&str> = input.split('.').collect();
    segments
        .iter()
        .all(|segment| !segment.starts_with('-') && !segment.ends_with('-') && !segment.is_empty())
}

/// Unescapes an IRCv3 tag value according to the specification.
///
/// The following sequences are unescaped:
/// - `\:` → `;` (backslash + colon → semicolon)
/// - `\s` → ` ` (backslash + s → space)
/// - `\\` → `\` (backslash + backslash → backslash)
/// - `\r` → CR (backslash + r → carriage return)
/// - `\n` → LF (backslash + n → line feed)
///
/// # Examples
///
/// ```
/// use ircv3_tags::unescaped_to_escaped;
///
/// assert_eq!(unescaped_to_escaped("hello\\sworld"), "hello world");
/// assert_eq!(unescaped_to_escaped("semi\\:colon"), "semi;colon");
/// assert_eq!(unescaped_to_escaped("back\\\\slash"), "back\\slash");
/// ```
pub fn unescaped_to_escaped(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(':') => result.push(';'),
                Some('s') => result.push(' '),
                Some('\\') => result.push('\\'),
                Some('r') => result.push('\r'),
                Some('n') => result.push('\n'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod test {
    use crate::{
        escaped_value, key, key_name, parse, tag, unescaped_to_escaped, vendor, IRCv3TagsError,
    };

    #[test]
    fn test_tag() {
        let input = "tag-name";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("tag-name", None))));

        let input = "tag-name=";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("tag-name", Some("")))));

        let input = "tag-name=value";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("tag-name", Some("value")))));

        let input = "+tag-name";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("+tag-name", None))));

        let input = "+tag-name=";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("+tag-name", Some("")))));

        let input = "+tag-name=value";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("+tag-name", Some("value")))));

        let input = "example.com/tag-name";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("example.com/tag-name", None))));

        let input = "example.com/tag-name=";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("example.com/tag-name", Some("")))));

        let input = "example.com/tag-name=value";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("example.com/tag-name", Some("value")))));

        let input = "+example.com/tag-name";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("+example.com/tag-name", None))));

        let input = "+example.com/tag-name=";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("+example.com/tag-name", Some("")))));

        let input = "+example.com/tag-name=value";
        let result = tag(input);
        assert_eq!(result, Ok(("", ("+example.com/tag-name", Some("value")))));
    }

    #[test]
    fn test_key() {
        let input = "tag-name";
        let result = key(input);
        assert_eq!(result, Ok(("", "tag-name")));

        let input = "+tag-name";
        let result = key(input);
        assert_eq!(result, Ok(("", "+tag-name")));

        let input = "example.com/tag-name";
        let result = key(input);
        assert_eq!(result, Ok(("", "example.com/tag-name")));

        let input = "+example.com/tag-name";
        let result = key(input);
        assert_eq!(result, Ok(("", "+example.com/tag-name")));
    }

    #[test]
    fn test_key_name() {
        let input = "example";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "example")));

        let input = "example12345";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "example12345")));

        let input = "exam-ple";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "exam-ple")));

        assert!(key_name("12345").is_err());
        assert!(key_name("-example").is_err());
    }

    #[cfg(feature = "allow-underdash_key_name")]
    #[test]
    fn test_allow_host() {
        let input = "exam_ple";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "exam_ple")));
        assert_eq!(key_name("example_"), Ok(("", "example_")));

        assert!(key_name("_example").is_err(),);
    }

    #[test]
    fn test_escaped_value() {
        let input = "";
        let result = escaped_value(input);
        assert_eq!(result, Ok(("", "")));

        let input = "\0";
        let result = escaped_value(input);
        assert_eq!(result, Ok(("\0", "")));

        let input = "\r";
        let result = escaped_value(input);
        assert_eq!(result, Ok(("\r", "")));

        let input = "\n";
        let result = escaped_value(input);
        assert_eq!(result, Ok(("\n", "")));

        let input = ";";
        let result = escaped_value(input);
        assert_eq!(result, Ok((";", "")));

        let input = " ";
        let result = escaped_value(input);
        assert_eq!(result, Ok((" ", "")));
    }

    #[test]
    fn test_escaped_semicolon() {
        let input = "@key=value\\: :nick!user@host PRIVMSG #channel :Hello";
        let (_, tags) = parse(input);
        assert_eq!(tags.get("key"), Some("value\\:"));
        assert_eq!(unescaped_to_escaped("value\\:"), "value;");

        let input = "@key=a\\:b\\sc\\\\d\\re\\nf :rest";
        let (_, tags) = parse(input);
        assert_eq!(tags.get("key"), Some("a\\:b\\sc\\\\d\\re\\nf"));
        assert_eq!(
            unescaped_to_escaped("a\\:b\\sc\\\\d\\re\\nf"),
            "a;b c\\d\re\nf"
        );
    }

    #[test]
    fn test_unescape_value() {
        assert_eq!(unescaped_to_escaped("hello\\sworld"), "hello world");
        assert_eq!(unescaped_to_escaped("semi\\:colon"), "semi;colon");
        assert_eq!(unescaped_to_escaped("back\\\\slash"), "back\\slash");
        assert_eq!(unescaped_to_escaped("new\\nline"), "new\nline");
        assert_eq!(
            unescaped_to_escaped("carriage\\rreturn"),
            "carriage\rreturn"
        );
        assert_eq!(unescaped_to_escaped("plain text"), "plain text");
        assert_eq!(unescaped_to_escaped("trailing\\"), "trailing\\");
        assert_eq!(unescaped_to_escaped("unknown\\xescape"), "unknown\\xescape");
        assert_eq!(unescaped_to_escaped(""), "");
    }

    #[test]
    fn test_vendor() {
        let input = "example.com/tag-name";
        let result = vendor(input);
        assert_eq!(result, Ok(("/tag-name", "example.com")));
    }

    #[test]
    fn test_unescaped_methods() {
        let input = "@escaped=a\\:b\\sc\\\\d\\re\\nf;normal=value :rest";
        let (_, tags) = parse(input);
        assert_eq!(tags.get("escaped"), Some(r"a\:b\sc\\d\re\nf"));
        assert_eq!(
            tags.get_escaped("escaped"),
            Some("a;b c\\d\re\nf".to_string())
        );
        assert_eq!(tags.get_escaped("normal"), Some("value".to_string()));
        assert_eq!(tags.get_escaped("missing"), None);

        let unescaped_map = tags.to_hashmap_escaped();
        assert_eq!(
            unescaped_map.get("escaped"),
            Some(&"a;b c\\d\re\nf".to_string())
        );
        assert_eq!(unescaped_map.get("normal"), Some(&"value".to_string()));

        let owned_map = tags.to_map();
        assert_eq!(
            owned_map.get("escaped"),
            Some(&r"a\:b\sc\\d\re\nf".to_string())
        );

        let owned_unescaped_map = tags.to_map_escaped();
        assert_eq!(
            owned_unescaped_map.get("escaped"),
            Some(&"a;b c\\d\re\nf".to_string())
        );
        assert_eq!(
            owned_unescaped_map.get("normal"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_consuming_methods() {
        let input = "@escaped=a\\:b\\sc\\\\d\\re\\nf;normal=value :rest";

        let (_, tags) = parse(input);
        let map = tags.into_hashmap();
        assert_eq!(map.get("escaped"), Some(&r"a\:b\sc\\d\re\nf"));
        assert_eq!(map.get("normal"), Some(&"value"));

        let (_, tags) = parse(input);
        let unescaped_map = tags.into_hashmap_escaped();
        assert_eq!(
            unescaped_map.get("escaped"),
            Some(&"a;b c\\d\re\nf".to_string())
        );
        assert_eq!(unescaped_map.get("normal"), Some(&"value".to_string()));

        let (_, tags) = parse(input);
        let owned_map = tags.into_map();
        assert_eq!(
            owned_map.get("escaped"),
            Some(&r"a\:b\sc\\d\re\nf".to_string())
        );
        assert_eq!(owned_map.get("normal"), Some(&"value".to_string()));

        let (_, tags) = parse(input);
        let owned_unescaped_map = tags.into_map_escaped();
        assert_eq!(
            owned_unescaped_map.get("escaped"),
            Some(&"a;b c\\d\re\nf".to_string())
        );
        assert_eq!(
            owned_unescaped_map.get("normal"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn debug_keyname() {
        let input = "";
        assert_eq!(
            key_name(input),
            Err(nom::Err::Error(IRCv3TagsError {
                input,
                code: nom::error::ErrorKind::Char,
                error: crate::error::ErrorKind::Empty,
                reason: "tag key must start with the ascii alphabet",
            }))
        );
        assert!(key_name("-").is_err());
        assert!(key_name("_").is_err());
        assert!(key_name("!").is_err());
        assert!(key_name(" ").is_err());
        assert!(key_name(":").is_err());
        assert!(key_name(";").is_err());
        assert!(key_name("@").is_err());
        assert!(key_name("=").is_err());
        assert!(key_name("😀").is_err());

        assert_eq!(
            key("example.com/"),
            Err(nom::Err::Error(IRCv3TagsError {
                input: "",
                code: nom::error::ErrorKind::Char,
                error: crate::error::ErrorKind::Empty,
                reason: "tag key must start with the ascii alphabet",
            }))
        );
    }
}
