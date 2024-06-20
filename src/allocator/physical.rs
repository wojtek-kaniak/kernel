// TODO: DMA support

use core::{sync::atomic::{AtomicUsize, Ordering}, slice};

use arrayvec::ArrayVec;

use crate::{
    arch::{boot::{self, MemoryMapEntryKind}, intrinsics::atomic_bit_test_set, paging::{self, IdentityMapToken}, PhysicalAddress},
    common::{macros::{assert_arg, debug_assert_arg, token_type}, sync::InitOnce}
};

pub const FRAME_SIZE: usize = paging::PAGE_SIZE;
pub const MAX_MEMORY_REGION_COUNT: usize = 4096;

static ALLOCATOR: InitOnce<FrameAllocator> = InitOnce::new(FrameAllocator::empty());

token_type!(FrameAllocatorToken);

pub fn global_allocator(#[allow(unused_variables)] token: FrameAllocatorToken) -> &'static FrameAllocator {
    debug_assert!(ALLOCATOR.is_completed());
    // SAFETY: allocator was initialized
    unsafe { ALLOCATOR.get_unchecked() }
}

/// This function may only be called once, all subsequent calls will panic or be ignored \
/// All `MemoryMapEntryKind::Usable` entries in `memory_map` must be valid and unused
pub unsafe fn initialize(memory_map: boot::MemoryMap, identity_map_token: IdentityMapToken) -> FrameAllocatorToken {
    // best effort panic
    if ALLOCATOR.is_completed() {
        panic!("initialize called after the allocator has been initialized");
    }

    // Create a new allocator only if ALLOCATOR is uninitialized
    ALLOCATOR.initialize(|allocator| unsafe {
        allocator.fill(memory_map, identity_map_token);
    });

    unsafe {
        FrameAllocatorToken::new()
    }
}

#[derive(Debug)]
pub struct FrameAllocator {
    regions: ArrayVec<MemoryRegion, MAX_MEMORY_REGION_COUNT>,
    last_allocation_region: AtomicUsize
}

impl FrameAllocator {
    const fn empty() -> Self {
        Self {
            regions: ArrayVec::new_const(),
            last_allocation_region: AtomicUsize::new(0),
        }
    }

    /// All `MemoryMapEntryKind::Usable` entries in `memory_map` must be valid and unused
    unsafe fn fill(&mut self, memory_map: boot::MemoryMap, identity_map_token: IdentityMapToken) {
        for entry in memory_map.entries.iter().filter(|x| x.kind == MemoryMapEntryKind::Usable) {
            let region = unsafe { MemoryRegion::new(entry.base, entry.len, identity_map_token) };
            if self.regions.try_push(region).is_err() {
                // TODO: warn!("Too many memory regions")
                break;
            }
        }
    }

    pub fn allocate(&self, frame_count: usize) -> Option<PhysicalAddress> {
        let region_count = self.regions.len();
        // start_region_id % region_count = index of the first region checked
        let start_region_id = self.last_allocation_region.fetch_add(1, Ordering::SeqCst);
        for i in 0..region_count {
            // ((start_region_id % region_count) + i) % region_count = (start_region_id + i) % region_count
            if let Some(address) = self.regions[(start_region_id + i) % region_count].allocate(frame_count) {
                return Some(address);
            }
        }
        None
    }

    pub fn free(&self, address: PhysicalAddress, frame_count: usize) {
        let region_ix = self.regions.as_slice().binary_search_by(|region| {
            if region.check_if_owned(address) {
                core::cmp::Ordering::Equal
            } else if region.base < address {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Greater
            }
        }).expect("Attempted to free an invalid address");

        self.regions[region_ix].free(address, frame_count);
    }
}

#[derive(Debug)]
pub struct MemoryRegion {
    base: PhysicalAddress,
    frames_used: AtomicUsize,
    chunks: &'static [FrameBitmapChunk]
}

