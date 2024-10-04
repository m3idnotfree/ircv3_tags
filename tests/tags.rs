use ircv3_tags::IRCv3Tags;
use std::collections::HashMap;

#[test]
fn tags2_parse_fail_not_at_start() {
    let tags ="badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= fue";

    let (remain, result) = parse(tags, None);
    assert_eq!(remain, tags);
    assert_eq!(result, None);
}

#[test]
fn tags2_parse_vec_str_ok_at_start() {
    let tags ="@badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= .y";

    let expect = HashMap::from([
        ("badge-info", ""),
        ("badges", ""),
        ("color", "#0000FF"),
        ("display-name", "barbar"),
        ("emote-sets", "0,300374282"),
        ("user-id", "713936733"),
        ("user-type", ""),
    ])
    .into_iter()
    .map(|(a, x)| (a.to_string(), x.to_string()))
    .collect::<HashMap<String, String>>();

    // let a = IRCv3Tags(expect);

    let (remain, result) = parse(tags, None);
    let result = result.unwrap();
    assert_eq!(remain, ".y");
    assert_eq!(result, IRCv3Tags::new(expect));
    let badges = result.get("badges");
    assert_eq!(badges, Some("".to_string()));
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
    ])
    .into_iter()
    .map(|(a, x)| (a.to_string(), x.to_string()))
    .collect::<HashMap<String, String>>();

    let (remain, result) = parse(tags, None);
    let result = result.unwrap();

    assert_eq!(remain, "ue");
    assert_eq!(result, IRCv3Tags::new(expect));
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
    let expect = expect
        .into_iter()
        .map(|(a, x)| (a.to_string(), x.to_string()))
        .collect::<HashMap<String, String>>();

    let (remain, result) = parse(tags, None);
    let result = result.unwrap();

    assert_eq!(remain, "ff");
    assert_eq!(result, IRCv3Tags::new(expect));
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

    let expect = expect
        .into_iter()
        .map(|(a, x)| (a.to_string(), x.to_string()))
        .collect::<HashMap<String, String>>();

    let (remain, result) = parse(tags, None);
    let result = result.unwrap();

    assert_eq!(remain, "af");
    assert_eq!(result, IRCv3Tags::new(expect));
    // let a = result.get_mapf("badges", BadgesTag);
}

fn parse<'a>(
    tags: &'a str,
    _expect: Option<HashMap<&'a str, &'a str>>,
) -> (&'a str, Option<IRCv3Tags>) {
    ircv3_tags::parse(tags)
}
