use ircv3_tags::host::{standard_host_validate, try_host, HostError};

#[test]
fn base() {
    let input = "example.com";
    let (remain, messages) = ircv3_tags::host::try_host(input).unwrap();
    assert_eq!(remain, "");
    assert_eq!(messages, "example.com");

    assert_eq!(
        try_host(""),
        Err(nom::Err::Error(HostError::new(
            "",
            nom::error::ErrorKind::Char,
            ircv3_tags::ErrorKind::Empty,
            "label must not be empty",
        )))
    );

    let inputs = ["-", "0", " "];

    for input in inputs {
        assert_eq!(
            try_host(input),
            Err(nom::Err::Error(HostError::new(
                input,
                nom::error::ErrorKind::Char,
                ircv3_tags::ErrorKind::Host,
                "label must start with an allowed character",
            )))
        );
    }

    assert_eq!(
        try_host("a-"),
        Err(nom::Err::Error(HostError {
            input: "a-",
            code: nom::error::ErrorKind::Char,
            error: ircv3_tags::ErrorKind::Host,
            reason: "label contains an invalid chracter",
        }))
    );

    assert_eq!(
        try_host("a--b"),
        Err(nom::Err::Error(HostError {
            input: "a--b",
            code: nom::error::ErrorKind::Char,
            error: ircv3_tags::ErrorKind::Host,
            reason: "label contains an invalid chracter",
        }))
    );

    assert_eq!(
        ircv3_tags::host::try_host("invalid-"),
        Err(nom::Err::Error(ircv3_tags::host::HostError::new(
            "invalid-",
            nom::error::ErrorKind::Char,
            ircv3_tags::ErrorKind::Host,
            "label contains an invalid chracter",
        )))
    );
}

#[test]
fn standard_host_validate_host() {
    assert!(standard_host_validate("example"));
    assert!(standard_host_validate("example.com"));
    assert!(standard_host_validate("example123.com"));
    assert!(standard_host_validate("example.com123"));
    assert!(standard_host_validate("exam123ple.com"));
    assert!(standard_host_validate("example.c123om"));
    assert!(standard_host_validate("examp123le.c123om"));
    assert!(standard_host_validate("exam-ple.com"));
    assert!(standard_host_validate("example.c-om"));
    assert!(standard_host_validate("sub.example.com"));
    assert!(!standard_host_validate("example-.com"));
    assert!(!standard_host_validate("-example.com"));
    assert!(!standard_host_validate("example.-com"));
    assert!(!standard_host_validate("example.com-"));
    assert!(!standard_host_validate("exam--ple.com"));
    assert!(!standard_host_validate("example.c--om"));
    assert!(!standard_host_validate("exam_ple.com"));
    assert!(!standard_host_validate("exam__ple.com"));
    assert!(!standard_host_validate("example.c_om"));
    assert!(!standard_host_validate("example..com"));
    assert!(!standard_host_validate("example..com"));
}

macro_rules! fn_test {
        ($name:ident, $fn:expr, [$($test:literal),+$(,)?]) => {
            #[test]
            fn $name(){
                let inputs = [$($test),+];

                for input in inputs {
                    assert!($fn(input).is_ok(), "{}", input);
                    let (remain, _) = $fn(input).unwrap();
                    assert!(remain.is_empty());
                }
            }
        };

        (err, $name:ident, $fn:expr, [$($test:literal),+$(,)?]) => {
            #[test]
            fn $name(){
                let inputs = [$($test),+];

                for input in inputs {
                    assert!($fn(input).is_err(), "'{}'", input);
                }
            }
        };

    }

