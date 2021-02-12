/// The base / format with which to represent this number.
///
/// - `Binary`: Emit this number's binary representation
/// - `Octal`: Emit this number's octal representation
/// - `Decimal`: Emit this number's decimal representation (default)
/// - `LowerHex`: Emit this number's hexadecimal representation with lowercase letters
/// - `UpperHex`: Emit this number's hexadecimal representation with uppercase letters
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    LowerHex,
    UpperHex,
}

impl Default for Base {
    #[inline]
    fn default() -> Self {
        Self::Decimal
    }
}
