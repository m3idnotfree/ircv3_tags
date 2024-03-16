//! only parse IRCv3 tags part
//! # Examples
//!
//! ```
//! use ircv3_tags::IRCv3Tags;
//! use std::collections::HashMap;
//!
//! let msg = "@badge-info=;badges=broadcaster/1;client-nonce=997dcf443c31e258c1d32a8da47b6936;color=#0000FF;display-name=abc;emotes=;first-msg=0;flags=0-6:S.7;id=eb24e920-8065-492a-8aea-266a00fc5126;mod=0;room-id=713936733;subscriber=0;tmi-sent-ts=1642786203573;turbo=0;user-id=713936733;user-type= :abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys";
//! let (remain, tags) = IRCv3Tags::parse(msg).unwrap();
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
//!]);
//! assert_eq!(remain, ":abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys");
//! assert_eq!(tags.as_ref(), &Some(expected_tags));
//!
//! let tmi_sent_ts = tags.get("tmi-sent-ts");
//! assert_eq!(tmi_sent_ts, Some("1642786203573"));
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
    IResult,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IRCv3Tags<'a>(Option<HashMap<&'a str, &'a str>>);

impl<'a> IRCv3Tags<'a> {
    pub fn parse(msg: &str) -> IResult<&str, IRCv3Tags> {
        let (remain, data) = IRCv3Tags::irc3_tags_parse(msg)?;
        let result = IRCv3Tags::to_hashmap_str(data);
        Ok((remain, IRCv3Tags(result)))
    }

    pub fn get(&self, tag: &str) -> Option<&str> {
        self.0.as_ref().and_then(|value| value.get(tag).copied())
    }

    /// (remain, (key, value)*)
    fn irc3_tags_parse(msg: &str) -> IResult<&str, Option<Vec<(&str, &str)>>> {
        opt(delimited(
            tag("@"),
            separated_list1(tag(";"), IRCv3Tags::ircv3_tags_key_value),
            space1,
        ))(msg)
    }

    /// (remain, (key, value))
    fn ircv3_tags_key_value(msg: &str) -> IResult<&str, (&str, &str)> {
        separated_pair(
            take_until1("="),
            tag("="),
            take_till(|c| c == ' ' || c == ';'),
        )(msg)
    }

    fn to_hashmap_str(data: Option<Vec<(&'a str, &'a str)>>) -> Option<HashMap<&str, &str>> {
        // self.data
        data.map(|k_v| k_v.into_iter().collect::<HashMap<&str, &str>>())
    }
}

impl<'a> AsRef<Option<HashMap<&'a str, &'a str>>> for IRCv3Tags<'a> {
    fn as_ref(&self) -> &Option<HashMap<&'a str, &'a str>> {
        &self.0
    }
}
