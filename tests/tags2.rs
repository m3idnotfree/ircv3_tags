use irc3_tags::{irc3_tags, Irc3TagsParse, Ircv3TagsParse};
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
fn tags2_parse_fail_not_at_start() {
    #[derive(irc3_tags)]
    struct S {}

    let tags ="badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ";

    let (msg, result) = Ircv3TagsParse::new(tags).vec_string();
    assert_eq!(msg, tags);
    assert_eq!(result, None);
}

#[test]
fn tags2_parse_vec_str_ok_at_start() {
    #[derive(irc3_tags)]
    struct S {}

    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ";
    let expect = Some(vec![
        ("badge-info", ""),
        ("badges", ""),
        ("color", "#0000FF"),
        ("display-name", "barbar"),
        ("emote-sets", "0,300374282"),
        ("user-id", "713936733"),
        ("user-type", ""),
    ]);
    let (msg, result) = Ircv3TagsParse::new(tags).vec_str();
    assert_eq!(msg, " ");
    assert_eq!(result, expect);
}

#[test]
fn tags2_parse_vec_string_ok_at_start() {
    #[derive(irc3_tags)]
    struct S {}

    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ";
    let expect = Some(vec![
        ("badge-info".to_string(), "".to_string()),
        ("badges".to_string(), "".to_string()),
        ("color".to_string(), "#0000FF".to_string()),
        ("display-name".to_string(), "barbar".to_string()),
        ("emote-sets".to_string(), "0,300374282".to_string()),
        ("user-id".to_string(), "713936733".to_string()),
        ("user-type".to_string(), "".to_string()),
    ]);
    let (msg, result) = Ircv3TagsParse::new(tags).vec_string();
    assert_eq!(msg, " ");
    assert_eq!(result, expect);
}
#[test]
fn tags2_parse_hashmap_str_ok_at_start() {
    #[derive(irc3_tags)]
    struct S {}

    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ";
    let mut expect = HashMap::new();
    expect.insert("badge-info", "");
    expect.insert("badges", "");
    expect.insert("color", "#0000FF");
    expect.insert("display-name", "barbar");
    expect.insert("emote-sets", "0,300374282");
    expect.insert("user-id", "713936733");
    expect.insert("user-type", "");
    let (msg, result) = Ircv3TagsParse::new(tags).hashmap_str();
    assert_eq!(msg, " ");
    assert_eq!(result, Some(expect));
}

#[test]
fn tags2_parse_hashmap_string_ok_at_start() {
    #[derive(irc3_tags)]
    struct S {}

    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ";
    let mut expect = HashMap::new();
    expect.insert("badge-info".to_string(), "".to_string());
    expect.insert("badges".to_string(), "".to_string());
    expect.insert("color".to_string(), "#0000FF".to_string());
    expect.insert("display-name".to_string(), "barbar".to_string());
    expect.insert("emote-sets".to_string(), "0,300374282".to_string());
    expect.insert("user-id".to_string(), "713936733".to_string());
    expect.insert("user-type".to_string(), "".to_string());

    let (msg, result) = Ircv3TagsParse::new(tags).hashmap_string();
    assert_eq!(msg, " ");
    assert_eq!(result, Some(expect));
}
