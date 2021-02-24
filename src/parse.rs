use crate::{Align, Base, NumFmt, Sign};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PARSE_RE: Regex = Regex::new(
        r"(?x)
        ^
        (
            (?P<fill>.)?
            (?P<align>[<^>v])
        )?
        (?P<sign>[-+])?
        (?P<hash>(?-x:#))?
        (
         (?P<zero>0)?
         (?P<width>[1-9]\d*)
        )?
        (
         \.
         (?P<precision>\d+)
        )?
        (?P<format>[bodxX])?
        (
         (?P<separator>(?-x:[_, ]))
         (?P<spacing>\d+)?
        )?
        $"
    )
    .unwrap();
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("Input did not match canonical format string regex")]
    NoMatch,
    #[error("failed to parse integer value \"{0}\"")]
    ParseInt(String, #[source] std::num::ParseIntError),
}

/// Parse a `NumFmt` instance from a format string.
///
/// See crate-level docs for the grammar.
pub(crate) fn parse(s: &str) -> Result<NumFmt, Error> {
    let captures = PARSE_RE.captures(s).ok_or(Error::NoMatch)?;
    let str_of = |name: &str| captures.name(name).map(|m| m.as_str());
    let char_of = |name: &str| str_of(name).and_then(|s| s.chars().next());

    let mut builder = NumFmt::builder();

    if let Some(fill) = char_of("fill") {
        builder = builder.fill(fill);
    }
    if let Some(align) = char_of("align") {
        builder = builder.align(match align {
            '<' => Align::Left,
            '^' => Align::Center,
            '>' => Align::Right,
            'v' => Align::Decimal,
            _ => unreachable!("guaranteed by regex"),
        });
    }
    if let Some(sign) = char_of("sign") {
        builder = builder.sign(match sign {
            '-' => Sign::OnlyMinus,
            '+' => Sign::PlusAndMinus,
            _ => unreachable!("guaranteed by regex"),
        });
    }
    if char_of("hash").is_some() {
        builder = builder.hash(true);
    }
    if char_of("zero").is_some() {
        builder = builder.zero(true);
    }
    if let Some(width) = str_of("width") {
        let width = width
            .parse()
            .map_err(|err| Error::ParseInt(width.to_string(), err))?;
        builder = builder.width(width);
    }
    if let Some(precision) = str_of("precision") {
        let precision = precision
            .parse()
            .map_err(|err| Error::ParseInt(precision.to_string(), err))?;
        builder = builder.precision(Some(precision));
    }
    if let Some(format) = char_of("format") {
        builder = builder.base(match format {
            'b' => Base::Binary,
            'o' => Base::Octal,
            'd' => Base::Decimal,
            'x' => Base::LowerHex,
            'X' => Base::UpperHex,
            _ => unreachable!("guaranteed by regex"),
        });
    }
    builder = builder.separator(char_of("separator"));
    if let Some(spacing) = str_of("spacing") {
        let spacing = spacing
            .parse()
            .map_err(|err| Error::ParseInt(spacing.to_string(), err))?;
        builder = builder.spacing(spacing);
    }

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_re_matches() {
        for format_str in &[
            "",
            "<",
            "->",
            "#x",
            "+#04o",
            "v-10.2",
            "#04x_2",
            "-v-#012.3d 4",
        ] {
            println!("{:?}:", format_str);
            assert!(
                PARSE_RE.captures(format_str).is_some(),
                "all valid format strings must be parsed"
            );
        }
    }
}
