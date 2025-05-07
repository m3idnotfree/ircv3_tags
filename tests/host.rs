use ircv3_tags::host;

#[test]
fn hos_t() {
    let input = "example.com";
    let (remain, result) = host(input).unwrap();
    assert_eq!(remain, "");
    assert_eq!(result, "example.com");

    let input = "a.b.c";
    let (remain, result) = host(input).unwrap();
    assert_eq!(remain, "");
    assert_eq!(result, "a.b.c");

    let input = "example-test.domain.com";
    let (remain, result) = host(input).unwrap();
    assert_eq!(remain, "");
    assert_eq!(result, "example-test.domain.com");

    let input = "example..com";
    let (remain, result) = host(input).unwrap();
    assert_eq!(remain, "..com");
    assert_eq!(result, "example");

    let input = "example.-test.com";
    let (remain, result) = host(input).unwrap();
    assert_eq!(remain, ".-test.com");
    assert_eq!(result, "example");

    let input = "example-.com";
    let result = host(input);
    assert!(result.is_err());

    let input = "examp@le.com";
    let (remain, result) = host(input).unwrap();
    assert_eq!(remain, "@le.com");
    assert_eq!(result, "examp");

    let result = host("123.com");
    assert!(result.is_err());

    let (remain, result) = host("a123.com").unwrap();
    assert_eq!(remain, "");
    assert_eq!(result, "a123.com");

    let (remain, result) = host("a123.1com").unwrap();
    assert_eq!(remain, ".1com");
    assert_eq!(result, "a123");
}
