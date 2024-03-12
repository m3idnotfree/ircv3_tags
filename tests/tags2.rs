use ircv3_tags::{IRCv3Tags, Ircv3TagsParse};
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
fn tags2_parse_fail_not_at_start() {
    let tags ="badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= fue";

    let (remain, result) = Ircv3TagsParse::parse(tags).unwrap();
    assert_eq!(remain, tags);
    assert_eq!(result, IRCv3Tags::new(None));
}

#[test]
fn tags2_parse_vec_str_ok_at_start() {
    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= .y";
    // let expect = Some(vec![
    let expect = HashMap::from([
        ("badge-info", ""),
        ("badges", ""),
        ("color", "#0000FF"),
        ("display-name", "barbar"),
        ("emote-sets", "0,300374282"),
        ("user-id", "713936733"),
        ("user-type", ""),
    ]);
    let (remain, result) = Ircv3TagsParse::parse(tags).unwrap();
    assert_eq!(remain, ".y");
    assert_eq!(result, IRCv3Tags::new(Some(expect)));
    let badges = result.get("badges");
    assert_eq!(badges, Some(""));
    let user = result.get("user");
    assert_eq!(user, None);
}

#[test]
fn tags2_parse_vec_string_ok_at_start() {
    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ue";
    // let expect = Some(vec![
    let expect = HashMap::from([
        ("badge-info", ""),
        ("badges", ""),
        ("color", "#0000FF"),
        ("display-name", "barbar"),
        ("emote-sets", "0,300374282"),
        ("user-id", "713936733"),
        ("user-type", ""),
    ]);
    let (remain, result) = Ircv3TagsParse::parse(tags).unwrap();
    assert_eq!(remain, "ue");
    assert_eq!(result, IRCv3Tags::new(Some(expect)));
}

#[test]
fn tags2_parse_hashmap_str_ok_at_start() {
    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= ff";
    let mut expect = HashMap::new();
    expect.insert("badge-info", "");
    expect.insert("badges", "");
    expect.insert("color", "#0000FF");
    expect.insert("display-name", "barbar");
    expect.insert("emote-sets", "0,300374282");
    expect.insert("user-id", "713936733");
    expect.insert("user-type", "");
    let (remain, result) = Ircv3TagsParse::parse(tags).unwrap();
    assert_eq!(remain, "ff");
    assert_eq!(result, IRCv3Tags::new(Some(expect)));
}

#[test]
fn tags2_parse_hashmap_string_ok_at_start() {
    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= af";
    let mut expect = HashMap::new();
    expect.insert("badge-info", "");
    expect.insert("badges", "");
    expect.insert("color", "#0000FF");
    expect.insert("display-name", "barbar");
    expect.insert("emote-sets", "0,300374282");
    expect.insert("user-id", "713936733");
    expect.insert("user-type", "");

    let (remain, result) = Ircv3TagsParse::parse(tags).unwrap();
    assert_eq!(remain, "af");
    assert_eq!(result, IRCv3Tags::new(Some(expect)));
}
