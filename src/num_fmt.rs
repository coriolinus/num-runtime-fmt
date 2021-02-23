use crate::{Align, Base, Builder, Dynamic, Numeric, Sign};
use iterext::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::VecDeque, str::FromStr};

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
pub enum ParseError {
    #[error("Input did not match canonical format string regex")]
    NoMatch,
    #[error("failed to parse integer value \"{0}\"")]
    ParseInt(String, #[source] std::num::ParseIntError),
}

/// Formatter for numbers.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct NumFmt {
    pub(crate) fill: Option<char>,
    pub(crate) align: Align,
    pub(crate) sign: Sign,
    pub(crate) hash: bool,
    pub(crate) include_sign_in_width: bool,
    pub(crate) width: usize,
    pub(crate) precision: Option<usize>,
    pub(crate) base: Base,
    pub(crate) separator: Option<char>,
    pub(crate) spacing: Option<usize>,
    pub(crate) decimal_separator: Option<char>,
}

impl NumFmt {
    /// Create a [`Builder`] to customize the parameters of a `NumFmt`.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Format the provided number according to this configuration.
    ///
    /// Will return `None` in the event that the requested format is incompatible with
    /// the number provided. This is most often the case when the number is not an
    /// integer but an integer format such as `b`, `o`, or `x` is requested.
    pub fn fmt<N: Numeric>(&self, number: N) -> Option<String> {
        self.fmt_with(number, Dynamic::default())
    }

