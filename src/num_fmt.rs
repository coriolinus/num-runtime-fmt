use crate::{Align, Base, Builder, Dynamic, Numeric, Sign};
use iterext::prelude::*;
use std::collections::VecDeque;

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
    /// assert_eq!(fmt.fmt(0), "0x00");
    /// assert_eq!(fmt.fmt_with(0, Dynamic::width(4)), "0x00_00");
    /// ```
    ///
    /// Will return `None` in the event that the requested format is incompatible with
    /// the number provided. This is most often the case when the number is not an
    /// integer but an integer format such as `b`, `o`, or `x` is requested.
    pub fn fmt_with<N: Numeric>(&self, number: N, dynamic: Dynamic) -> Option<String> {
        let negative = number.is_negative() && self.base() == Base::Decimal;
        let separator = self.separator();
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
                if let Some(right) = right {
                    dq.push_front(self.decimal_separator());

                    let mut past_decimal: Box<dyn Iterator<Item = char>> = Box::new(right);
                    if let Some(precision) = self.precision_with(dynamic) {
                        past_decimal =
                            Box::new(past_decimal.chain(std::iter::repeat('0')).take(precision));
                    }

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
                let legal: &dyn Fn(&char) -> bool = match self.base() {
                    Base::Binary => &|ch| ('0'..='1').contains(ch),
                    Base::Octal => &|ch| ('0'..='7').contains(ch),
                    Base::Decimal => &|ch| *ch == '.' || ('0'..='9').contains(ch),
                    Base::LowerHex => &|ch| ('0'..='9').contains(ch) || ('a'..='f').contains(ch),
                    Base::UpperHex => &|ch| ('0'..='9').contains(ch) || ('A'..='F').contains(ch),
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
