use super::{Align, Base, NumFmt, Sign};

/// Builder for a numeric formatter.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Builder {
    fill: Option<char>,
    align: Align,
    sign: Sign,
    hash: bool,
    include_sign_in_width: bool,
    width: usize,
    precision: Option<usize>,
    format: Base,
    separator: Option<char>,
    spacing: Option<usize>,
    decimal_separator: Option<char>,
}

impl Builder {
    /// Construct a new `Builder`.
    pub fn new() -> Builder {
        Self::default()
    }

    /// Build a [`NumFmt`] instance, consuming this builder.
    pub fn build(self) -> NumFmt {
        let Builder {
            fill,
            align,
            sign,
            hash,
            include_sign_in_width,
            width,
            precision,
            format,
            separator,
            spacing,
            decimal_separator,
        } = self;
        NumFmt {
            fill,
            align,
            sign,
            hash,
            include_sign_in_width,
            width,
            precision,
            base: format,
            separator,
            spacing,
            decimal_separator,
        }
    }

    /// When `width` is greater than the actual rendered width of the number, the excess is padded
    /// with this character.
    ///
    /// ## Note
    ///  Wide characters are counted according to their bit width, not their quantity.
    ///
    /// ```rust
    /// let heart = '🖤';
    /// assert_eq!(heart.len_utf8(), 4);
    /// let fmt = NumFmt::builder().fill(heart).width(6).build();
    /// // Note that this renders as two characters: we requested a width of 6.
    /// // The number renders as a single character. The heart fills up the next 4 for a total of 5.
    /// // Adding an extra heart would exceed the requested width, so it only renders one.
    /// assert_eq!(fmt.fmt(1), "🖤1");
    /// ```
    #[inline]
    pub fn fill(mut self, param: char) -> Self {
        self.fill = Some(param);
        self
    }

    /// Set the alignment of rendering within allotted `width`. See [`Align`].
    #[inline]
    pub fn align(mut self, param: Align) -> Self {
        self.align = param;
        self
    }

    /// Set the rendering of the sign. See [`Sign`].
    #[inline]
    pub fn sign(mut self, param: Sign) -> Self {
        self.sign = param;
        self
    }

    /// If a `set`, print a base specification before the number
    /// according to its format.
    ///
    /// See [`Builder::format`].
    ///
    /// - binary: `0b`
    /// - octal: `0o`
    /// - decimal: `0d`
    /// - hex: `0x`
    ///
    /// Corresponds to the `#` format specifier.
    #[inline]
    pub fn hash(mut self, set: bool) -> Self {
        self.hash = set;
        self
    }

    /// If `set`, engage the zero handler.
    ///
    /// Conceptually, this is similar to the common pattern `0>`; it saves a
    /// char, and looks better when combined with a sign specifier. However, it comes
    /// with a caveat:
    ///
    /// ```rust
    /// assert_eq!(NumFmt::from_str("-03").unwrap().fmt(-1), "-01");
    /// assert_eq!(NumFmt::from_str("0>-3").unwrap().fmt(-1), "-001");
    /// ```
    ///
    /// The distinction is that the `0` handler includes the number's sign in the
    /// desired width; an explicit fill does not include the sign in the width
    /// calculation.
    #[inline]
    pub fn zero(mut self, set: bool) -> Self {
        if set {
            self.fill = Some('0');
            self.include_sign_in_width = true;
        } else {
            self.fill = None;
            self.include_sign_in_width = false;
        }
        self
    }

    /// Set the `width` parameter.
    ///
    /// This is a parameter for the "minimum width" that the format should take up. If
    /// the value's string does not fill up this many characters, then the padding
    /// specified by fill/alignment will be used to take up the required space (see
    /// [`Builder::fill`]).
    ///
    /// The width can be set dynamically:
    ///
    /// ```rust
    /// assert_eq!(NumFmt::from_str("-^$").unwrap().fmt_with(1, Dynamic::width(5)), "--1--");
    /// ```
    #[inline]
    pub fn width(mut self, param: usize) -> Self {
        self.width = param;
        self
    }

    /// Set the `precision` parameter.
    ///
    /// Ignored for integers.
    ///
    /// For non-integers, this is how many digits after the decimal point are printed.
    ///
    /// Precision will pad or truncate as required if set. If unset, passes through as many
    /// digits past the decimal as the underlying type naturally returns.
    ///
    /// ```rust
    /// assert_eq!(NumFmt::from_str("|^.$").unwrap().fmt_with(1, Dynamic::precision(5)), "|0.3|");
    /// ```
    ///
    /// If the requested precision exceeds the native precision available to this number,
    /// the remainder is always filled with `'0'`, even if `fill` is specified:
    ///
    /// ```rust
    /// assert_eq!(NumFmt::from_str("-<6.2").unwrap().fmt(1), "1.00--");
    /// ```
    #[inline]
    pub fn precision(mut self, param: Option<usize>) -> Self {
        self.precision = param;
        self
    }

    /// Set the output format.
    ///
    /// See [`Base`].
    #[inline]
    pub fn base(mut self, param: Base) -> Self {
        self.format = param;
        self
    }

    /// Set the separator.
    ///
    /// A separator is a (typically non-numeric) character inserted between groups of digits to make
    /// it easier for humans to parse the number when reading. Different separators may
    /// be desirable in different contexts.
    #[inline]
    pub fn separator(mut self, param: Option<char>) -> Self {
        self.separator = param;
        self
    }

    /// Set the spacing.
    ///
    /// Spacing determines the number of characters in each character group. It is only
    /// of interest when the separator is set. The default spacing is 3.
    #[inline]
    pub fn spacing(mut self, param: usize) -> Self {
        self.spacing = Some(param);
        self
    }

    /// Set the decimal separator.
    ///
    /// This can be desirable to i.e. support German number formats, which use a `.` to separate
    /// numeric groups and a `,` as a decimal separator.
    #[inline]
    pub fn decimal_separator(mut self, param: char) -> Self {
        self.decimal_separator = Some(param);
        self
    }
}

impl From<NumFmt> for Builder {
    fn from(
        NumFmt {
            fill,
            align,
            sign,
            hash,
            include_sign_in_width,
            width,
            precision,
            base: format,
            separator,
            spacing,
            decimal_separator,
        }: NumFmt,
    ) -> Self {
        Builder {
            fill,
            align,
            sign,
            hash,
            include_sign_in_width,
            width,
            precision,
            format,
            separator,
            spacing,
            decimal_separator,
        }
    }
}
