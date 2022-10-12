use static_assertions::const_assert_eq;

use crate::arch::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

// #[repr(C, align(4096))]
// pub struct Level5PageTable {
//     entries: []
// }

// TODO: Refactor struct LevelXPageTable into a macro

// Page Map Level 4 Table
#[repr(C, align(4096))]
pub struct Level4PageTable {
    entries: [Level4PageTableEntry; 512],
}
page_table_level_entry!(Level4PageTableEntry);
const_assert_eq!(core::mem::align_of::<Level4PageTable>(), PAGE_SIZE);

// Page Directory Pointer Table
#[repr(C, align(4096))]
pub struct Level3PageTable {
    entries: [Level3PageTableEntry; 512],
}
page_table_level_entry!(Level3PageTableEntry);
const_assert_eq!(core::mem::align_of::<Level3PageTable>(), PAGE_SIZE);

// Page Directory Table
#[repr(C, align(4096))]
pub struct Level2PageTable {
    entries: [Level2PageTableEntry; 512],
}
page_table_level_entry!(Level2PageTableEntry);
const_assert_eq!(core::mem::align_of::<Level2PageTable>(), PAGE_SIZE);

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}
const_assert_eq!(core::mem::align_of::<PageTable>(), PAGE_SIZE);

// Page table entry layout (x86_64):
// 0        present
// 1        writable
// 2        userspace accessible
// 3        write-through (no write caching)
// 4        disable cache
// 5        accessed
// 6        dirty (written)
// 7        PAT / reserved (0)
// 8        global, CR4 PGE bit must be set
// 9:11     ignored
// 12:51    physical address
// 52:62    reserved (0)
// 63       no execute / reserved (0)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    page_table_entry_bit!(present, set_present, 0);

    page_table_entry_bit!(writable, set_writable, 1);

    page_table_entry_bit!(user, set_user, 2);

    page_table_entry_bit!(writethrough, set_writethrough, 3);

    page_table_entry_bit!(disable_cache, set_disable_cache, 4);

    page_table_entry_bit!(accessed, set_accessed, 5);

    page_table_entry_bit!(dirty, set_dirty, 6);

    page_table_entry_bit!(pat, set_pat, 7);

    page_table_entry_bit!(global, set_global, 8);

    page_table_entry_bit!(no_execute, set_no_execute, 63);

    // TODO: tests
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from(((self.0 >> 12) & ((1_u64 << 40) - 1)) << 12)
    }

    pub fn set_address(&mut self, value: PhysicalAddress) {
        let value: u64 = value.0 as u64 >> 12;
        let mask = ((1 << 40) - 1) << 12;
        self.0 = (self.0 & !mask) | ((value << 12) & mask);
    }
}

pub trait PageMapLevel {}

impl PageMapLevel for PageTable {}
impl PageMapLevel for Level2PageTable {}
impl PageMapLevel for Level3PageTable {}
impl PageMapLevel for Level4PageTable {}

macro_rules! page_table_entry_bit {
    ($id:ident, $set_id:ident, $bit:expr) => {
        pub fn $id(&self) -> bool {
            (self.0 & 1 << $bit) != 0
        }

        pub fn $set_id(&mut self, value: bool) {
            let mask = 1 << $bit;
            let value = value as u64;
            self.0 = (self.0 & !mask) | ((0_u64.wrapping_sub(value)) & mask);
        }
    };
}
use page_table_entry_bit;

macro_rules! page_table_level_entry {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct $name(u64);

        impl $name {
            page_table_entry_bit!(present, set_present, 0);

            page_table_entry_bit!(writable, set_writable, 1);

            page_table_entry_bit!(user, set_user, 2);

            page_table_entry_bit!(writethrough, set_writethrough, 3);

            page_table_entry_bit!(disable_cache, set_disable_cache, 4);

            page_table_entry_bit!(accessed, set_accessed, 5);

            page_table_entry_bit!(dirty, set_dirty, 6);

            page_table_entry_bit!(page_size, set_page_size, 7);

            page_table_entry_bit!(global, set_global, 8);

            page_table_entry_bit!(no_execute, set_no_execute, 63);

            pub fn address(&self) -> u64 {
                (self.0 >> 12) & ((1_u64 << 40) - 1)
            }

            pub fn set_address(&mut self, value: u64) {
                let mask = ((1 << 40) - 1) << 12;
                self.0 = (self.0 & !mask) | ((value << 12) & mask);
            }
        }
    };
}
use page_table_level_entry;
