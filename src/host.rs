use nom::{
    branch::alt,
    character::complete::{alpha1, alphanumeric1, char},
    combinator::recognize,
    multi::many0,
    sequence::preceded,
    IResult, Parser,
};

/// From RFC 952:
/// - A "name" (hostname) is a text string up to 24 characters drawn from the alphabet (A-Z),
///   digits (0-9), minus sign (-), and period (.).
/// - Names must start with a letter, end with a letter or digit, and have as interior
///   characters only letters, digits, and hyphen.
/// - Segments are separated by periods.
/// - A valid hostname can have multiple segments.
pub fn host(input: &str) -> IResult<&str, &str> {
    // Hostname segment parser: must start with letter, end with letter/digit,
    // and contain only letters, digits, and hyphens in between
    fn hostname_segment(input: &str) -> IResult<&str, &str> {
        let (remain, segment) =
            recognize((alpha1, many0(alt((alphanumeric1, recognize(char('-'))))))).parse(input)?;

        if let Some(last_char) = segment.chars().last() {
            if !last_char.is_alphanumeric() {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::AlphaNumeric,
                )));
            }
        }

        Ok((remain, segment))
    }

    recognize((
        hostname_segment,
        many0(preceded(char('.'), hostname_segment)),
    ))
    .parse(input)
}

