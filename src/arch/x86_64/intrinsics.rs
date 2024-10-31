use core::{arch::asm, mem::MaybeUninit};
use super::interrupts::idt::IdtRegister;

/// # Safety
/// `value` pointer must be writable
pub unsafe fn atomic_bit_test_set(value: *mut usize, index: usize) -> bool {
    let result: u32;
    unsafe {
        asm!(
            "xor {o:e}, {o:e}",
            "lock bts [{val}], {ix}",
            "setc {o:l}",
            o = out(reg) result, val = in(reg) value, ix = in(reg) index,
            options(nostack)
        );

        // This is safe, as only the lowest bit can be set
        result as u8 != 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CpuidResult {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

impl From<(u32, u32, u32, u32)> for CpuidResult {
    fn from(value: (u32, u32, u32, u32)) -> Self {
        let (eax, ebx, ecx, edx) = value;
        Self {
            eax, ebx, ecx, edx
        }
    }
}

impl From<CpuidResult> for (u32, u32, u32, u32) {
    fn from(value: CpuidResult) -> Self {
        (value.eax, value.ebx, value.ecx, value.edx)
    }
}

// TODO: should it be unsafe?
/// # Safety
/// The request must be valid
pub unsafe fn cpuid(eax: MaybeUninit<u32>, ecx: MaybeUninit<u32>) -> CpuidResult {
    // TODO: verify if cpuid is available
    let (eax_in, ecx_in) = (eax, ecx);
    let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
    unsafe {
        asm!(
            "mov {1:e}, ebx",
            "cpuid",
            // ebx can't be used in inline asm, see: https://github.com/rust-lang/rust/pull/84658
            "mov {0:e}, ebx",
            "mov ebx, {1:e}",
            out(reg) ebx,
            out(reg) _,
            inout("eax") eax_in => eax,
            inout("ecx") ecx_in => ecx,
            out("edx") edx,
            options(nostack, nomem, preserves_flags)
        );
    }
    
    (eax, ebx, ecx, edx).into()
}

pub mod cpuid {
    use core::mem::MaybeUninit;

    use super::cpuid;

    pub fn brand() -> [u8; 12] {
        let res = unsafe {
            cpuid(MaybeUninit::new(0), MaybeUninit::uninit())
        };

        let mut out = [0; 12];

        out[0..4].copy_from_slice(&res.ebx.to_le_bytes());
        out[4..8].copy_from_slice(&res.edx.to_le_bytes());
        out[8..].copy_from_slice(&res.ecx.to_le_bytes());

        out
    }
}

pub fn time_stamp_counter() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") low, out("edx") high,
            options(nostack, nomem, preserves_flags)
        );
    }
    (high as u64) << 32 | (low as u64)
}

/// # Safety
/// The referenced IDT must have the correct lifetime
/// (be valid until replaced).
pub unsafe fn load_idt(idt: IdtRegister) {
    unsafe {
        asm!(
            "lidt [{}]",
            in(reg) &idt as *const _,
            options(readonly, preserves_flags, nostack)
        );
    }
}

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!(
                "cli",
                "hlt",
            )
        }
    }
}

macro_rules! read_cr {
    ($register:literal) => {{
        let result: u64;
        ::core::arch::asm!(
            concat!("mov {}, cr", $register),
            out(reg) result,
            options(nomem, preserves_flags, nostack)
        );
        result
    }}
}
pub(super) use read_cr;

macro_rules! write_cr {
    ($register:literal, $value:expr) => {{
        let value: u64 = $value;
        ::core::arch::asm!(
            concat!("mov cr", $register, ", {}"),
            in(reg) value,
            options(nomem, preserves_flags, nostack)
        );
        value
    }}
}
pub(super) use write_cr;
