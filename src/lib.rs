use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until1},
    multi::separated_list1,
    IResult,
};

pub use irc3_tags_derive::irc3_tags;

pub trait Irc3TagsParse {
    fn irc3_parse_tags(tags: &str) -> IResult<&str, Option<HashMap<String, String>>> {
        fn parse_t(msg: &str) -> IResult<&str, Option<HashMap<String, String>>> {
            println!("parse t: {}", msg);
            if msg.is_empty() {
                Ok(("", None))
            } else {
                let (msg, kv_pairs) = separated_list1(tag(";"), parse_key_values)(msg)?;

                Ok((
                    msg,
                    Some(
                        kv_pairs
                            .into_iter()
                            .map(|(k, v)| (k.to_owned().to_string(), v.to_owned().to_string()))
                            .collect::<HashMap<String, String>>(),
                    ),
                ))
            }
        }
        fn parse_key_values(msg: &str) -> IResult<&str, (&str, &str)> {
            let (msg, key) = take_until1("=")(msg)?;
            let (msg, _) = tag("=")(msg)?;
            let (msg, value) = take_till(|c| c == ' ' || c == ';')(msg)?;

            Ok((msg, (key, value)))
        }
        parse_t(tags)
    }
}
