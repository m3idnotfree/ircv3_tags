//! only parse IRCv3 tags part
//! # Examples
//!
//! ```
//! use ircv3_tags::Ircv3TagsParse;
//! use std::collections::HashMap;
//!
//! let msg = "@badge-info=;badges=broadcaster/1;client-nonce=997dcf443c31e258c1d32a8da47b6936;color=#0000FF;display-name=abc;emotes=;first-msg=0;flags=0-6:S.7;id=eb24e920-8065-492a-8aea-266a00fc5126;mod=0;room-id=713936733;subscriber=0;tmi-sent-ts=1642786203573;turbo=0;user-id=713936733;user-type= :abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys";
//! let tags = Ircv3TagsParse::new(msg);
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
//! assert_eq!(tags.remain, ":abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys");
//! assert_eq!(tags.to_hashmap_str(),Some(expected_tags));
//! ```
use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until1},
    combinator::opt,
    multi::separated_list1,
    sequence::{delimited, separated_pair},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Ircv3TagsParse<'a> {
    data: Option<Vec<(&'a str, &'a str)>>,
    pub remain: &'a str,
}

impl<'a> Ircv3TagsParse<'a> {
    pub fn new(msg: &'a str) -> Ircv3TagsParse {
        let (remain, data) = Ircv3TagsParse::irc3_tags_parse(msg).unwrap();
        Ircv3TagsParse { data, remain }
    }

    pub fn to_vec_str(self) -> Option<Vec<(&'a str, &'a str)>> {
        self.data
    }

    pub fn to_vec_string(self) -> Option<Vec<(String, String)>> {
        self.data.map(|k_v| {
            k_v.into_iter()
                .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                .collect::<Vec<(String, String)>>()
        })
    }

    pub fn to_hashmap_string(self) -> Option<HashMap<String, String>> {
        self.data.map(|k_v| {
            k_v.into_iter()
                .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                .collect::<HashMap<String, String>>()
        })
    }

    pub fn to_hashmap_str(self) -> Option<HashMap<&'a str, &'a str>> {
        self.data
            .map(|k_v| k_v.into_iter().collect::<HashMap<&str, &str>>())
    }

    /// (remain, (key, value)*)
    pub fn irc3_tags_parse(msg: &str) -> IResult<&str, Option<Vec<(&str, &str)>>> {
        opt(delimited(
            tag("@"),
            separated_list1(tag(";"), Ircv3TagsParse::ircv3_tags_key_value),
            tag(" "),
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
}
