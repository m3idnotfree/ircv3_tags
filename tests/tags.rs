#[test]
fn basic_tags() {
    let input = "@id=123456789;time=2025-05-04T12:34:56Z;msgid=abc123 :nick!user@host.com PRIVMSG #channel :Hello,
   world!";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :Hello,
   world!"
    );
    assert_eq!(result.get("id"), Some("123456789"));
    assert_eq!(result.get("time"), Some("2025-05-04T12:34:56Z"));
    assert_eq!(result.get("msgid"), Some("abc123"));
}

#[test]
fn vendor_specific() {
    let input = "@badge-info=subscriber/12;badges=subscriber/12,premium/1;tmi.twitch.tv/emote-only=1;room-id=12345;user-id=67890 :nick!user@host.com PRIVMSG #channel :GlitchCat";

    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();

    assert_eq!(remain, ":nick!user@host.com PRIVMSG #channel :GlitchCat");
    assert_eq!(result.get("badge-info"), Some("subscriber/12"));
    assert_eq!(result.get("badges"), Some("subscriber/12,premium/1"));
    assert_eq!(result.get("tmi.twitch.tv/emote-only"), Some("1"));
    assert_eq!(result.get("room-id"), Some("12345"));
    assert_eq!(result.get("user-id"), Some("67890"));

    let input =
        "@discord.com/server-id=12345;discord.com/channel-id=67890;discord.com/message-type=regular :nick!user@host.com PRIVMSG #general :Hello everyone!";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #general :Hello everyone!"
    );
    assert_eq!(result.get("discord.com/server-id"), Some("12345"));
    assert_eq!(result.get("discord.com/channel-id"), Some("67890"));
    assert_eq!(result.get("discord.com/message-type"), Some("regular"));

    let input = "@github.com/repo=ircv3_tags;gitlab.org/issue=123;bitbucket.org/pr=456 :nick!user@host.com PRIVMSG #dev :Fixed that bug!";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, ":nick!user@host.com PRIVMSG #dev :Fixed that bug!");
    assert_eq!(result.get("github.com/repo"), Some("ircv3_tags"));
    assert_eq!(result.get("gitlab.org/issue"), Some("123"));
    assert_eq!(result.get("bitbucket.org/pr"), Some("456"));

    let input = "@+draft/reply=123456789;+typing=active;server.com/seen=2025-05-04T12:30:00Z :nick!user@host.com PRIVMSG #channel :I'm replying to your message";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :I'm replying to your message"
    );
    assert_eq!(result.get("+draft/reply"), Some("123456789"));
    assert_eq!(result.get("+typing"), Some("active"));
    assert_eq!(result.get("server.com/seen"), Some("2025-05-04T12:30:00Z"));
}

#[test]
fn escaped_characters() {
    let input = r"@display-name=John\sDoe;message=Hello\sWorld :nick!user@host.com PRIVMSG #channel :This has escaped spaces in tags";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :This has escaped spaces in tags"
    );
    assert_eq!(result.get("display-name"), Some(r"John\sDoe"));
    assert_eq!(result.get("message"), Some(r"Hello\sWorld"));
    // Using a simpler case with properly escaped semicolon
    let input = r"@id=123456789;css-style=color\:#ff0000 :nick!user@host.com PRIVMSG #channel :This has escaped semicolons in tags";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :This has escaped semicolons in tags"
    );
    assert_eq!(result.get("id"), Some("123456789"));
    assert_eq!(result.get("css-style"), Some(r"color\:#ff0000"));
    //
    let input = r"@file-path=C\\:\\\\Users\\\\Name\\\\Documents;command=echo\s\\\\ :nick!user@host.com PRIVMSG #channel :This has escaped backslashes in tags";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :This has escaped backslashes in tags"
    );
    assert_eq!(
        result.get("file-path"),
        Some(r"C\\:\\\\Users\\\\Name\\\\Documents")
    );
    assert_eq!(result.get("command"), Some(r"echo\s\\\\"));

    let input = r"@multi-line=First\sLine\nSecond\sLine;formatted=Title\r\nBody :nick!user@host.com PRIVMSG #channel :This has escaped newlines in tags";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :This has escaped newlines in tags"
    );
    assert_eq!(result.get("multi-line"), Some(r"First\sLine\nSecond\sLine"));
    assert_eq!(result.get("formatted"), Some(r"Title\r\nBody"));

    let input = r"@mixed-escapes=Value\swith\sspace\sand\:\ssemicolon\sand\\backslash\r\nand\snewlines :nick!user@host.com PRIVMSG #channel :This has mixed escaped characters";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(
        remain,
        ":nick!user@host.com PRIVMSG #channel :This has mixed escaped characters"
    );
    assert_eq!(
        result.get("mixed-escapes"),
        Some(r"Value\swith\sspace\sand\:\ssemicolon\sand\\backslash\r\nand\snewlines")
    );
}

