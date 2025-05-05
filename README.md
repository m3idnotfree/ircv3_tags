# IRCv3 Tags Parser

[API Docs](https://docs.rs/ircv3_tags)

## Examples

```rust
let input = "@id=234AB;+example.com/key=value :nick!user@host PRIVMSG #channel :Hello";
let (remain, tags) = ircv3_tags::parse(input);

assert_eq!(remain, ":nick!user@host PRIVMSG #channel :Hello");
assert_eq!(tags.get("id"), Some("234AB"));
assert_eq!(tags.get("+example.com/key"), Some("value"));
```

Returns a IResult

```rust
let input = "@id=123 :nick!user@host PRIVMSG #channel :Hello";
let result = ircv3_tags::try_parse(input);
assert!(result.is_ok());
```
