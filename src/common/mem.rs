// TODO: remove and replace with specific traits
/// # Safety
/// Every bit pattern is valid for the marked type
pub unsafe trait Bittable {}

pub struct Aligned<const ALIGNMENT: usize, T> where elain::Align<ALIGNMENT>: elain::Alignment {
    _align: elain::Align<ALIGNMENT>,
    pub value: T
}

impl<const ALIGNMENT: usize, T> Aligned<ALIGNMENT, T>
where elain::Align<ALIGNMENT>: elain::Alignment
{
    pub const fn new(value: T) -> Self {
        Self {
            _align: elain::Align::NEW,
            value,
        }
    }
}
