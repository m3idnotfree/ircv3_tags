//!
//! # IRCv3 Tags Parser
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
    character::complete::{alphanumeric1, char, space1},
    combinator::{opt, peek, recognize},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

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
    let (remain, tags) = delimited(char('@'), tags, space1).parse(input).unwrap();
    (remain, IRCv3Tags(tags))
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
pub fn try_parse(input: &str) -> nom::IResult<&str, IRCv3Tags<'_>> {
    let (remain, tags) = delimited(char('@'), tags, space1).parse(input)?;
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

    /// Gets the unescaped value for a key in the tag list.
    ///
    /// This method performs the same lookup as `get()` but also unescapes
    /// the value according to the IRCv3 tag specification.
    ///
    /// * `None` if the key doesn't exist
    /// * `Some("")` if the key exists with an empty value
    /// * `Some(value)` if the key exists with a value (unescaped)
    ///
    /// # Examples
    ///
    /// ```
    /// use ircv3_tags::parse;
    ///
    /// let input = "@key=value\\:with\\sescapes :nick PRIVMSG #channel :Hello";
    /// let (_, tags) = parse(input);
    ///
    /// // With get(), escape sequences are preserved
    /// assert_eq!(tags.get("key"), Some("value\\:with\\sescapes"));
    ///
    /// // With get_unescaped(), escape sequences are converted
    /// assert_eq!(tags.get_unescaped("key"), Some("value;with escapes".to_string()));
    /// ```
    pub fn get_unescaped(&self, key: &str) -> Option<String> {
        self.get(key).map(unescape_value)
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

    /// Converts the tags to a HashMap with unescaped values.
    pub fn to_hashmap_unescaped(&self) -> HashMap<&'a str, String> {
        self.0
            .iter()
            .map(|(k, v)| (*k, unescape_value(v.unwrap_or(""))))
            .collect()
    }

    /// Consumes the tags and converts them to a HashMap with unescaped values.
    pub fn into_hashmap_unescaped(self) -> HashMap<&'a str, String> {
        self.0
            .into_iter()
            .map(|(k, v)| (k, unescape_value(v.unwrap_or(""))))
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

    /// Converts the tags to an owned HashMap with unescaped values.
    pub fn to_map_unescaped(&self) -> HashMap<String, String> {
        self.0
            .iter()
            .map(|(k, v)| (k.to_string(), unescape_value(v.unwrap_or(""))))
            .collect()
    }

    /// Consumes the tags and converts them to an owned HashMap with unescaped values.
    pub fn into_map_unescaped(self) -> HashMap<String, String> {
        self.0
            .into_iter()
            .map(|(k, v)| (k.to_string(), unescape_value(v.unwrap_or(""))))
            .collect()
    }
}

fn tags(input: &str) -> IResult<&str, Vec<(&str, Option<&str>)>> {
    separated_list1(char(';'), tag).parse(input)
}

fn tag(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    (key, opt(preceded(char('='), escaped_value))).parse(input)
}

fn key(input: &str) -> IResult<&str, &str> {
    recognize((
        opt(client_prefix),
        opt(terminated(vendor, char('/'))),
        key_name,
    ))
    .parse(input)
}

fn client_prefix(input: &str) -> IResult<&str, char> {
    char('+').parse(input)
}

/// Parses a key name according to the IRCv3 specifications.
///
/// A key name can contain alphanumeric characters and hyphens.
///
/// Note: This implementation allows keys starting with or ending with hyphens,
/// which might need to be validated separately in strict mode. The current behavior
/// is maintained for compatibility with existing parsers.
fn key_name(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((alphanumeric1, recognize(char('-')))))).parse(input)
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

    if key.starts_with('-') || key.ends_with('-') {
        return false;
    }

    key.chars().all(|c| c.is_alphanumeric() || c == '-')
}

