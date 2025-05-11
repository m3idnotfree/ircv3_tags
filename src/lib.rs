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
use std::{collections::HashMap, slice::Iter};

use host::StandardHostValidator;
use nom::IResult;
use tags::{CustomTagNameValidator, IRCv3TagsParser};

pub mod host;
pub mod tags;

mod error;
mod traits;
mod unescaped_to_escaped;

pub use error::{ErrorKind, IRCv3TagsError};
pub use traits::CharValidator;
pub use unescaped_to_escaped::unescaped_to_escaped;

/// Parses only the tags portion of an IRC message, using an unwrapping fallback for errors
/// ['@' <tags> <SPACE>]
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
/// ['@' <tags> <SPACE>]
///
/// Similar to `parse`, but returns a `Result` instead of unwrapping.
///
/// # Examples
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
    // Use the default parser
    IRCv3TagsParser::default().debug_parse(input)
}

/// Create a parser allowing underscore characters in tag names
///
/// # Examples
///
/// ```
/// let input = "@tag_name=value :nick!user@host PRIVMSG #channel :Hello";
/// let parser = ircv3_tags::with_underscore();
/// let (remain, tags) = parser.parse(input);
///
/// assert_eq!(remain, ":nick!user@host PRIVMSG #channel :Hello");
/// assert_eq!(tags.get("tag_name"), Some("value"));
/// ```
pub fn with_underscore() -> IRCv3TagsParser<CustomTagNameValidator, StandardHostValidator> {
    IRCv3TagsParser::new(CustomTagNameValidator::new().allow_chars(&['_']))
}

/// Create a custom parser with flexible character validation
///
/// # Examples
///
/// ```
/// # use ircv3_tags::tags::IRCv3TagsParser;
///
/// let input = "@tag$name=value :nick!user@host PRIVMSG #channel :Hello";
/// let parser = IRCv3TagsParser::new(ircv3_tags::custom_parser().allow_chars(&['_', '$']));
/// let (remain, tags) = parser.parse(input);
///
/// assert_eq!(remain, ":nick!user@host PRIVMSG #channel :Hello");
/// assert_eq!(tags.get("tag$name"), Some("value"));
/// ```
pub fn custom_parser() -> CustomTagNameValidator {
    CustomTagNameValidator::new()
}

#[derive(Clone, Debug, PartialEq)]
pub struct IRCv3Tags<'a>(pub Vec<(&'a str, Option<&'a str>)>);

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
    pub fn to_hashmap(&'a self) -> HashMap<&'a str, &'a str> {
        self.iter().map(|(k, v)| (*k, v.unwrap_or(""))).collect()
    }

    /// Consumes the tags and converts them to a HashMap where empty values are represented as empty strings.
    pub fn into_hashmap(self) -> HashMap<&'a str, &'a str> {
        self.into_iter()
            .map(|(k, v)| (k, v.unwrap_or("")))
            .collect()
    }

    /// Converts the tags to a HashMap with escaped values.
    pub fn to_hashmap_escaped(&'a self) -> HashMap<&'a str, String> {
        self.iter()
            .map(|(k, v)| (*k, unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    /// Consumes the tags and converts them to a HashMap with escaped values.
    pub fn into_hashmap_escaped(self) -> HashMap<&'a str, String> {
        self.into_iter()
            .map(|(k, v)| (k, unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    /// Converts the tags to an owned HashMap where the keys and values are owned Strings.
    pub fn to_map(&self) -> HashMap<String, String> {
        self.iter()
            .map(|(k, v)| (k.to_string(), v.unwrap_or("").to_string()))
            .collect()
    }

    /// Consumes the tags and converts them to an owned HashMap.
    pub fn into_map(self) -> HashMap<String, String> {
        self.into_iter()
            .map(|(k, v)| (k.to_string(), v.unwrap_or("").to_string()))
            .collect()
    }

    /// Converts the tags to an owned HashMap with escaped values.
    pub fn to_map_escaped(&self) -> HashMap<String, String> {
        self.iter()
            .map(|(k, v)| (k.to_string(), unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    /// Consumes the tags and converts them to an owned HashMap with escaped values.
    pub fn into_map_escaped(self) -> HashMap<String, String> {
        self.into_iter()
            .map(|(k, v)| (k.to_string(), unescaped_to_escaped(v.unwrap_or(""))))
            .collect()
    }

    pub fn iter(&'a self) -> Iter<'a, (&'a str, Option<&'a str>)> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for IRCv3Tags<'a> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = (&'a str, Option<&'a str>);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::fmt::Display for IRCv3Tags<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.iter().peekable();
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
