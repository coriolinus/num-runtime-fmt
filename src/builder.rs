use super::{Align, Base, NumFmt, Sign};

/// Builder for a numeric formatter.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Builder {
    fill: Option<char>,
    align: Align,
    sign: Sign,
    hash: bool,
    zero: bool,
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
            zero,
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
            zero,
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
    /// The zero handler overrides the padding specification to `0`, and
    /// treats pad characters as part of the number, in contrast
    /// to the default behavior which treats them as arbitrary spacing.
    ///
    /// Only valid with `Align::Right` and `Align::Decimal`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use num_runtime_fmt::NumFmt;
    /// // sign handling
    /// assert_eq!(NumFmt::from_str("-03").unwrap().fmt(-1).unwrap(),   "-01");
    /// assert_eq!(NumFmt::from_str("0>-3").unwrap().fmt(-1).unwrap(), "-001");
    /// ```
    ///
    /// ```rust
    /// # use num_runtime_fmt::NumFmt;
    /// // separator handling
    /// assert_eq!(NumFmt::from_str("0>7,").unwrap().fmt(1).unwrap(), "0000001");
    /// assert_eq!(NumFmt::from_str("07,").unwrap().fmt(1).unwrap(),  "000,001");
    /// ```
    #[inline]
    pub fn zero(mut self, set: bool) -> Self {
        if set {
            self.fill = Some('0');
            self.zero = true;
        } else {
            self.fill = None;
            self.zero = false;
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
    /// # use num_runtime_fmt::{NumFmt, Dynamic};
    /// assert_eq!(NumFmt::from_str("-^").unwrap().fmt_with(1, Dynamic::width(5)).unwrap(), "--1--");
    /// ```
    #[inline]
    pub fn width(mut self, param: usize) -> Self {
        self.width = param;
        self
    }

    /// Set the `precision` parameter.
    ///
    /// How many digits after the decimal point are printed. Note that integers can be forced
    /// to emit decimal places with this modifier.
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
            zero,
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
            zero,
            width,
            precision,
            format,
            separator,
            spacing,
            decimal_separator,
        }
    }
}
