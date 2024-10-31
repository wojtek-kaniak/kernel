use crate::common::macros::assert_arg;

pub mod interrupts;
pub mod paging;
pub mod intrinsics;

/// Segment selector
/// 
/// # Fields
/// [0, 1]  - RPL (requested privilage level)
/// [2, 2]  - Table indicator
/// [3, 15] - Segment index
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    pub const fn new(index: SegmentIndex, ti: TableIndicator, rpl: PrivilegeLevel) -> Self {
        let mut selector = Self(0);
        selector.set_index(index);
        selector.set_table_indicator(ti);
        selector.set_rpl(rpl);
        selector
    }

    pub const fn null() -> Self {
        SegmentSelector(0)
    }
    
    pub const fn index(self) -> u16 {
        self.0 >> 3
    }

    pub const fn set_index(&mut self, index: SegmentIndex) {
        const MASK: u16 = !(((1 << 4) - 1) << 3);
        self.0 = (self.0 & MASK) | (index.0 << 3);
    }

    pub const fn rpl(self) -> PrivilegeLevel {
        PrivilegeLevel(self.0 as u8 & 0b11)
    }

    pub const fn set_rpl(&mut self, rpl: PrivilegeLevel) {
        const MASK: u16 = !0b11;
        self.0 = (self.0 & MASK) | rpl.0 as u16;
    }

    pub const fn table_indicator(self) -> TableIndicator {
        match self.0 >> 2 & 1 {
            0 => TableIndicator::Gdt,
            1 => TableIndicator::Ldt,
            _ => unreachable!(),
        }
    }

    pub const fn set_table_indicator(&mut self, value: TableIndicator) {
        const MASK: u16 = !(1 << 2);
        self.0 = (self.0 & MASK) | ((value as u16) << 2);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentIndex(u16);

impl SegmentIndex {
    /// `value` must be lower than 8192
    pub const fn new(value: u16) -> Self {
        assert!(value < 8192);

        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableIndicator {
    Gdt = 0,
    Ldt = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivilegeLevel(u8);

impl PrivilegeLevel {
    pub const KERNEL: PrivilegeLevel = PrivilegeLevel(0);
    pub const USERSPACE: PrivilegeLevel = PrivilegeLevel(3);

    /// `value` must be smaller than 4
    pub const fn new(value: u8) -> Self {
        assert!(value < 4);

        PrivilegeLevel(value)
    }
}

impl From<PrivilegeLevel> for u8 {
    fn from(val: PrivilegeLevel) -> Self {
        val.0
    }
}
