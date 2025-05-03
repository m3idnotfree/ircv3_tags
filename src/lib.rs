//! only parse IRCv3 tags part
//! # Examples
//!
//! ```
//! use ircv3_tags::{IRCv3Tags, parse};
//! use std::collections::HashMap;
//!
//! let msg = "@badge-info=;badges=broadcaster/1;client-nonce=997dcf443c31e258c1d32a8da47b6936;color=#0000FF;display-name=abc;emotes=;first-msg=0;flags=0-6:S.7;id=eb24e920-8065-492a-8aea-266a00fc5126;mod=0;room-id=713936733;subscriber=0;tmi-sent-ts=1642786203573;turbo=0;user-id=713936733;user-type= :abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys";
//! let (remain, tags) = parse(msg);
//! let expected_tags = HashMap::from([
//!    ("badge-info", ""),
//!    ("subscriber", "0"),
//!    ("id", "eb24e920-8065-492a-8aea-266a00fc5126"),
//!    ("user-id", "713936733"),
//!    ("emotes", ""),
//!    ("tmi-sent-ts", "1642786203573"),
//!    ("client-nonce", "997dcf443c31e258c1d32a8da47b6936"),
//!    ("mod", "0"),
//!    ("badges", "broadcaster/1"),
//!    ("room-id", "713936733"),
//!    ("flags", "0-6:S.7"),
//!    ("color", "#0000FF"),
//!    ("turbo", "0"),
//!    ("display-name", "abc"),
//!    ("first-msg", "0"),
//!    ("user-type", ""),
//! ]).into_iter()
//!   .map(|(key, value)| (key.to_string(), value.to_string()))
//!   .collect::<HashMap<String, String>>();
//!
//! assert_eq!(remain, ":abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys");
//!
//! assert!(tags.is_some());
//! let tags = tags.unwrap();
//! assert_eq!(tags, IRCv3Tags::new(expected_tags));
//!
//! let tmi_sent_ts = tags.get("tmi-sent-ts");
//! assert_eq!(tmi_sent_ts, Some(&"1642786203573".to_string()));
//!
//! let notif = tags.get("not-if");
//! assert_eq!(notif, None);
//! ```
use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until1},
    character::complete::space1,
    combinator::opt,
    multi::separated_list1,
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

pub fn parse(msg: &str) -> (&str, Option<IRCv3Tags>) {
    let (remain, data) = irc3_tags_parse(msg).unwrap();
    (remain, data.map(|x| IRCv3Tags(from_hash_string(x))))
}

pub fn parse_nom(msg: &str) -> IResult<&str, Option<IRCv3Tags>> {
    let (remain, data) = irc3_tags_parse(msg)?;
    Ok((remain, data.map(|x| IRCv3Tags(from_hash_string(x)))))
}

#[derive(Clone, Debug, PartialEq)]
pub struct IRCv3Tags(HashMap<String, String>);

impl IRCv3Tags {
    pub fn new(tags: HashMap<String, String>) -> Self {
        Self(tags)
    }

    pub fn get(&self, tag: &str) -> Option<&String> {
        self.0.get(tag)
    }
}

/// (remain, (key, value)*)
fn irc3_tags_parse(msg: &str) -> IResult<&str, Option<Vec<(&str, &str)>>> {
    opt(delimited(
        tag("@"),
        separated_list1(tag(";"), ircv3_tags_key_value),
        space1,
    ))
    .parse(msg)
}

/// (remain, (key, value))
fn ircv3_tags_key_value(msg: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(
        take_until1("="),
        tag("="),
        take_till(|c| c == ' ' || c == ';'),
    )
    .parse(msg)
}

fn from_hash_string(data: Vec<(&str, &str)>) -> HashMap<String, String> {
    data.into_iter()
        .map(|row| (row.0.to_string(), row.1.to_string()))
        .collect::<HashMap<String, String>>()
}