    /// Format the provided number according to this configuration and dynamic parameters.
    ///
    /// Note that dynamic parameters always override the formatter's parameters:
    ///
    /// ```rust
    /// # use num_runtime_fmt::{NumFmt, Dynamic};
    /// let fmt = NumFmt::from_str("#04x_2").unwrap();
    /// assert_eq!(fmt.fmt(0).unwrap(), "0x00");
    /// assert_eq!(fmt.fmt_with(0, Dynamic::width(7)).unwrap(), "0x00_00");
    /// ```
    ///
    /// Will return `None` in the event that the requested format is incompatible with
    /// the number provided. This is most often the case when the number is not an
    /// integer but an integer format such as `b`, `o`, or `x` is requested.
    pub fn fmt_with<N: Numeric>(&self, number: N, dynamic: Dynamic) -> Option<String> {
        let negative = number.is_negative() && self.base() == Base::Decimal;
        let separator = self.separator();
        let decimal_separator = self.decimal_separator();
        let spacing = self.spacing_with(dynamic);

        // core formatting: construct a reversed queue of digits, with separator and decimal
        // decimal is the index of the decimal point
        let (digits, decimal_pos): (VecDeque<_>, Option<usize>) = match self.base() {
            Base::Binary => (number.binary()?.separate(separator, spacing), None),
            Base::Octal => (number.octal()?.separate(separator, spacing), None),
            Base::Decimal => {
                let (left, right) = number.decimal();
                let mut dq: VecDeque<_> = left.separate(separator, spacing);
                let decimal = dq.len();
                let past_decimal: Option<Box<dyn Iterator<Item = char>>> =
                    match (right, self.precision_with(dynamic)) {
                        (Some(digits), None) => Some(Box::new(digits)),
                        (Some(digits), Some(precision)) => Some(Box::new(
                            digits.chain(std::iter::repeat('0')).take(precision),
                        )),
                        (None, Some(precision)) => {
                            Some(Box::new(std::iter::repeat('0').take(precision)))
                        }
                        (None, None) => None,
                    };
                if let Some(past_decimal) = past_decimal {
                    dq.push_front(self.decimal_separator());

                    // .extend only pushes to the back
                    for item in past_decimal {
                        dq.push_front(item);
                    }
                }
                (dq, Some(decimal))
            }
            Base::LowerHex => (number.hex()?.separate(separator, spacing), None),
            Base::UpperHex => (
                number
                    .hex()?
                    .map(|ch| ch.to_ascii_uppercase())
                    .separate(separator, spacing),
                None,
            ),
        };
        let decimal_pos = decimal_pos.unwrap_or_else(|| digits.len());

        debug_assert!(
            {
                let legal: Box<dyn Fn(&char) -> bool> = match self.base() {
                    Base::Binary => {
                        Box::new(move |ch| *ch == separator || ('0'..='1').contains(ch))
                    }
                    Base::Octal => Box::new(move |ch| *ch == separator || ('0'..='7').contains(ch)),
                    Base::Decimal => Box::new(move |ch| {
                        *ch == decimal_separator || *ch == separator || ('0'..='9').contains(ch)
                    }),
                    Base::LowerHex => Box::new(move |ch| {
                        *ch == separator || ('0'..='9').contains(ch) || ('a'..='f').contains(ch)
                    }),
                    Base::UpperHex => Box::new(move |ch| {
                        *ch == separator || ('0'..='9').contains(ch) || ('A'..='F').contains(ch)
                    }),
                };
                digits.iter().all(legal)
            },
            "illegal characters in number; check its `impl Numeric`",
        );

        let width_used = digits.len();
        let width_desired = self.width_with(dynamic);
        let (mut padding_front, padding_rear) = match self.align() {
            Align::Right => (width_desired.saturating_sub(width_used), 0),
            Align::Left => (0, width_desired.saturating_sub(width_used)),
            Align::Center => {
                let unused_width = width_desired.saturating_sub(width_used);
                let half_unused_width = unused_width / 2;
                // bias right
                (unused_width - half_unused_width, half_unused_width)
            }
            Align::Decimal => (width_desired.saturating_sub(decimal_pos), 0),
        };

        let sign_char = match (self.sign(), negative) {
            (Sign::PlusAndMinus, _) => Some(if negative { '-' } else { '+' }),
            (Sign::OnlyMinus, true) => Some('-'),
            (Sign::OnlyMinus, false) => None,
        };
        if sign_char.is_some() && self.include_sign_in_width {
            padding_front = padding_front.saturating_sub(1);
        }

        let prefix = match (self.hash(), self.base()) {
            (false, _) => None,
            (_, Base::Binary) => Some("0b"),
            (_, Base::Octal) => Some("0o"),
            (_, Base::Decimal) => Some("0d"),
            (_, Base::LowerHex) | (_, Base::UpperHex) => Some("0x"),
        };
        if prefix.is_some() {
            padding_front = padding_front.saturating_sub(2);
        }

        // constant 3 ensures that even with a sign and a prefix, we don't have to reallocate
        let mut rendered = String::with_capacity(padding_front + padding_rear + width_used + 3);

        // finally, assemble all the ingredients
        if let Some(sign) = sign_char {
            rendered.push(sign);
        }
        if let Some(prefix) = prefix {
            rendered.push_str(prefix);
        }
        for _ in 0..padding_front {
            rendered.push(self.fill());
        }
        for digit in digits.into_iter().rev() {
            rendered.push(digit);
        }
        for _ in 0..padding_rear {
            rendered.push(self.fill());
        }

        Some(rendered)
    }

