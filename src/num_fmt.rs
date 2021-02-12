use super::{Align, Base, Builder, Dynamic, Sign};

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
    pub fn fmt<N>(&self, number: N) -> String {
        self.fmt_with(number, Dynamic::default())
    }

    /// Format the provided number according to this configuration and dynamic parameters.
    ///
    /// Note that dynamic parameters always override the formatter's parameters:
    ///
    /// ```rust
    /// let fmt = NumFmt::from_str("#04x_2").unwrap();
    /// assert_eq!(fmt.fmt(0), "0x00");
    /// assert_eq!(fmt.fmt_with(0, Dynamic::width(4)), "0x00_00");
    /// ```
    pub fn fmt_with<N>(&self, number: N, dynamic: Dynamic) -> String {
        unimplemented!()
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