// #[test]
// fn combine_multiple() {
//     // Not allow key_name '\', ':'
//     let input = r"@id=123456789;twitch.tv/badges=subscriber/12,premium/1;display-name=John\sDoe;css-style=color\:#ff0000\;font-weight\:bold;+client/typing=active :nick!user@host.com PRIVMSG #channel :This combines multiple tag types";
//     let result = ircv3_tags::try_parse(input);
//     assert!(result.is_err());
//
//     let result = ircv3_tags::try_parse_extend(input);
//     assert!(result.is_ok());
//     let (remain, result) = result.unwrap();
//     assert_eq!(
//         remain,
//         ":nick!user@host.com PRIVMSG #channel :This combines multiple tag types"
//     );
//     assert_eq!(result.get("id"), Some("123456789"));
//     assert_eq!(
//         result.get("twitch.tv/badges"),
//         Some("subscriber/12,premium/1")
//     );
//     assert_eq!(result.get("display-name"), Some(r"John\sDoe"));
//     assert_eq!(result.get("css-style"), Some(r"color\:#ff0000\"));
//     // Not allow key_name '\:'
//     assert_eq!(result.get(r"font-weight\:bold"), Some(""));
//     assert_eq!(result.get("+client/typing"), Some("active"));
//
//     let input =
//         "@badges=;github.com/repo=ircv3_tags;file-path=C:\\Program\\Files;room-id=123;user-id=456 :nick!user@host.com PRIVMSG #channel :Combined tag types with some empty values";
//     let result = ircv3_tags::try_parse(input);
//     assert!(result.is_ok());
//     let (remain, result) = result.unwrap();
//     assert_eq!(
//         remain,
//         ":nick!user@host.com PRIVMSG #channel :Combined tag types with some empty values"
//     );
//     assert_eq!(result.get("badges"), Some(""));
//     assert_eq!(result.get("github.com/repo"), Some("ircv3_tags"));
//     assert_eq!(result.get("file-path"), Some("C:\\Program\\Files"));
//     assert_eq!(result.get("room-id"), Some("123"));
//     assert_eq!(result.get("user-id"), Some("456"));
//
//     let input = r"@id=abc123;twitch.tv/emote-only=1;discord.com/reply-to=789123;comment=This\sis\sa\slong\sstring\\with\:\ssemicolons\nand\rnewlines;+draft/marked=important :nick!user@host.com PRIVMSG #channel :Complex message with all tag types";
//
//     let result = ircv3_tags::try_parse(input);
//     assert!(result.is_ok());
//     let (remain, result) = result.unwrap();
//     assert_eq!(
//         remain,
//         ":nick!user@host.com PRIVMSG #channel :Complex message with all tag types"
//     );
//     assert_eq!(result.get("id"), Some("abc123"));
//     assert_eq!(result.get("twitch.tv/emote-only"), Some("1"));
//     assert_eq!(result.get("discord.com/reply-to"), Some("789123"));
//     assert_eq!(
//         result.get("comment"),
//         Some(r"This\sis\sa\slong\sstring\\with\:\ssemicolons\nand\rnewlines")
//     );
//     assert_eq!(result.get("+draft/marked"), Some("important"));
//
//     let input = r"@id=123456789;invalid-escape=This\bshould\qdrop\invalid\chars;trailing-backslash=test\ :nick!user@host.com PRIVMSG #channel :Tests edge cases in escaping";
//     let result = ircv3_tags::try_parse(input);
//     assert!(result.is_ok());
//     let (remain, result) = result.unwrap();
//     assert_eq!(
//         remain,
//         ":nick!user@host.com PRIVMSG #channel :Tests edge cases in escaping"
//     );
//     assert_eq!(result.get("id"), Some("123456789"));
//     assert_eq!(
//         result.get("invalid-escape"),
//         Some(r"This\bshould\qdrop\invalid\chars")
//     );
//     assert_eq!(result.get("trailing-backslash"), Some(r"test\"));
//
//     let input = r"@server.com/id=123456;+client/highlight=true;empty=;complex-value=Multi\sline\nvalue\swith\:\ssemicolon\sand\\\backslash;twitch.tv/badges=moderator/1,subscriber/24;timestamp=2025-05-04T12\:34\:56Z :nick!user@host.com PRIVMSG #channel :This message has everything";
//     let result = ircv3_tags::try_parse(input);
//     assert!(result.is_ok());
//     let (remain, result) = result.unwrap();
//     assert_eq!(
//         remain,
//         ":nick!user@host.com PRIVMSG #channel :This message has everything"
//     );
//     assert_eq!(result.get("server.com/id"), Some("123456"));
//     assert_eq!(result.get("+client/highlight"), Some("true"));
//     assert_eq!(result.get("empty"), Some(""));
//     assert_eq!(
//         result.get("complex-value"),
//         Some(r"Multi\sline\nvalue\swith\:\ssemicolon\sand\\\backslash")
//     );
//     assert_eq!(
//         result.get("twitch.tv/badges"),
//         Some("moderator/1,subscriber/24")
//     );
//     assert_eq!(result.get("timestamp"), Some(r"2025-05-04T12\:34\:56Z"));
// }