impl MemoryRegion {
    const MIN_FRAMES_REQUIRED: usize = 4;

    // TODO: refactor
    /// `base` and `size` must be `FRAME_SIZE` aligned \
    /// `size` must be greater than `FRAME_SIZE` \
    /// Memory in range [`base`; `base + size`) must be valid and unused
    pub unsafe fn new(base: PhysicalAddress, size: usize, identity_map_token: IdentityMapToken) -> Self {
        assert_arg!(base, base % FRAME_SIZE == 0, "Must be FRAME_SIZE aligned.");
        assert_arg!(size, size % FRAME_SIZE == 0, "Must be FRAME_SIZE aligned.");
        assert_arg!(size, size > FRAME_SIZE, "Must be greater than FRAME_SIZE.");

        // bytes per chunk
        const ALIGNMENT: usize = FRAME_SIZE * FrameBitmapChunk::BITS as usize;

        /// Returns size of a chunks array in bytes
        /// size must be a multiple of ALIGNMENT
        fn chunk_array_size(size: usize) -> usize {
            (size / ALIGNMENT) * core::mem::size_of::<FrameBitmapChunk>()
        }

        let region_end = (base + size).next_multiple_of(ALIGNMENT);

        let chunks_size = chunk_array_size(size);
        // Frames required to store the chunk array
        let chunks_size_frames = chunks_size.div_ceil(FRAME_SIZE);
        assert!(chunks_size < size);

        // Reserved frames - frames between ((base + end) | region_end)
        let end_reserved_frames = (region_end - (base + size)) / FRAME_SIZE;
        assert!(end_reserved_frames < FrameBitmapChunk::BITS as usize);
        
        let chunk_array_ptr = paging::to_virtual(base, identity_map_token).as_mut_ptr().cast::<FrameBitmapChunk>();
        let mut start_reserved_frames_left = chunks_size_frames;
        for i in 0..chunks_size {
            unsafe {
                // Reserved frames in the current chunk
                let chunk = FrameBitmapChunk::new(start_reserved_frames_left);
                start_reserved_frames_left = start_reserved_frames_left.saturating_sub(FrameBitmapChunk::BITS as usize);

                core::ptr::write_volatile(chunk_array_ptr.add(i), chunk);
            }
        }
        unsafe {
            let last_chunk = (*chunk_array_ptr.add(chunks_size - 1)).0.get_mut();
            // Set `end_reserved_frames` most significant bits to 1
            let end_reserved_bits = !((1_usize << (usize::BITS as usize - end_reserved_frames)).wrapping_sub(1));
            // `chunks_size_frames` and `end_reserved_frames` shouldn't overlap
            assert_eq!(*last_chunk & end_reserved_bits, 0);
            *last_chunk |= end_reserved_bits;
        }

        assert!(chunk_array_ptr.is_aligned());
        Self {
            base,
            frames_used: AtomicUsize::new(0),
            chunks: unsafe { slice::from_raw_parts(chunk_array_ptr, chunks_size) }
        }
    }

    fn frame_count(&self) -> usize {
        self.chunks.len() * (FrameBitmapChunk::BITS as usize)
    }

    fn frames_available(&self) -> usize {
        self.frame_count() - self.frames_used.load(Ordering::Relaxed)
    }

    /// Length in bytes
    fn len(&self) -> usize {
         self.frame_count() * FRAME_SIZE
    }

    fn end(&self) -> PhysicalAddress {
        self.base + self.len()
    }

