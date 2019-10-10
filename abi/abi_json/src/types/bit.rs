#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Bit {
    Zero,
    One,
}

impl From<bool> for Bit {
    fn from(b: bool) -> Bit {
        if b {
            Bit::One
        } else {
            Bit::Zero
        }
    }
}
