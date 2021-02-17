pub mod impls;

/// This trait enables a type to be formatted by [`NumFmt`][crate::NumFmt].
///
/// The fundamental abstraction used is an optional iterator over a stream of characters. Returning
/// `None` always indicates that representation in that base is not available for this type. Any
/// particular function implemented for a type should always return either `None` or `Some`; it
/// should not depend on the value being formatted.
///
/// In all cases, when implemented, the iterators must iterate away from the decimal: for a
/// representation of `N` in base `B`, it must return the appropriate digit for `B**0`, `B**1`, ...
/// `BN**k` where `k` is `ceil(log_B(N))`.
///
/// Iterators should only return digits within the appropriate range for the base. All other
/// formatting is handled by the formatter.
///
/// Iterator types must be declared even when the appropriate function always returns `None`. In
/// those cases, [`std::iter::Empty`] is appropriate.
pub trait Numeric {
    /// Iterate over binary digits of this number.
    ///
    /// Legal output characters: `[01]`.
    type BinIter: Iterator<Item = char>;

    /// Iterate over octal digits of this number.
    ///
    /// Legal output characters: `[0-7]`.
    type OctIter: Iterator<Item = char>;

    /// Iterate over decimal digits of this number which are >= 1.
    ///
    /// Legal output characters: `[0-9]`.
    type DecLeftIter: Iterator<Item = char>;

    /// Iterate over decimal digits of this number which are < 1.
    ///
    /// Legal output characters: `[0-9]`.
    ///
    /// This should iterate away from the decimal: for a representation of `N`, it must return the appropriate
    /// digit for `10**-1`, `10**-2`, etc.
    type DecRightIter: Iterator<Item = char>;

    /// Iterate over hexadecimal digits of this number, with letters as lowercase.
    ///
    /// Legal output characters: `[0-9a-f]`.
    type HexIter: Iterator<Item = char>;

    /// Iterate over the binary digits of this number, from least to most significant.
    ///
    /// This function should always return either `None` or `Some`; it should not depend on the
    /// value of `self`.
    fn binary(&self) -> Option<Self::BinIter>;

    /// Iterate over the octal digits of this number, from least to most significant.
    ///
    /// This function should always return either `None` or `Some`; it should not depend on the
    /// value of `self`.
    fn octal(&self) -> Option<Self::OctIter>;

    /// Produce a pair of iterators over the decimal digits of this number.
    ///
    /// ## `DecLeftIter`
    ///
    /// `Self::DecLeftIter` must iterate over the decimal digits of this number which are >= 1, from
    /// least to most significant. Note that it is assumed that all numeric types can produce a
    /// decimal representation of an integer component.
    ///
    /// ## `DecRightIter`
    ///
    /// `Self::DecRightIter` should iterate away from the decimal: for a representation of `N`, it
    /// must return the appropriate digit for `10**-1`, `10**-2`, etc.
    ///
    /// It is an exception to the general rule; it may return `None` or `Some` according to the
    /// value of `self`.
    ///
    /// If `Self` is not an integral type, such as `f64`, but `self` is an integer, like `1.0`, then
    /// the output will vary by what this function returns as follows:
    ///
    /// - `None` => `"1"`
    /// - `Some(std::iter::empty())` => `"1."`
    /// - `Some(std::iter::once('0')) => `"1.0"`
    fn decimal(&self) -> (Self::DecLeftIter, Option<Self::DecRightIter>);

    /// Iterate over the hexadecimal digits of this number, with letters as lowercase.
    ///
    /// This function should always return either `None` or `Some`; it should not depend on the
    /// value of `self`.
    ///
    /// Note that the implementation must provide only the lowercase implementation. The formatter
    /// uppercases the output of this function when the user requests uppercase hexadecimal.
    fn hex(&self) -> Option<Self::HexIter>;

    /// `true` when this value is less than 0.
    fn is_negative(&self) -> bool;
}
