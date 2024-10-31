use core::fmt::{Debug, Display, Write};
use crate::{arch::{interrupts::idt::Idt, processor::Processor, PhysicalAddress, PrivilegeLevel, SegmentIndex, SegmentSelector, VirtualAddress}, common::{macros::{assert_arg, debug_assert_arg}, time::UnixEpochTime}};

use self::logo::LogoScreen;

use super::{devices::framebuffer::{Framebuffer, FramebufferInfo, FramebufferList, RawFramebuffer}, interrupts::{define_interrupt_handler, StackFrame}, intrinsics::{cpuid, halt}};

mod logo;

#[cfg(all(target_arch = "x86_64", feature = "limine"))]
mod x86_64_limine;

pub static mut BOOT_TERMINAL_WRITER: Option<BootTerminalWriter> = Option::None;

pub fn main(data: BootData) -> ! {
    initialize_terminal(data.terminal_writer);

    print_cpu_brand();
    // halt();

    // TODO: initialize arch::devices::framebuffer instead
    // let framebuffer = data.framebuffers.entries.first().and_then(|&fb| unsafe { RawFramebuffer::new(fb).ok() });
    // if let Some(framebuffer) = framebuffer {
    //     LogoScreen::new(Framebuffer::new(&framebuffer));
    // }

    let identity_map_token = crate::arch::paging::initialize_identity_map(data.identity_map_base);
    // TODO: fix memory map loading
    // halt();
    unsafe {
        crate::allocator::physical::initialize(data.memory_map, identity_map_token);
    }

    boot_println!("time: {}", data.boot_time.millis());
    boot_println!("boot: {:?}", data.terminal_writer);

    let mut proc = Processor {
        idt: Idt::new(),
    };
    
    proc.idt.swap_handler::<InvalidOpcodeTest>(
        SegmentSelector::new(
            SegmentIndex::new(5),
            crate::arch::TableIndicator::Gdt,
            PrivilegeLevel::KERNEL
        )
    );
    
    unsafe {
        Idt::load(&proc.idt);
    }

    // unsafe { core::arch::asm!("ud2") }

    halt();
    
    // todo!()
    //unreachable!();
}

fn breakpoint() -> ! {
    loop {
        boot_println!("test");
        unsafe {
            core::arch::asm!("pause");
        }
    }
}

define_interrupt_handler! {
    handler InvalidOpcodeTest(stack_frame: &StackFrame) for super::interrupts::InvalidOpcode {
        breakpoint()
    }
}

fn initialize_terminal(writer: BootTerminalWriter) {
    unsafe { BOOT_TERMINAL_WRITER = Some(writer) };
}

fn print_cpu_brand() {
    let brand = cpuid::brand();
    let brand = core::str::from_utf8(&brand).unwrap_or("[invalid UTF-8]");
    boot_println!("CPU brand string: {brand}");
}

#[derive(Clone, Copy, Debug)]
pub struct BootData {
    pub bootloader_info: BootloaderInfo,
    pub memory_map: MemoryMap,
    pub identity_map_base: PhysicalAddress,
    pub framebuffers: FramebufferList,
    pub terminal_writer: BootTerminalWriter,
    /// Unix epoch time on boot
    pub boot_time: UnixEpochTime,
    pub kernel_address: (PhysicalAddress, VirtualAddress),
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct BootTerminalWriter(fn(&str) -> core::fmt::Result);

impl Write for BootTerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0(s)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BootloaderInfo {
    pub protocol: BootloaderProtocol,
    pub name: Option<&'static str>,
    pub version: Option<&'static str>,
}

impl Display for BootloaderInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}",
            self.name.unwrap_or_default(),
            self.version.unwrap_or_default()
        ))
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum BootloaderProtocol {
    Limine,
}

impl Display for BootloaderProtocol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryMap {
    pub entries: &'static [MemoryMapEntry],
}

impl MemoryMap {
    /// # Safety
    /// All entries must be sorted by base and valid,
    /// all `MemoryMapEntryKind::Usable` entries must not be overlapping
    pub unsafe fn new(entries: &'static [MemoryMapEntry]) -> Self {
        #[cfg(debug_assertions)]
        use itertools::Itertools;

        assert_arg!(
            entries,
            !entries.is_empty(),
            "Memory map contains no elements"
        );
        debug_assert_arg!(
            entries,
            entries.is_sorted_by_key(|x| x.base),
            "Memory map entries not sorted by base"
        );
        debug_assert_arg!(
            entries,
            entries
                .iter()
                .filter(|x| x.kind == MemoryMapEntryKind::Usable)
                .tuple_windows()
                .all(|(prev, next)| prev.base + prev.len <= next.base),
            "Usable memory map entries overlapping"
        );

        MemoryMap { entries }
    }
}

impl IntoIterator for MemoryMap {
    type Item = &'static MemoryMapEntry;

    type IntoIter = core::slice::Iter<'static, MemoryMapEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryMapEntry {
    /// Base physical address
    pub base: PhysicalAddress,
    pub len: usize,
    pub kind: MemoryMapEntryKind,
}

impl MemoryMapEntry {
    pub fn new(base: PhysicalAddress, len: usize, kind: MemoryMapEntryKind) -> Self {
        MemoryMapEntry { base, len, kind }
    }

    pub fn end(self) -> PhysicalAddress {
        self.base + self.len
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryMapEntryKind {
    Usable,
    Kernel,
    Reserved,
}

// TODO: refactor into generic logger with fb/serial/etc. support
macro_rules! boot_print {
    ($($arg:tt)*) => (_ = core::fmt::Write::write_fmt(
        unsafe { crate::arch::boot::BOOT_TERMINAL_WRITER }.as_mut().expect("Boot terminal unavailable"), format_args!($($arg)*)
    ));
}
pub(crate) use boot_print;

macro_rules! boot_println {
    () => (crate::arch::boot::print!("\n"));
    ($($arg:tt)*) => (crate::arch::boot::boot_print!("{}\n", format_args!($($arg)*)));
}
pub(crate) use boot_println;
