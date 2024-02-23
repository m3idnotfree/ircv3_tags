use std::collections::HashMap;

use ircv3_tags::{irc3_tags, Irc3TagsParse};

#[cfg(test)]
use pretty_assertions::assert_eq;
#[test]
fn tags_parse() {
    #[derive(irc3_tags)]
    struct S {}

    let tags ="badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type=";
    let mut expect = HashMap::new();
    expect.insert("badge-info".to_string(), "".to_string());
    expect.insert("badges".to_string(), "".to_string());
    expect.insert("color".to_string(), "#0000FF".to_string());
    expect.insert("display-name".to_string(), "barbar".to_string());
    expect.insert("emote-sets".to_string(), "0,300374282".to_string());
    expect.insert("user-id".to_string(), "713936733".to_string());
    expect.insert("user-type".to_string(), "".to_string());

    let tags = S::irc3_parse_tags(tags);
    assert_eq!(tags, Ok(("", Some(expect))));
}

#[test]
fn tags_empty() {
    #[derive(irc3_tags)]
    struct S {}

    let tags = "";
    let tags = S::irc3_parse_tags(tags);

    assert_eq!(tags, Ok(("", None)));
}