macro_rules! fn_tests {
        ($fn:expr) => {
            fn_test!(
                simple,
                $fn,
                [
                    "example.com",
                    "host.a.z",
                    "server.host1",
                    "a-b-c.x-y-z123",
                ]
            );
            fn_test!(
                normal,
                $fn,
                [
                    "my-host-name",
                    "my-host-name.server-01",
                    "web-server-prod",
                    "app-server-1.database-cluster-a"
                ]
            );
            fn_test!(edge_cases,$fn,[
                "a-very-long-hostname-that-is-still-valid-according-to-rfc952-specs-with-multiple-hyphens",
                "a-b",
                "x1",
                "a-1"
            ]);
            fn_test!(
                max_length,
                $fn,
                ["abcdefghijklmnopqrstuvwxyz-0123456789-abcdefghijklmnopqrstuvwxyz"]
            );
        };
        ($fn:expr, err) => {
            fn_test!(
                err,
                starts_with_digit,
                $fn,
                [
                    "1host",
                    "host.1host",
                    "9server",
                    "123-invalid",
                    "0invalid"
                ]
            );
            fn_test!(
                err,
                starts_with_hyphen,
                $fn,
                [
                    "-host",
                    "host.-host",
                    "-server-1",
                    "-invalid-name"
                ]
            );
            fn_test!(
                err,
                ends_with_hyphen,
                $fn,
                [
                    "host-",
                    "host.host-",
                    "server-name-",
                    "server.server-name-",
                    "invalid-"
                ]
            );
            fn_test!(
                err,
                consecutive_hyphens,
                $fn,
                [
                    "host--name",
                    "host.host--name",
                    "server--01",
                    "double--hyphen--test",
                    "triple---hyphen",
                    "quadruple----hyphen"
                ]
            );
            fn_test!(err, empty, $fn, [""]);
            fn_test!(err, whitespace_only, $fn, [" ", "  ", "\t", "\n"]);
        };
        ($fn:expr, allow) => {
            fn_test!(
                simple,
                $fn,
                [
                    "example.com",
                    "host.a.z",
                    "server.host1",
                    "a-b-c.x-y-z123",
                    "a-b-c.x-y-z123",
                ]
            );
            fn_test!(
                normal,
                $fn,
                [
                    "my-host-name",
                    "my-host-name.server-01",
                    "web-server-prod",
                    "app-server-1.database-cluster-a"
                ]
            );
            fn_test!(edge_cases,$fn,[
                "a-very-long-hostname-that-is-still-valid-according-to-rfc952-specs-with-multiple-hyphens",
                "a-b",
                "x1",
                "a-1"
            ]);
            fn_test!(
                max_length,
                $fn,
                ["abcdefghijklmnopqrstuvwxyz-0123456789-abcdefghijklmnopqrstuvwxyz"]
            );
        };
        ($fn:expr, err, allow) => {
            fn_test!(
                err,
                starts_with_digit,
                $fn,
                [
                    "1host",
                    "host.1host",
                    "9server",
                    "123-invalid",
                    "0invalid"
                ]
            );
            fn_test!(
                err,
                starts_with_hyphen,
                $fn,
                [
                    "-host",
                    "host.-host",
                    "-server-1",
                    "-invalid-name"
                ]
            );
            fn_test!(
                err,
                ends_with_hyphen,
                $fn,
                [
                    "host-",
                    "host.host-",
                    "server-name-",
                    "server.server-name-",
                    "invalid-"
                ]
            );
            fn_test!(
                err,
                ends_with_underdash,
                $fn,
                [
                    "host_",
                    "host.host_",
                    "server-name_",
                    "server.server-name_",
                    "invalid_"
                ]
            );
            fn_test!(
                err,
                consecutive_hyphens,
                $fn,
                [
                    "host--name",
                    "host.host--name",
                    "server--01",
                    "double--hyphen--test",
                    "triple---hyphen",
                    "quadruple----hyphen"
                ]
            );
            fn_test!(err, empty, $fn, [""]);
            fn_test!(err, whitespace_only, $fn, [" ", "  ", "\t", "\n"]);
        }

    }
fn_tests!(ircv3_tags::host::host);
fn_tests!(ircv3_tags::host::host, err);
