use core::fmt::{Debug, Display, Write};

use itertools::Itertools;

use crate::{common::{macros::debug_assert_arg, time::UnixEpochTime}, arch::PhysicalAddress};

#[cfg(all(target_arch = "x86_64", feature = "limine"))]
pub mod x86_64_limine;

pub static mut BOOT_TERMINAL_WRITER: Option<BootTerminalWriter> = Option::None;

pub fn main(data: BootData) -> ! {
    initialize_terminal(data.terminal_writer);
    crate::allocator::physical::initialize(data.memory_map);
    
    todo!()
    //unreachable!();
}

fn initialize_terminal(writer: BootTerminalWriter) {
    unsafe { BOOT_TERMINAL_WRITER = Some(writer) };
}

#[derive(Clone, Copy, Debug)]
pub struct BootData {
    pub bootloader_info: BootloaderInfo,
    pub memory_map: MemoryMap,
    pub direct_map_base: PhysicalAddress,
    pub framebuffers: FramebufferList,
    pub terminal_writer: BootTerminalWriter,
    /// Unix epoch time on boot
    pub boot_time: UnixEpochTime,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct BootTerminalWriter(fn(&str) -> core::fmt::Result);

impl Debug for BootTerminalWriter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(stringify!(BootTerminalWriter)).finish()
    }
}

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
    pub fn new(entries: &'static [MemoryMapEntry]) -> Self {
        debug_assert_arg!(
            entries,
            entries.len() > 0,
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
                .any(|(prev, next)| prev.base + prev.len > next.base),
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
    pub base: usize,
    pub len: usize,
    pub kind: MemoryMapEntryKind,
}

impl MemoryMapEntry {
    pub fn new(base: usize, len: usize, kind: MemoryMapEntryKind) -> Self {
        MemoryMapEntry { base, len, kind }
    }

    pub fn end(self) -> usize {
        self.base + self.len
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryMapEntryKind {
    Usable,
    Kernel,
    Reserved,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FramebufferList {
    pub entries: &'static [FramebufferInfo],
}

#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    /// Linear framebuffer (virtual) address
    pub address: usize,
    /// Bits per pixel
    pub bpp: u8,
    pub red_mask: u8,
    pub red_shift: u8,
    pub green_mask: u8,
    pub green_shift: u8,
    pub blue_mask: u8,
    pub blue_shift: u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
}

// TODO: refactor into generic logger with fb/serial/etc. support
#[macro_export]
macro_rules! boot_print {
    ($($arg:tt)*) => (_ = core::fmt::Write::write_fmt(
        unsafe { crate::boot::BOOT_TERMINAL_WRITER }.as_mut().expect("Boot terminal unavailable"), format_args!($($arg)*)
    ));
}

#[macro_export]
macro_rules! boot_println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => (crate::boot_print!("{}\n", format_args!($($arg)*)));
}
