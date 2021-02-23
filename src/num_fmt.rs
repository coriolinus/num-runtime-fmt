use crate::{parse, Align, Base, Builder, Dynamic, Numeric, Sign};
use iterext::prelude::*;
use std::{collections::VecDeque, str::FromStr};

/// Formatter for numbers.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct NumFmt {
    pub(crate) fill: Option<char>,
    pub(crate) align: Align,
    pub(crate) sign: Sign,
    pub(crate) hash: bool,
    pub(crate) zero: bool,
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

    /// Parse a `NumFmt` instance from a format string.
    ///
    /// See crate-level documentation for the grammar.
    pub fn from_str(s: &str) -> Result<Self, parse::Error> {
        parse::parse(s)
    }

    #[inline]
    fn width_desired(&self, dynamic: Dynamic) -> usize {
        let mut width_desired = self.width_with(dynamic);
        if self.hash() {
            width_desired = width_desired.saturating_sub(2);
        }

        width_desired
    }

    /// normalize a digit iterator
    ///
    /// - ensure that the iterator returns, bare minimum, a single char (default 0)
    /// - pad it to the desired width
    /// - space it out to the desired spacing
    fn normalize(&self, digits: impl Iterator<Item = char>, dynamic: Dynamic) -> VecDeque<char> {
        let pad_to = if self.zero() {
            self.width_desired(dynamic)
        } else {
            1
        };
        let pad_char = if self.zero() { '0' } else { self.fill() };

        let mut digits = digits.peekable();
        let digits: Box<dyn Iterator<Item = char>> = if digits.peek().is_some() {
            Box::new(digits)
        } else {
            Box::new(std::iter::once('0'))
        };

        digits
            .pad(pad_char, pad_to)
            .separate(self.separator(), self.spacing_with(dynamic))
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

        // core formatting: construct a reversed queue of digits, with separator and decimal
        // decimal is the index of the decimal point
        let (mut digits, decimal_pos): (VecDeque<_>, Option<usize>) = match self.base() {
            Base::Binary => (self.normalize(number.binary()?, dynamic), None),
            Base::Octal => (self.normalize(number.octal()?, dynamic), None),
            Base::Decimal => {
                let (left, right) = number.decimal();
                let mut dq = self.normalize(left, dynamic);
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
            Base::LowerHex => (self.normalize(number.hex()?, dynamic), None),
            Base::UpperHex => (
                self.normalize(number.hex()?.map(|ch| ch.to_ascii_uppercase()), dynamic),
                None,
            ),
        };
        let width_desired = self.width_desired(dynamic);
        let mut decimal_pos = decimal_pos.unwrap_or_else(|| digits.len());
        // padding and separating can introduce extraneous leading 0 chars, so let's fix that
        while decimal_pos > width_desired && {
            let last = *digits.back().expect("can't be empty while decimal_pos > 0");
            last == '0' || last == separator
        } {
            decimal_pos -= 1;
            digits.pop_back();
        }

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
        if sign_char.is_some() && self.zero {
            padding_front = padding_front.saturating_sub(1);
            if !digits.is_empty() {
                let back = *digits.back().expect("known not to be empty");
                if back == '0' || back == separator {
                    digits.pop_back();
                }
            }
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
        self.zero && self.fill() == '0'
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
    type Err = parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_width() {
        let fmt = NumFmt::from_str("#04x_2").unwrap();
        assert!(fmt.zero());
        assert_eq!(fmt.fmt(0).unwrap(), "0x00");

        let dynamic = Dynamic::width(7);
        dbg!(
            fmt.separator(),
            dynamic,
            fmt.width_with(dynamic),
            fmt.precision_with(dynamic),
            fmt.spacing_with(dynamic)
        );

        assert_eq!(fmt.fmt_with(0, dynamic).unwrap(), "0x00_00");
    }

    #[test]
    fn test_separator() {
        let fmt = NumFmt::from_str(",").unwrap();
        assert_eq!(fmt.fmt(123_456_789).unwrap(), "123,456,789");
    }
}
