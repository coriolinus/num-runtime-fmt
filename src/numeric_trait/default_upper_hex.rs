///
pub struct DefaultUpperHex<LowerHex>(pub LowerHex);

impl<LowerHex> Iterator for DefaultUpperHex<LowerHex>
where
    LowerHex: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| c.to_ascii_uppercase())
    }
}