/// Parses an escaped value which is a sequence of zero or more UTF-8 characters
/// except NUL, CR, LF, semicolon (`;`) and SPACE.
///
/// Returns a string of the parsed characters or an empty string if there are none.
fn escaped_value(input: &str) -> IResult<&str, &str> {
    // Take until we find a NUL, CR, LF, semicolon, or space
    take_till(|c| c == '\0' || c == '\r' || c == '\n' || c == ';' || c == ' ').parse(input)
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
/// use ircv3_tags::unescape_value;
///
/// assert_eq!(unescape_value("hello\\sworld"), "hello world");
/// assert_eq!(unescape_value("semi\\:colon"), "semi;colon");
/// assert_eq!(unescape_value("back\\\\slash"), "back\\slash");
/// ```
pub fn unescape_value(value: &str) -> String {
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

/// Parses a vendor part of the tag which follows the format `vendor/` where vendor
/// must be a valid hostname as defined in RFC 952.
///
/// A valid hostname consists of:
/// - Only alphanumeric characters, dots '.', and hyphens '-'
/// - Segments cannot start or end with a hyphen '-'
/// - Must end with a forward slash '/'
fn vendor(input: &str) -> IResult<&str, &str> {
    let hostname_chars = |c: char| c.is_alphanumeric() || c == '.' || c == '-';

    let pattern = recognize((
        alphanumeric1,
        take_till(|c: char| c == '/' || !hostname_chars(c)),
        peek(char('/')),
    ))
    .parse(input)?;

    Ok(pattern)
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

#[cfg(test)]
mod test {
    use crate::{
        escaped_value, key, key_name, parse, tag, unescape_value, validate_key_name, vendor,
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

        let input = "12345";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "12345")));

        let input = "example12345";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "example12345")));

        let input = "12345example";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "12345example")));

        let input = "12-345ex-ample";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "12-345ex-ample")));

        let input = "exam-ple";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "exam-ple")));

        let input = "-example";
        let result = key_name(input);
        assert_eq!(result, Ok(("", "-example")));
        let (_, result) = result.unwrap();
        assert!(!validate_key_name(result));

        let input = "exam_ple";
        let result = key_name(input);
        assert_eq!(result, Ok(("_ple", "exam")));
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
        assert_eq!(unescape_value("value\\:"), "value;");

        let input = "@key=a\\:b\\sc\\\\d\\re\\nf :rest";
        let (_, tags) = parse(input);
        assert_eq!(tags.get("key"), Some("a\\:b\\sc\\\\d\\re\\nf"));
        assert_eq!(unescape_value("a\\:b\\sc\\\\d\\re\\nf"), "a;b c\\d\re\nf");
    }

    #[test]
    fn test_unescape_value() {
        assert_eq!(unescape_value("hello\\sworld"), "hello world");
        assert_eq!(unescape_value("semi\\:colon"), "semi;colon");
        assert_eq!(unescape_value("back\\\\slash"), "back\\slash");
        assert_eq!(unescape_value("new\\nline"), "new\nline");
        assert_eq!(unescape_value("carriage\\rreturn"), "carriage\rreturn");
        assert_eq!(unescape_value("plain text"), "plain text");
        assert_eq!(unescape_value("trailing\\"), "trailing\\");
        assert_eq!(unescape_value("unknown\\xescape"), "unknown\\xescape");
        assert_eq!(unescape_value(""), "");
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
            tags.get_unescaped("escaped"),
            Some("a;b c\\d\re\nf".to_string())
        );
        assert_eq!(tags.get_unescaped("normal"), Some("value".to_string()));
        assert_eq!(tags.get_unescaped("missing"), None);

        let unescaped_map = tags.to_hashmap_unescaped();
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

        let owned_unescaped_map = tags.to_map_unescaped();
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
        let unescaped_map = tags.into_hashmap_unescaped();
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
        let owned_unescaped_map = tags.into_map_unescaped();
        assert_eq!(
            owned_unescaped_map.get("escaped"),
            Some(&"a;b c\\d\re\nf".to_string())
        );
        assert_eq!(
            owned_unescaped_map.get("normal"),
            Some(&"value".to_string())
        );
    }
}
