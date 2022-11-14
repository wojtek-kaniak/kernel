use core::{fmt::Debug, ops::{Index, IndexMut}};

use static_assertions::const_assert_eq;

use crate::{common::macros::debug_assert_arg, arch::PrivilegeLevel};

use super::{InterruptHandler, Interrupt};

#[repr(C)]
pub struct Idt {
    entries: [IdtEntry; 256]
}
const_assert_eq!(core::mem::size_of::<Idt>(), 16 * 256);

impl Idt {
    pub fn new() -> Self {
        Idt {
            entries: [IdtEntry::default(); 256]
        }
    }

    pub fn load(&'static self) {
        crate::arch::intrinsics::load_idt(self);
    }

    pub fn register_handler<Handler: InterruptHandler>(&mut self) {
        type RawHandler = extern "C" fn(core::convert::Infallible) -> !;
        let vector: IdtVector = Handler::Interrupt::VECTOR;
        let handler: RawHandler = Handler::invoke;
        self[vector].set_offset(handler as usize);
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
    pub segment_selector: u16,
    data: IdtEntryData,
    offset_mid: u16,
    offset_high: u32,
    _reserved: u32,
}
const_assert_eq!(core::mem::size_of::<IdtEntry>(), 16);

impl IdtEntry {
    pub fn new(offset: usize, segment_selector: u16, ist_index: u8, gate_type: GateType, dpl: PrivilegeLevel) -> Self {
        debug_assert_arg!(ist_index, ist_index < 16, "ist_index must be less than 16");
        let offset = offset as u64;
        Self {
            offset_low: offset as u16,
            offset_mid: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            segment_selector,
            data: IdtEntryData::new(ist_index, gate_type, dpl),
            _reserved: 0
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

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct IdtEntryData(u16);
const_assert_eq!(core::mem::size_of::<IdtEntryData>(), 2);

impl IdtEntryData {
    pub fn new(ist_index: u8, gate_type: GateType, dpl: PrivilegeLevel) -> Self {
        let mut entry = IdtEntryData(0);
        entry.set_ist(ist_index);
        entry.set_gate_type(gate_type);
        entry.set_dpl(dpl);
        entry.set_present(true);
        entry
    }

    pub const fn invalid() -> Self {
        IdtEntryData(0)
    }

    pub fn ist(self) -> u8 {
        (self.0 as u8) & 0b111
    }

    pub fn set_ist(&mut self, value: u8) {
        let ist = value as u16 & 0b111;
        let mask = !(0b111_u16);
        self.0 = (self.0 & mask) | ist;
    }

    pub fn gate_type(self) -> GateType {
        GateType::from((self.0 >> 8) as u8 & 0b1111)
    }

    pub fn set_gate_type(&mut self, value: GateType) {
        let value = Into::<u8>::into(value) as u16;
        let mask = !(0b1111_u16 << 8);
        self.0 = (self.0 & mask) | (value << 8);
    }

    pub fn dpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from((self.0 >> 13) as u8 & 0b11)
    }

    pub fn set_dpl(&mut self, value: PrivilegeLevel) {
        let value: u8 = value.into();
        let value = value as u16;
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

impl From<u8> for GateType {
    fn from(value: u8) -> Self {
        GateType(value)
    }
}

impl Into<u8> for GateType {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdtVector(u8);

impl IdtVector {
    pub const INTEGER_DIVIDE_BY_ZERO: IdtVector = IdtVector(0);
    pub const DEBUG: IdtVector = IdtVector(1);
    pub const NON_MASKABLE_INTERRUPT: IdtVector = IdtVector(2);
    pub const BREAKPOINT: IdtVector = IdtVector(3);
    pub const OVERFLOW: IdtVector = IdtVector(4);
    pub const BOUND_RANGE_EXCEEDED: IdtVector = IdtVector(5);
    pub const INVALID_OPCODE: IdtVector = IdtVector(6);
    pub const DEVICE_NOT_AVAILABLE: IdtVector = IdtVector(7);
    pub const DOUBLE_FAULT: IdtVector = IdtVector(8);
    /// Unused
    pub const COPROCESSOR_SEGMENT_OVERRUN: IdtVector = IdtVector(9);
    pub const INVALID_TTS: IdtVector = IdtVector(10);
    pub const SEGMENT_NOT_PRESENT: IdtVector = IdtVector(11);
    pub const STACK_SEGMENT_FAULT: IdtVector = IdtVector(12);
    pub const GENERAL_PROTECTION: IdtVector = IdtVector(13);
    pub const PAGE_FAULT: IdtVector = IdtVector(14);
    pub const X87_FLOATING_POINT_ERROR: IdtVector = IdtVector(16);
    pub const ALIGNMENT_CHECK: IdtVector = IdtVector(17);
    pub const MACHINE_CHECK: IdtVector = IdtVector(18);
    pub const SIMD_FLOATING_POINT_EXCEPTION: IdtVector = IdtVector(19);

    /// Intel specific
    pub const VIRTUALIZATION_EXCEPTION: IdtVector = IdtVector(20);

    pub const CONTROL_PROTECTION_EXCEPTION: IdtVector = IdtVector(21);

    /// AMD specific
    pub const HYPERVISOR_INJECTION_EXCEPTION: IdtVector = IdtVector(28);
    /// AMD specific
    pub const VMM_COMMUNICATION_EXCEPTION: IdtVector = IdtVector(29);
    /// AMD specific
    pub const SECURITY_EXCEPTION: IdtVector = IdtVector(30);

    /// [0:32) - predefined interrupts \
    /// [32: 255] - software / maskable external interrupts
    pub fn is_predefined(self) -> bool {
        self.0 < 32
    }
}

impl Into<u8> for IdtVector {
    fn into(self) -> u8 {
        self.0
    }
}

impl From<u8> for IdtVector {
    fn from(value: u8) -> Self {
        IdtVector(value)
    }
}
