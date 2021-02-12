/// Whether to render a sign sigil.
///
/// - `OnlyMinus`: print a leading `-` for negative numbers, and nothing in particular for
///   positive (default)
/// - `PlusAndMinus`: print a leading `+` for positive numbers
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sign {
    PlusAndMinus,
    OnlyMinus,
}

impl Default for Sign {
    #[inline]
    fn default() -> Self {
        Self::OnlyMinus
    }
}
