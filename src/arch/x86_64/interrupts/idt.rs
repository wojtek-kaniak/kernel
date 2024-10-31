use core::{fmt::Debug, ops::{Index, IndexMut}};

use static_assertions::{const_assert, const_assert_eq};

use crate::{arch::{PrivilegeLevel, SegmentSelector}, common::macros::assert_arg};

use super::{InterruptHandler, Interrupt};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Idt {
    entries: [IdtEntry; 256],
}
const_assert_eq!(core::mem::size_of::<Idt>(), 16 * 256);
const_assert!(core::mem::size_of::<Idt>() <= u16::MAX as usize);

impl Idt {
    pub const fn new() -> Self {
        Idt {
            entries: [IdtEntry::empty(); 256]
        }
    }

    /// # Safety
    /// The referenced IDT must have the correct lifetime
    /// (be valid until replaced).
    pub unsafe fn load(idt: *const Idt) {
        const_assert!(core::mem::size_of::<Idt>() <= u16::MAX as usize);

        let reg = IdtRegister {
            base: idt,
            limit: core::mem::size_of::<Idt>() as u16,
        };

        unsafe {
            crate::arch::intrinsics::load_idt(reg);
        }
    }

    /// Registers a new interrupt handler and returns the previous if present
    pub fn swap_handler<Handler: InterruptHandler>(&mut self, segment_descriptor: SegmentSelector) -> Option<IdtEntry> {
        type RawHandler = unsafe extern "C" fn() -> !;

        let vector: IdtVector = Handler::Interrupt::VECTOR;
        let handler: RawHandler = Handler::invoke;
        
        let old = self[vector];

        self[vector] = IdtEntry::new(
            handler as usize,
            segment_descriptor,
            IstIndex::UNUSED,
            GateType::TRAP, // TODO:
            PrivilegeLevel::USERSPACE,
        );

        if old.data.present() {
            Some(old)
        } else {
            None
        }
    }
}

impl Default for Idt {
    fn default() -> Self {
        Idt::new()
    }
}

impl Index<IdtVector> for Idt {
    type Output = IdtEntry;

    fn index(&self, index: IdtVector) -> &Self::Output {
        unsafe {
            self.entries.get_unchecked(index.0 as usize)
        }
    }
}

// Should this be unsafe?
impl IndexMut<IdtVector> for Idt {
    fn index_mut(&mut self, index: IdtVector) -> &mut Self::Output {
        unsafe {
            self.entries.get_unchecked_mut(index.0 as usize)
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IdtEntry {
    offset_low: u16,
    pub segment: SegmentSelector,
    pub data: IdtEntryData,
    offset_mid: u16,
    offset_high: u32,
    _reserved: u32,
}
const_assert_eq!(core::mem::size_of::<IdtEntry>(), 16);

impl IdtEntry {
    pub fn new(offset: usize, segment: SegmentSelector, ist_index: IstIndex, gate_type: GateType, dpl: PrivilegeLevel) -> Self {
        let offset = offset as u64;
        Self {
            offset_low: offset as u16,
            offset_mid: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            segment,
            data: IdtEntryData::new(ist_index, gate_type, dpl),
            _reserved: 0
        }
    }

    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            segment: SegmentSelector::null(),
            data: IdtEntryData(0),
            offset_mid: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }

    pub fn offset(self) -> usize {
        (self.offset_low as u64 | (self.offset_mid as u64) << 16 | (self.offset_high as u64) << 32) as usize
    }

    pub fn set_offset(&mut self, value: usize) {
        self.offset_low = value as u16;
        self.offset_mid = (value >> 16) as u16;
        self.offset_high = (value >> 32) as u32;
    }
}


/// IDT entry data
///
/// # Fields
/// [0, 2]   - IST index - offset into the Interrupt Stack Table
/// [3, 7]   - reserved
/// [8, 11]  - Gate type (Interrupt / Trap)
/// [12, 12] - must be 0
/// [13, 14] - DPL - privilage level required to invoke this software interrupt
/// [15, 15] - Present bit
#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct IdtEntryData(u16);
const_assert_eq!(core::mem::size_of::<IdtEntryData>(), 2);

impl IdtEntryData {
    pub fn new(ist_index: IstIndex, gate_type: GateType, dpl: PrivilegeLevel) -> Self {
        let mut entry = IdtEntryData(0);
        entry.set_ist(ist_index);
        entry.set_gate_type(gate_type);
        entry.set_dpl(dpl);
        entry.set_present(true);
        entry
    }

    pub const fn empty() -> Self {
        IdtEntryData(0)
    }

    pub fn ist(self) -> IstIndex {
        IstIndex((self.0 as u8) & 0b111)
    }

    pub fn set_ist(&mut self, value: IstIndex) {
        const MASK: u16 = !0b111_u16;

        self.0 = (self.0 & MASK) | value.0 as u16;
    }

    pub fn gate_type(self) -> GateType {
        GateType((self.0 >> 8) as u8 & 0b1111)
    }

    pub fn set_gate_type(&mut self, value: GateType) {
        const MASK: u16 = !(0b1111_u16 << 8);

        let value = value.0 as u16;
        self.0 = (self.0 & MASK) | (value << 8);
    }

    pub fn dpl(self) -> PrivilegeLevel {
        PrivilegeLevel::new((self.0 >> 13) as u8 & 0b11)
    }

    pub fn set_dpl(&mut self, value: PrivilegeLevel) {
        let value = value.0 as u16;
        let mask = !(0b11_u16 << 13);
        self.0 = (self.0 & mask) | (value << 13);
    }

    pub fn present(self) -> bool {
        self.0 >> 15 != 0
    }

    pub fn set_present(&mut self, value: bool) {
        let value = value as u16;
        self.0 = (self.0 & !(1_u16 << 15)) | (value << 15);
    }
}

impl Debug for IdtEntryData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(stringify!(IdtEntryData))
            .field("ist", &self.ist())
            .field("gate_type", &self.gate_type())
            .field("dpl", &self.dpl())
            .field("present", &self.present())
            .finish()
    }
}

