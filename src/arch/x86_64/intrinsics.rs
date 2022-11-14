use core::arch::asm;
use super::interrupts::idt::Idt;

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
        core::mem::transmute(result as u8)
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
    (high as u64) << 64 | (low as u64)
}

pub fn load_idt(idt: &'static Idt) {
    let idt = idt as *const Idt;
    unsafe {
        asm!(
            "lidt {}",
            in(reg) idt,
            options(nomem, preserves_flags, nostack)
        );
    }
}

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!(
                "cli",
                "hlt",
                options(noreturn)
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
