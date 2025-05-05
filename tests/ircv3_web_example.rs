#![allow(non_snake_case)]

#[test]
fn A_message_with_0_tags() {
    let input = ":nick!ident@h#[warn(non_snake_case)]ost.com PRIVMSG me :Hello";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_err());
}

#[test]
fn A_message_sent_by_the_server_with_3_tags() {
    let input = "@aaa=bbb;ccc;example.com/ddd=eee :nick!ident@host.com PRIVMSG me :Hello";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();

    assert_eq!(remain, ":nick!ident@host.com PRIVMSG me :Hello");
    assert_eq!(result.get("aaa"), Some("bbb"));
    assert_eq!(result.get("ccc"), Some(""));
    assert_eq!(result.get("example.com/ddd"), Some("eee"));
}

#[test]
fn A_message_sent_by_a_client_with_the_example_tag_tag() {
    let input = "@example-tag=example-value PRIVMSG #channel :Message";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, "PRIVMSG #channel :Message");
    assert_eq!(result.get("example-tag"), Some("example-value"));
}
// assert_eq!(result.get(""),Some(""));
#[test]
fn A_message_sent_by_a_client_with_the__example_client_tag_client_only_tag() {
    let input = "@+example-client-tag=example-value PRIVMSG #channel :Message";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, "PRIVMSG #channel :Message");
    assert_eq!(result.get("+example-client-tag"), Some("example-value"));
}

#[test]
fn The_server_sends_these_messages() {
    let input = "@+icon=https://example.com/favicon.png :url_bot!bot@example.com PRIVMSG #channel :Example.com: A News Story";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();

    assert_eq!(
        remain,
        ":url_bot!bot@example.com PRIVMSG #channel :Example.com: A News Story"
    );
    assert_eq!(result.get("+icon"), Some("https://example.com/favicon.png"));
}
#[test]
fn An_example_of_a_vendor_prefixed_client_only_tag() {
    let input = "@+example.com/foo=bar :irc.example.com NOTICE #channel :A vendor-prefixed client-only tagged message";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":irc.example.com NOTICE #channel :A vendor-prefixed client-only tagged message"
    );
    assert_eq!(result.get("+example.com/foo"), Some("bar"));
}

#[test]
fn A_client_only_tag__example_with_a_value_containing_valid_raw_and_escaped_characters() {
    let input = r"@+example=raw+:=,escaped\:\s\\ :irc.example.com NOTICE #channel :Message";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();

    assert_eq!(remain, ":irc.example.com NOTICE #channel :Message");
    assert_eq!(result.get("+example"), Some(r"raw+:=,escaped\:\s\\"));
    assert_eq!(
        result.get_unescaped("+example"),
        Some("raw+:=,escaped; \\".to_string())
    );
}

#[test]
fn A_TAGMSG_sent_by_a_client_with_tags_that_exceed_the_size_limit() {
    let input = "@+tag1;+tag2;+tag5000 TAGMSG #channel";
    let result = ircv3_tags::try_parse(input);

    assert!(result.is_ok());
    let (remain, result) = result.unwrap();

    assert_eq!(remain, "TAGMSG #channel");
    assert_eq!(result.get("+tag1"), Some(""));
    assert_eq!(result.get("+tag2"), Some(""));
    assert_eq!(result.get("+tag5000"), Some(""));
}
