use core::fmt::{Debug, Display, Write};
use crate::{common::{macros::{debug_assert_arg, assert_arg}, time::UnixEpochTime}, arch::{PhysicalAddress, VirtualAddress}};

use self::logo::LogoScreen;

use super::{devices::framebuffer::{FramebufferInfo, FramebufferList, RawFramebuffer, Framebuffer}, intrinsics::halt};

mod logo;

#[cfg(all(target_arch = "x86_64", feature = "limine"))]
mod x86_64_limine;

pub static mut BOOT_TERMINAL_WRITER: Option<BootTerminalWriter> = Option::None;

pub fn main(data: BootData) -> ! {
    initialize_terminal(data.terminal_writer);

    // TODO: initialize arch::devices::framebuffer instead
    let framebuffer = data.framebuffers.entries.first().map(|&fb| unsafe { RawFramebuffer::new(fb).ok() }).flatten();
    if let Some(framebuffer) = framebuffer {
        LogoScreen::new(Framebuffer::new(&framebuffer));
    }

    let identity_map_token = crate::arch::paging::initialize_identity_map(data.identity_map_base);
    // TODO: fix memory map loading
    halt();
    unsafe {
        crate::allocator::physical::initialize(data.memory_map, identity_map_token);
    }
    
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
    pub identity_map_base: PhysicalAddress,
    pub framebuffers: FramebufferList,
    pub terminal_writer: BootTerminalWriter,
    /// Unix epoch time on boot
    pub boot_time: UnixEpochTime,
    pub kernel_address: (PhysicalAddress, VirtualAddress),
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
    /// All entries must be sorted by base \
    /// `entries.len()` must be greater than 0
    /// All entries must be valid, all `MemoryMapEntryKind::Usable` entries must be usable
    pub unsafe fn new(entries: &'static [MemoryMapEntry]) -> Self {
        #[cfg(debug_assertions)]
        use itertools::Itertools;

        assert_arg!(
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
#[macro_export]
macro_rules! boot_print {
    ($($arg:tt)*) => (_ = core::fmt::Write::write_fmt(
        unsafe { crate::arch::boot::BOOT_TERMINAL_WRITER }.as_mut().expect("Boot terminal unavailable"), format_args!($($arg)*)
    ));
}

#[macro_export]
macro_rules! boot_println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => (crate::boot_print!("{}\n", format_args!($($arg)*)));
}
