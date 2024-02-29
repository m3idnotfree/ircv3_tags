//! only parse IRCv3 tags part
//! # Examples
//!
//! ```no_run
//! let msg = "@badge-info=;badges=broadcaster/1;client-nonce=997dcf443c31e258c1d32a8da47b6936;color=#0000FF;display-name=abc;emotes=;first-msg=0;flags=0-6:S.7;id=eb24e920-8065-492a-8aea-266a00fc5126;mod=0;room-id=713936733;subscriber=0;tmi-sent-ts=1642786203573;turbo=0;user-id=713936733;user-type= :abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys";
//! let tags = Ircv3TagsParse::new(msg);
//! assert_eq!(tags.msg, ":abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys");
//! ```
//! select tags return
//! - vec_str
//! - vec_string
//! - hashmap_str
//! - hashmap_string
use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until1},
    combinator::opt,
    multi::separated_list1,
    sequence::{delimited, separated_pair},
    IResult,
};

#[derive(Debug)]
pub struct Ircv3TagsParse<'a> {
    data: Option<Vec<(&'a str, &'a str)>>,
    pub msg: &'a str,
}

impl<'a> Ircv3TagsParse<'a> {
    pub fn new(msg: &'a str) -> Ircv3TagsParse {
        let (msg, data) = Ircv3TagsParse::irc3_tags(msg).unwrap();
        Ircv3TagsParse { data, msg }
    }
    pub fn vec_str(self) -> Option<Vec<(&'a str, &'a str)>> {
        self.data
    }

    pub fn vec_string(self) -> Option<Vec<(String, String)>> {
        match self.data {
            Some(k_v) => Some(
                k_v.into_iter()
                    .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                    .collect::<Vec<(String, String)>>(),
            ),
            None => None,
        }
    }

    pub fn hashmap_string(self) -> Option<HashMap<String, String>> {
        match self.data {
            Some(k_v) => Some(
                k_v.into_iter()
                    .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                    .collect::<HashMap<String, String>>(),
            ),
            None => None,
        }
    }

    pub fn hashmap_str(self) -> Option<HashMap<&'a str, &'a str>> {
        match self.data {
            Some(k_v) => Some(k_v.into_iter().collect::<HashMap<&str, &str>>()),
            None => None,
        }
    }

    fn irc3_tags(msg: &str) -> IResult<&str, Option<Vec<(&str, &str)>>> {
        opt(delimited(
            tag("@"),
            separated_list1(tag(";"), Ircv3TagsParse::ircv3_tags_key_values),
            tag(" "),
        ))(msg)
    }

    fn ircv3_tags_key_values(msg: &str) -> IResult<&str, (&str, &str)> {
        separated_pair(
            take_until1("="),
            tag("="),
            take_till(|c| c == ' ' || c == ';'),
        )(msg)
    }
}
