use core::{fmt::{Display, Pointer, Debug}, ops::{Add, Sub, AddAssign, SubAssign}};

use crate::common::DebugHex;

pub mod interrupts;
pub mod intrinsics;
pub mod syscalls;
pub mod paging;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub const fn new(value: usize) -> Self {
        PhysicalAddress(value)
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn add(self, rhs: usize) -> Self::Output {
        PhysicalAddress(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn sub(self, rhs: usize) -> Self::Output {
        PhysicalAddress(self.0 - rhs)
    }
}

impl SubAssign<usize> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = usize;

    fn sub(self, rhs: PhysicalAddress) -> Self::Output {
        self.0 - rhs.0
    }
}

impl From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        PhysicalAddress(value)
    }
}

impl From<u64> for PhysicalAddress {
    fn from(value: u64) -> Self {
        PhysicalAddress(value as usize)
    }
}

impl Into<usize> for PhysicalAddress {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<u64> for PhysicalAddress {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl Pointer for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(&self, f)
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysicalAddress").field(&DebugHex::new(&self.0)).finish()
    }
}