#[test]
fn diffrent_tag_formats() {
    let input = "@badge-info=subscriber/8;badges=subscriber/6,premium/1;color=#0000FF;display-name=TwitchUser123;emotes=25:0-4,12-16;flags=;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=12345678;subscriber=1;tmi-sent-ts=1612312345678;turbo=0;user-id=87654321;user-type= :twitchuser123!twitchuser123@twitchuser123.tmi.twitch.tv PRIVMSG #channelname :Kappa This is a Kappa message";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain,":twitchuser123!twitchuser123@twitchuser123.tmi.twitch.tv PRIVMSG #channelname :Kappa This is a Kappa message");
    assert_eq!(result.get("badge-info"), Some("subscriber/8"));
    assert_eq!(result.get("badges"), Some("subscriber/6,premium/1"));
    assert_eq!(result.get("color"), Some("#0000FF"));
    assert_eq!(result.get("display-name"), Some("TwitchUser123"));
    assert_eq!(result.get("emotes"), Some("25:0-4,12-16"));
    assert_eq!(result.get("flags"), Some(""));
    assert_eq!(
        result.get("id"),
        Some("b34ccfc7-4977-403a-8a94-33c6bac34fb8")
    );
    assert_eq!(result.get("mod"), Some("0"));
    assert_eq!(result.get("room-id"), Some("12345678"));
    assert_eq!(result.get("subscriber"), Some("1"));
    assert_eq!(result.get("tmi-sent-ts"), Some("1612312345678"));
    assert_eq!(result.get("turbo"), Some("0"));
    assert_eq!(result.get("user-id"), Some("87654321"));
    assert_eq!(result.get("user-type"), Some(""));

    let input = "@+draft/reply=123456789;msgid=matrix-$1612312345678abcdef:matrix.org;time=2023-05-10T15:23:45.678Z :matrixuser@matrix.org PRIVMSG #matrix-bridged-channel :This message was sent from Matrix and bridged to IRC";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, ":matrixuser@matrix.org PRIVMSG #matrix-bridged-channel :This message was sent from Matrix and bridged to IRC");
    assert_eq!(result.get("+draft/reply"), Some("123456789"));
    assert_eq!(
        result.get("msgid"),
        Some("matrix-$1612312345678abcdef:matrix.org")
    );
    assert_eq!(result.get("time"), Some("2023-05-10T15:23:45.678Z"));

    let input = r"@badge=admin/1,founder/1;color=#FF0000;display-name=GameMaster;game-rank=100;guild=Defenders\sof\sLight;id=game-123456789;server=us-west;user-id=12345 :gamemaster@gamechat.example.com PRIVMSG #global-chat :Server maintenance in 10 minutes, please finish your quests";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, ":gamemaster@gamechat.example.com PRIVMSG #global-chat :Server maintenance in 10 minutes, please finish your quests");
    assert_eq!(result.get("badge"), Some("admin/1,founder/1"));
    assert_eq!(result.get("color"), Some("#FF0000"));
    assert_eq!(result.get("display-name"), Some("GameMaster"));
    assert_eq!(result.get("game-rank"), Some("100"));
    assert_eq!(result.get("guild"), Some(r"Defenders\sof\sLight"));
    assert_eq!(result.get("id"), Some("game-123456789"));
    assert_eq!(result.get("server"), Some("us-west"));
    assert_eq!(result.get("user-id"), Some("12345"));

    let input = "@account=contributor;time=2025-03-15T14:30:45.123Z;msgid=libera-123456789;+draft/label=commit-notification :gitbot!gitbot@services.libera.chat PRIVMSG #project-dev :New commit by developer: \"Fix memory leak in parser module - Issue #4567\" - https://github.com/project/repo/commit/a1b2c3d4";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, ":gitbot!gitbot@services.libera.chat PRIVMSG #project-dev :New commit by developer: \"Fix memory leak in parser module - Issue #4567\" - https://github.com/project/repo/commit/a1b2c3d4");
    assert_eq!(result.get("account"), Some("contributor"));
    assert_eq!(result.get("time"), Some("2025-03-15T14:30:45.123Z"));
    assert_eq!(result.get("msgid"), Some("libera-123456789"));
    assert_eq!(result.get("+draft/label"), Some("commit-notification"));

    let input = "@id=discord-123456789;discord-role=admin,moderator;avatar=https://cdn.discordapp.com/avatars/123456789/abcdef.png;time=2025-03-16T08:12:32.456Z :discorduser#1234@discord.gateway PRIVMSG #general :@everyone Important announcement about the new server rules";
    let result = ircv3_tags::try_parse(input);
    assert!(result.is_ok());
    let (remain, result) = result.unwrap();
    assert_eq!(remain, ":discorduser#1234@discord.gateway PRIVMSG #general :@everyone Important announcement about the new server rules");
    assert_eq!(result.get("id"), Some("discord-123456789"));
    assert_eq!(result.get("discord-role"), Some("admin,moderator"));
    assert_eq!(
        result.get("avatar"),
        Some("https://cdn.discordapp.com/avatars/123456789/abcdef.png")
    );
    assert_eq!(result.get("time"), Some("2025-03-16T08:12:32.456Z"));
}

#[test]
fn parse_fail_not_at_start() {
    let input ="badge-info=;badges=;color=#0000FF;display-name=barbar;emote-sets=0,300374282;user-id=713936733;user-type= fue";

    let result = ircv3_tags::try_parse(input);
    assert!(result.is_err());
}
