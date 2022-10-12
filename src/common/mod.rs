use core::fmt::{Debug, LowerHex};

pub mod collections;
pub mod macros;
pub mod random;
pub mod sync;
pub mod time;

#[repr(transparent)]
pub struct DebugHex<T: LowerHex>(T);

impl<T: LowerHex> DebugHex<T> {
    pub fn new(value: T) -> Self {
        DebugHex(value)
    }
}

impl<T: LowerHex + Clone> Clone for DebugHex<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: LowerHex + Copy> Copy for DebugHex<T> {}

impl<T: LowerHex> Debug for DebugHex<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
