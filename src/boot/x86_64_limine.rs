use core::mem::MaybeUninit;

use lazy_static::lazy_static;
use limine::{
    LimineBootInfoRequest, LimineFramebufferRequest, LimineHhdmRequest, LimineMmapRequest,
    LimineTerminal, LimineTerminalRequest, LimineTerminalResponse, LimineBootTimeRequest,
};
use spin::Mutex;

use crate::{allocator::physical::MAX_MEMORY_REGION_COUNT, common::{sync::UnsafeSync, time::UnixEpochTime}, arch::PhysicalAddress};

use super::{
    BootData, BootTerminalWriter, BootloaderInfo, FramebufferInfo, FramebufferList, MemoryMap,
    MemoryMapEntry, MemoryMapEntryKind,
};

static BOOTLOADER_INFO_REQUEST: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
static MMAP_REQUEST: LimineMmapRequest = LimineMmapRequest::new(0);
static HHDM: LimineHhdmRequest = LimineHhdmRequest::new(0);
static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);
static BOOT_TIME_REQUEST: LimineBootTimeRequest = LimineBootTimeRequest::new(0);

const MEMORY_MAP_BUFFER_SIZE: usize = MAX_MEMORY_REGION_COUNT;
static mut MEMORY_MAP_BUFFER: [MaybeUninit<MemoryMapEntry>; MEMORY_MAP_BUFFER_SIZE] =
    [MaybeUninit::uninit(); MEMORY_MAP_BUFFER_SIZE];

const FRAMEBUFFER_INFO_BUFFER_SIZE: usize = 1024;
static mut FRAMEBUFFER_INFO_BUFFER: [MaybeUninit<FramebufferInfo>; FRAMEBUFFER_INFO_BUFFER_SIZE] =
    [MaybeUninit::uninit(); FRAMEBUFFER_INFO_BUFFER_SIZE];

#[export_name = "_start"]
extern "C" fn limine_start() -> ! {
    let terminal_writer = BootTerminalWriter(LimineTerminalWriter::write_str);
    let bootloader_info = load_bootloader_info();
    let memory_map = load_memory_map();
    let direct_map_address = load_direct_map_base();
    let framebuffers = load_framebuffer_info();
    let boot_time = load_boot_time();

    let boot_data = BootData {
        terminal_writer,
        bootloader_info,
        memory_map,
        direct_map_base: direct_map_address,
        framebuffers,
        boot_time,
    };

    crate::boot::main(boot_data);
}

fn load_bootloader_info() -> BootloaderInfo {
    let bootloader_info = BOOTLOADER_INFO_REQUEST
        .get_response()
        .get()
        .expect("Bootloader info unavailable");
    
    BootloaderInfo {
        protocol: super::BootloaderProtocol::Limine,
        name: bootloader_info.name.to_string(),
        version: bootloader_info.version.to_string(),
    }
}

fn load_memory_map() -> MemoryMap {
    let mmap = MMAP_REQUEST
        .get_response()
        .get()
        .expect("Memory map unavailable");

    if MEMORY_MAP_BUFFER_SIZE < mmap.entry_count as usize {
        panic!(
            "Memory map too large ({} / max. {})",
            mmap.entry_count, MEMORY_MAP_BUFFER_SIZE
        );
    }

    let mut entries = mmap.entries.as_ptr().expect("Invalid memory map");

    for i in 0..mmap.entry_count as usize {
        unsafe {
            let entry = entries.read().get().expect("Invalid memory map");

            use limine::LimineMemoryMapEntryType as LimineMemType;
            MEMORY_MAP_BUFFER[i] = MaybeUninit::new(MemoryMapEntry::new(
                entry.base as usize,
                entry.len as usize,
                match entry.typ {
                    LimineMemType::AcpiNvs
                    | LimineMemType::AcpiReclaimable
                    | LimineMemType::BadMemory
                    | LimineMemType::BootloaderReclaimable
                    | LimineMemType::Framebuffer
                    | LimineMemType::Reserved => MemoryMapEntryKind::Reserved,

                    LimineMemType::KernelAndModules => MemoryMapEntryKind::Kernel,
                    LimineMemType::Usable => MemoryMapEntryKind::Usable,
                },
            ));

            entries = entries.offset(i as isize);
        }
    }

    MemoryMap {
        entries: unsafe { MaybeUninit::slice_assume_init_ref(&MEMORY_MAP_BUFFER[..mmap.entry_count as usize]) },
    }
}

fn load_direct_map_base() -> PhysicalAddress {
    let offset = HHDM.get_response()
        .get()
        .expect("Direct map unavailable")
        .offset as usize;

    offset.into()
}

fn load_framebuffer_info() -> FramebufferList {
    let fb = FRAMEBUFFER_REQUEST
        .get_response()
        .get()
        .expect("Framebuffer info unavailable");
    let mut entries = fb.framebuffers.as_ptr().expect("Invalid framebuffer info");

    if fb.framebuffer_count as usize > FRAMEBUFFER_INFO_BUFFER_SIZE {
        panic!(
            "Framebuffer list too large ({} / max. {})",
            fb.framebuffer_count, FRAMEBUFFER_INFO_BUFFER_SIZE
        );
    }

    for i in 0..fb.framebuffer_count as usize {
        unsafe {
            let limine_fb = entries.read().get().expect("Invalid framebuffer info");
            let entry = FramebufferInfo {
                address: limine_fb.address.as_ptr().expect("Invalid framebuffer info") as usize,
                bpp: limine_fb.bpp.try_into().unwrap(),
                red_mask: limine_fb.red_mask_size,
                red_shift: limine_fb.red_mask_shift,
                green_mask: limine_fb.green_mask_size,
                green_shift: limine_fb.green_mask_shift,
                blue_mask: limine_fb.blue_mask_size,
                blue_shift: limine_fb.blue_mask_shift,
                width: limine_fb.width as usize,
                height: limine_fb.height as usize,
                pitch: limine_fb.pitch as usize,
            };
            FRAMEBUFFER_INFO_BUFFER[i] = MaybeUninit::new(entry);

            entries = entries.offset(i as isize);
        }
    }

    FramebufferList {
        entries: unsafe { MaybeUninit::slice_assume_init_ref(&FRAMEBUFFER_INFO_BUFFER[..fb.framebuffer_count as usize]) },
    }
}

fn load_boot_time() -> UnixEpochTime {
    let time = BOOT_TIME_REQUEST.get_response().get().expect("Boot time unavailable").boot_time as u64;
    UnixEpochTime::new(time)
}

// TODO: remove UnsafeSync
lazy_static! {
    static ref TERMINAL_RESPONSE: UnsafeSync<Option<&'static LimineTerminalResponse>> =
        TERMINAL_REQUEST.get_response().get().into();

    static ref TERMINAL: Mutex<Option<&'static LimineTerminal>> =
        unsafe { TERMINAL_RESPONSE.get() }
            .and_then(|x| x.terminals().and_then(|x| x.first()))
            .into();
}

/// Warning: Not thread safe
pub struct LimineTerminalWriter;

impl LimineTerminalWriter {
    fn write_str(str: &str) -> core::fmt::Result {
        use core::fmt::Error;

        let writer = unsafe { TERMINAL_RESPONSE.get().ok_or(Error)?.write().ok_or(Error)? };
        let terminal_lock = TERMINAL.lock();
        writer(terminal_lock.ok_or(Error)?, str);

        Ok(())
    }
}
