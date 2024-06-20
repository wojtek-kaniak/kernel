pub mod boot;
pub mod devices;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

// TODO: reexport contract / trait with proc macros

use core::{fmt::{Display, Pointer, Debug}, ops::{Add, Sub, AddAssign, SubAssign, Rem, RemAssign}};

use crate::common::DebugHex;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn next_multiple_of(&self, rhs: usize) -> Self {
        Self(self.0.next_multiple_of(rhs))
    }

    #[must_use]
    pub const fn last_multiple_of(&self, rhs: usize) -> Self {
        Self(self.0 / rhs * rhs)
    }

    #[must_use]
    pub const fn is_aligned<T>(&self) -> bool {
        // TODO: refactor to core::ptr::Alignment when stablized
        (self.0 % core::mem::align_of::<T>()) == 0
    }

    #[must_use]
    pub const fn is_aligned_to(&self, alignment: usize) -> bool {
        (self.0 % alignment) == 0
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Rem<usize> for PhysicalAddress {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 % rhs
    }
}

impl RemAssign<usize> for PhysicalAddress {
    fn rem_assign(&mut self, rhs: usize) {
        self.0 %= rhs;
    }
}

impl From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<u64> for PhysicalAddress {
    fn from(value: u64) -> Self {
        Self(value as usize)
    }
}

impl From<PhysicalAddress> for usize {
    fn from(val: PhysicalAddress) -> Self {
        val.0
    }
}

impl From<PhysicalAddress> for u64 {
    fn from(val: PhysicalAddress) -> Self {
        val.0 as u64
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
        f.debug_tuple(stringify!(PhysicalAddress)).field(&DebugHex::new(&self.0)).finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn next_multiple_of(&self, rhs: usize) -> Self {
        Self(self.0.next_multiple_of(rhs))
    }

    #[must_use]
    pub const fn last_multiple_of(&self, rhs: usize) -> Self {
        Self(self.0 / rhs * rhs)
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *const () {
        self.0 as *const ()
    }

    #[must_use]
    pub const fn as_mut_ptr(&self) -> *mut () {
        self.0 as *mut ()
    }
}

impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtualAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for VirtualAddress {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs
    }
}

impl Sub<VirtualAddress> for VirtualAddress {
    type Output = usize;

    fn sub(self, rhs: VirtualAddress) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Rem<usize> for VirtualAddress {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 % rhs
    }
}

impl RemAssign<usize> for VirtualAddress {
    fn rem_assign(&mut self, rhs: usize) {
        self.0 %= rhs;
    }
}

impl<T> From<*const T> for VirtualAddress {
    fn from(value: *const T) -> Self {
        (value as usize).into()
    }
}

impl<T> From<*mut T> for VirtualAddress {
    fn from(value: *mut T) -> Self {
        (value as usize).into()
    }
}

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<u64> for VirtualAddress {
    fn from(value: u64) -> Self {
        Self(value as usize)
    }
}

impl From<VirtualAddress> for usize {
    fn from(val: VirtualAddress) -> Self {
        val.0
    }
}

impl From<VirtualAddress> for u64 {
    fn from(val: VirtualAddress) -> Self {
        val.0 as u64
    }
}

impl Pointer for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(&self, f)
    }
}

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(stringify!(VirtualAddress)).field(&DebugHex::new(&self.0)).finish()
    }
}