/// A 3 bit offset into the Interrupt Stack Table, unused if 0
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IstIndex(u8);

impl IstIndex {
    pub const UNUSED: Self = IstIndex(0);

    /// Must be valid (3 bit)
    pub fn new(value: u8) -> Self {
        assert_arg!(value, value < 8);

        IstIndex(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateType(u8);

impl GateType {
    pub const INVALID: GateType = GateType(0);
    pub const INTERRUPT: GateType = GateType(0xE);
    pub const TRAP: GateType = GateType(0xF);

    /// Checks if this gate type is valid on x86_64
    pub fn is_valid(self) -> bool {
        self == Self::INTERRUPT || self == Self::TRAP
    }
}

impl From<GateType> for u8 {
    fn from(val: GateType) -> Self {
        val.0
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdtRegister {
    limit: u16,
    base: *const Idt,
}
const_assert_eq!(core::mem::size_of::<IdtRegister>(), 10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdtVector(u8);

macro_rules! define_idt_vectors {
    (
        $(
            $(#[$attr:meta])*
            $name:ident = $value:expr,
        )*
    ) => {
        $(
            $(
                #[$attr]
            )*
            pub const $name: IdtVector = IdtVector($value);
        )*
    };
}
use define_idt_vectors;

impl IdtVector {
    define_idt_vectors! {
        INTEGER_DIVIDE_BY_ZERO = 0,
        DEBUG = 1,
        NON_MASKABLE_INTERRUPT = 2,
        BREAKPOINT = 3,
        OVERFLOW = 4,
        BOUND_RANGE_EXCEEDED = 5,
        INVALID_OPCODE = 6,
        DEVICE_NOT_AVAILABLE = 7,
        DOUBLE_FAULT = 8,

        /// Unused
        COPROCESSOR_SEGMENT_OVERRUN = 9,

        INVALID_TTS = 10,
        SEGMENT_NOT_PRESENT = 11,
        STACK_SEGMENT_FAULT = 12,
        GENERAL_PROTECTION = 13,
        PAGE_FAULT = 14,
        X87_FLOATING_POINT_ERROR = 16,
        ALIGNMENT_CHECK = 17,
        MACHINE_CHECK = 18,
        SIMD_FLOATING_POINT_EXCEPTION = 19,

        /// Intel specific
        VIRTUALIZATION_EXCEPTION = 20,

        CONTROL_PROTECTION_EXCEPTION = 21,

        /// AMD specific
        HYPERVISOR_INJECTION_EXCEPTION = 28,

        /// AMD specific
        VMM_COMMUNICATION_EXCEPTION = 29,

        /// AMD specific
        SECURITY_EXCEPTION = 30,
    }

    /// [0:32) - predefined interrupts \
    /// [32: 255] - software / maskable external interrupts
    pub fn is_predefined(self) -> bool {
        self.0 < 32
    }
}

impl From<IdtVector> for u8 {
    fn from(val: IdtVector) -> Self {
        val.0
    }
}

impl From<u8> for IdtVector {
    fn from(value: u8) -> Self {
        IdtVector(value)
    }
}
