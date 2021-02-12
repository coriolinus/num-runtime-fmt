#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
    Decimal,
}

impl Default for Align {
    #[inline]
    fn default() -> Self {
        Self::Right
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sign {
    Plus,
    Minus,
}

impl Default for Sign {
    #[inline]
    fn default() -> Self {
        Self::Minus
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Binary,
    Octal,
    Decimal,
    LowerHex,
    UpperHex,
}

impl Default for Format {
    #[inline]
    fn default() -> Self {
        Self::Decimal
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct NumFmt {
    fill: Option<char>,
    align: Align,
    sign: Sign,
    hash: bool,
    width: usize,
    precision: usize,
    format: Format,
    separator: Option<char>,
    spacing: Option<usize>,
    decimal_separator: Option<char>,
}
