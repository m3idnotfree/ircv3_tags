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
    data: (&'a str, Option<Vec<(&'a str, &'a str)>>),
}

impl<'a> Ircv3TagsParse<'a> {
    pub fn new(msg: &'a str) -> Ircv3TagsParse {
        let result = Ircv3TagsParse::irc3_tags(msg).unwrap();
        Ircv3TagsParse { data: result }
    }
    pub fn vec_str(self) -> (&'a str, Option<Vec<(&'a str, &'a str)>>) {
        (self.data.0, self.data.1)
    }

    pub fn vec_string(self) -> (&'a str, Option<Vec<(String, String)>>) {
        match self.data.1 {
            Some(k_v) => (
                self.data.0,
                Some(
                    k_v.into_iter()
                        .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                        .collect::<Vec<(String, String)>>(),
                ),
            ),
            None => (self.data.0, None),
        }
    }

    pub fn hashmap_string(self) -> (&'a str, Option<HashMap<String, String>>) {
        match self.data.1 {
            Some(k_v) => (
                self.data.0,
                Some(
                    k_v.into_iter()
                        .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                        .collect::<HashMap<String, String>>(),
                ),
            ),
            None => (self.data.0, None),
        }
    }

    pub fn hashmap_str(self) -> (&'a str, Option<HashMap<&'a str, &'a str>>) {
        match self.data.1 {
            Some(k_v) => (
                self.data.0,
                Some(k_v.into_iter().collect::<HashMap<&str, &str>>()),
            ),
            None => (self.data.0, None),
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