    /// Parse a `NumFmt` instance from a format string.
    ///
    /// This is implemented as a native method in order to reduce the need to import an extra trait,
    /// but `std::str::FromStr` is also implemented for `NumFmt` and delegates to this method.
    ///
    /// ## Grammar
    ///
    /// The gramar for the format string derives substantially from the standard library's:
    ///
    /// ```text
    /// format_spec := [[fill]align][sign]['#'][['0']width]['.' precision][format][separator[spacing]]
    /// fill := character
    /// align := '<' | '^' | '>' | 'v'
    /// sign := '+' | '-'
    /// width := integer not beginning with '0'
    /// precision := integer
    /// format := 'b' | 'o' | 'd' | 'x' | 'X'
    /// separator := '_', | ',' | ' '
    /// spacing := integer
    /// ```
    ///
    /// ### Note
    ///
    /// There is no special syntax for dynamic insertion of `with`, `precision` and `spacing`.
    /// Simply use [`NumFmt::format_with`]; the dynamic values provided there always override any
    /// values for those fields, whether set or not in the format string.
    ///
    /// ## `fill`
    ///
    /// Any single `char` which precedes an align specifier is construed as the fill
    /// character: when `width` is greater than the actual rendered width of the number,
    /// the excess is padded with this character.
    ///
    /// ### Note
    /// Wide characters are counted according to their quantity, not their bit width.
    ///
    /// ```rust
    /// # use num_runtime_fmt::NumFmt;
    /// let heart = '🖤';
    /// assert_eq!(heart.len_utf8(), 4);
    /// let fmt = NumFmt::builder().fill(heart).width(3).build();
    /// let formatted = fmt.fmt(1).unwrap();
    /// assert_eq!(formatted, "🖤🖤1");
    /// // Note that even though we requested a width of 3, the binary length is 9.
    /// assert_eq!(formatted.len(), 9);
    /// ```
    ///
    /// ## `align`ment
    ///
    /// - `>`: the output is right-aligned in `width` columns (default).
    /// - `^`: the output is centered in `width` columns.
    /// - `<`: the output is left-aligned in `width` columns.
    /// - `v`: attempt to align the decimal point at column index `width`. For integers,
    ///   equivalent to `>`.
    ///
    /// ## `sign`
    ///
    /// - `-`: print a leading `-` for negative numbers, and nothing in particular for
    ///   positive (default)
    /// - `+`: print a leading `+` for positive numbers
    ///
    /// ## `#`
    ///
    /// If a `#` character is present, print a base specification before the number
    /// according to its format (see `format` below).
    ///
    /// - binary: `0b`
    /// - octal: `0o`
    /// - decimal: `0d`
    /// - hex: `0x`
    ///
    /// This base specification counts toward the width of the number:
    ///
    /// ```rust
    /// # use num_runtime_fmt::NumFmt;
    /// assert_eq!(NumFmt::from_str("#04b").unwrap().fmt(2).unwrap(), "0b10");
    /// ```
    ///
    /// ## `0`
    ///
    /// Conceptually, this is similar to the common pattern `0>`; it saves a
    /// char, and looks better when combined with a sign specifier. However, it comes
    /// with a caveat:
    ///
    /// ```rust
    /// # use num_runtime_fmt::NumFmt;
    /// assert_eq!(NumFmt::from_str("-03").unwrap().fmt(-1).unwrap(), "-01");
    /// assert_eq!(NumFmt::from_str("0>-3").unwrap().fmt(-1).unwrap(), "-001");
    /// ```
    ///
    /// The distinction is that the `0` formatter includes the number's sign in the
    /// desired width; an explicit fill does not include the sign in the width
    /// calculation.
    ///
    /// ## `width`
    ///
    /// This is a parameter for the "minimum width" that the format should take up. If
    /// the value's string does not fill up this many characters, then the padding
    /// specified by fill/alignment will be used to take up the required space (see
    /// `fill` above).
    ///
    /// When using the `$` sigil instead of an explicit width, the width can be set
    /// dynamically:
    ///
    /// ```rust
    /// # use num_runtime_fmt::{NumFmt, Dynamic};
    /// assert_eq!(NumFmt::from_str("-^").unwrap().fmt_with(1, Dynamic::width(5)).unwrap(), "--1--");
    /// ```
    ///
    /// If an explicit width is not provided, defaults to 0.
    ///
    /// ## `precision`
    ///
    /// Precision will pad or truncate as required if set. If unset, passes through as many
    /// digits past the decimal as the underlying type naturally returns.
    ///
    /// ```rust
    /// # use num_runtime_fmt::{NumFmt, Dynamic};
    /// assert_eq!(NumFmt::from_str(".2").unwrap().fmt(3.14159).unwrap(), "3.14");
    /// assert_eq!(NumFmt::from_str(".7").unwrap().fmt(3.14159).unwrap(), "3.1415900");
    /// ```
    ///
    /// If the requested precision exceeds the native precision available to this number,
    /// the remainder is always filled with `'0'`, even if `fill` is specified:
    ///
    /// ```rust
    /// # use num_runtime_fmt::NumFmt;
    /// assert_eq!(NumFmt::from_str("-<6.2").unwrap().fmt(1.0_f32).unwrap(), "1.00--");
    /// ```
    ///
    /// ## `format`
    ///
    /// - `b`: Emit this number's binary representation
    /// - `o`: Emit this number's octal representation
    /// - `d`: Emit this number's decimal representation (default)
    /// - `x`: Emit this number's hexadecimal representation with lowercase letters
    /// - `X`: Emit this number's hexadecimal representation with uppercase letters
    ///
    /// ### Note
    ///
    /// This is one of a few areas where the standard library has
    /// capabilities this library does not: it supports some other numeric formats.
    /// Pull requests welcomed to bring this up to parity.
    ///
    /// ## `separator`
    ///
    /// A separator is a (typically non-numeric) character inserted between groups of digits to make
    /// it easier for humans to parse the number when reading. Different separators may
    /// be desirable in different contexts.
    ///
    /// - `_`: Separate numeric groups with an underscore
    /// - `,`: Separate numeric groups with a comma
    /// - ` ` (space char): Separate numeric groups with a space
    ///
    /// By default, numeric groups are not separated. It is not possible to explicitly
    /// specify that numeric groups are not separated when using a format string.
    /// However, this can be specified when building the formatter via builder.
    ///
    /// When using the builder to explicitly set formatter options, it is also possible
    /// to separate numeric groups with an arbitrary `char`. This can be desirable to
    /// i.e. support German number formats, which use a `.` to separate numeric groups
    /// and a `,` as a decimal separator.
    ///
    /// ## `spacing`
    ///
    /// Spacing determines the number of characters in each character group. It is only
    /// of interest when the separator is set. The default spacing is 3.
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let captures = PARSE_RE.captures(s).ok_or(ParseError::NoMatch)?;
        let str_of = |name: &str| captures.name(name).map(|m| m.as_str());
        let char_of = |name: &str| str_of(name).and_then(|s| s.chars().next());