    pub fn allocate(&self, frame_count: usize) -> Option<PhysicalAddress> {
        if frame_count > usize::BITS as usize {
            // Current implementation can't handle allocations crossing bitmap chunks
            return None;
        }
        let frame_count = frame_count as u8;
        if self.frames_available() < Self::MIN_FRAMES_REQUIRED {
            // Not enough frames available - contention too high for this region
            return None;
        }

        if frame_count == 1 {
            for (chunk_ix, chunk) in self.chunks.iter().enumerate() {
                if let Some(offset) = chunk.allocate_single() {
                    let address = (chunk_ix * FrameBitmapChunk::MEMORY_SIZE) + (offset as usize * FRAME_SIZE);
                    self.frames_used.fetch_add(1, Ordering::Relaxed); // TODO: is relaxed enough?
                    return Some(PhysicalAddress::new(address));
                }
            }
        } else {
            for (chunk_ix, chunk) in self.chunks.iter().enumerate() {
                if let Some(offset) = chunk.allocate_many(frame_count) {
                    let address = (chunk_ix * FrameBitmapChunk::MEMORY_SIZE) + (offset as usize * FRAME_SIZE);
                    self.frames_used.fetch_add(frame_count as usize, Ordering::Relaxed); // TODO: is relaxed enough?
                    return Some(PhysicalAddress::new(address));
                }
            }
        }
        None
    }

    pub fn free(&self, base: PhysicalAddress, frame_count: usize) {
        debug_assert_arg!(base, self.check_if_owned(base));

        debug_assert_arg!(frame_count, frame_count <= usize::BITS as usize);

        let chunk_ix = Self::chunk_index(self.base, base);
        let offset = (Into::<usize>::into(base) / FRAME_SIZE) % FrameBitmapChunk::BITS as usize;
        self.chunks[chunk_ix].free(offset as u8, frame_count as u8);
        self.frames_used.fetch_sub(frame_count, Ordering::Relaxed); // TODO: is relaxed enough?
    }

    pub fn check_if_owned(&self, address: PhysicalAddress) -> bool {
        address >= self.base && address < self.end()
    }

    fn chunk_index(region_base: PhysicalAddress, address: PhysicalAddress) -> usize {
        (address - region_base) / ((FrameBitmapChunk::BITS as usize) * FRAME_SIZE)
    }
}

#[repr(transparent)]
#[derive(Debug)]
struct FrameBitmapChunk(AtomicUsize);

impl FrameBitmapChunk {
    pub const BITS: u32 = usize::BITS;

    /// Size of memory covered by this chunk
    pub const MEMORY_SIZE: usize = Self::BITS as usize * FRAME_SIZE;

    pub fn new(initial_value: usize) -> Self {
        FrameBitmapChunk(AtomicUsize::new(initial_value))
    }

    pub fn allocate_single(&self) -> Option<u8> {
        if self.0.load(Ordering::SeqCst) != usize::MAX {
            for bit in 0..(usize::BITS as usize) {
                if unsafe { !atomic_bit_test_set(self.0.as_ptr(), bit) } {
                    return Some(bit as u8);
                }
            }
        }

        None
    }

    pub fn allocate_many(&self, count: u8) -> Option<u8> {
        debug_assert_arg!(count, count < usize::BITS as u8);

        let mut previous = self.0.load(Ordering::SeqCst);
        let mask = (1_usize << count).wrapping_sub(1);

        // All possible bit patterns (e.g. 0011, 0110, 1100...)
        for shift in 0..(usize::BITS as u8 - count) {
            let shifted_mask = mask << shift;
            for _ in 0..2 {
                if (!previous & shifted_mask) == shifted_mask {
                    match self.0.compare_exchange(previous, previous | shifted_mask, Ordering::SeqCst, Ordering::SeqCst) {
                        Ok(_) => return Some(shift),
                        Err(value) => {
                            previous = value;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        None
    }

    pub fn free(&self, offset: u8, count: u8) {
        assert!(count <= usize::BITS as u8);
        let mask: usize = (1_usize << count).wrapping_sub(1) << offset;

        let old = self.0.fetch_xor(mask, Ordering::SeqCst);
        debug_assert!(old & mask == mask, "Double free detected");
    }
}

impl Clone for FrameBitmapChunk {
    fn clone(&self) -> Self {
        Self(self.0.load(Ordering::Acquire).into())
    }
}
