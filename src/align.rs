/// Positioning of rendered number within the allotted `width`.
///
/// - `Right`: the output is right-aligned in `width` columns (default).
/// - `Center`: the output is centered in `width` columns.
/// - `Left`: the output is left-aligned in `width` columns.
/// - `Decimal`: attempt to align the decimal point at column index `width`. For integers,
///   equivalent to `Right`.
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