        let mut builder = Self::builder();

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
                .map_err(|err| ParseError::ParseInt(width.to_string(), err))?;
            builder = builder.width(width);
        }
        if let Some(precision) = str_of("precision") {
            let precision = precision
                .parse()
                .map_err(|err| ParseError::ParseInt(precision.to_string(), err))?;
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
                .map_err(|err| ParseError::ParseInt(spacing.to_string(), err))?;
            builder = builder.spacing(spacing);
        }

        Ok(builder.build())
    }

    /// `char` used to pad the extra space when the rendered number is smaller than the `width`.
    #[inline]
    pub fn fill(&self) -> char {
        self.fill.unwrap_or(' ')
    }

    /// Desired alignment.
    #[inline]
    pub fn align(&self) -> Align {
        self.align
    }

    /// Which signs are printed with the number.
    #[inline]
    pub fn sign(&self) -> Sign {
        self.sign
    }

    /// Whether to print a base specification before the number.
    #[inline]
    pub fn hash(&self) -> bool {
        self.hash
    }

    /// Whether the zero formatter was used.
    #[inline]
    pub fn zero(&self) -> bool {
        self.include_sign_in_width && self.fill() == '0'
    }

    /// Requested render width in bytes.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Requested post-decimal precision in bytes.
    ///
    /// Precision will pad or truncate as required if set. If unset, passes through as many
    /// digits past the decimal as the underlying type naturally returns.
    #[inline]
    pub fn precision(&self) -> Option<usize> {
        self.precision
    }

    /// Requested output format.
    #[inline]
    pub fn base(&self) -> Base {
        self.base
    }

    /// Requested group separator.
    #[inline]
    pub fn separator(&self) -> char {
        self.separator.unwrap_or(',')
    }

    /// Requested group size.
    #[inline]
    pub fn spacing(&self) -> usize {
        self.spacing.unwrap_or(3)
    }

    /// Requested decimal separator.
    #[inline]
    pub fn decimal_separator(&self) -> char {
        self.decimal_separator.unwrap_or('.')
    }

    fn width_with(&self, dynamic: Dynamic) -> usize {
        dynamic.width.unwrap_or(self.width)
    }

    fn precision_with(&self, dynamic: Dynamic) -> Option<usize> {
        dynamic.precision.or(self.precision)
    }

    fn spacing_with(&self, dynamic: Dynamic) -> usize {
        dynamic.spacing.unwrap_or_else(|| self.spacing())
    }
}

impl FromStr for NumFmt {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <NumFmt>::from_str(s)
    }
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
