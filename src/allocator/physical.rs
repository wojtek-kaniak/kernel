// TODO: physical page allocator

use core::sync::atomic::{AtomicUsize, Ordering, AtomicBool};

use crate::{
    boot,
    arch::{paging::{self, IdentityMapToken}, PhysicalAddress, intrinsics::atomic_bit_test_set},
    common::{macros::{debug_assert_arg, token_type}, collections::FixedSizeVec}
};

pub const FRAME_SIZE: usize = paging::PAGE_SIZE;
pub const MAX_MEMORY_REGION_COUNT: usize = 4096;

// Initialized once, frequently read
static mut ALLOCATOR: Option<FrameAllocator> = None;
static ALLOCATOR_INITIALIZED: AtomicBool = AtomicBool::new(false);

token_type!(FrameAllocatorToken);

pub fn global_allocator(#[allow(unused_variables)] token: FrameAllocatorToken) -> &'static FrameAllocator {
    unsafe {
        debug_assert!(ALLOCATOR.is_some());
        ALLOCATOR.as_ref().unwrap_unchecked()
    }
}

/// This function may only be called once
pub fn initialize(memory_map: boot::MemoryMap) -> FrameAllocatorToken {
    ALLOCATOR_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .expect("Frame allocator already initialized");
    
    let allocator = FrameAllocator::new(memory_map);
    unsafe {
        debug_assert!(ALLOCATOR.is_none());
        ALLOCATOR = Some(allocator);
        FrameAllocatorToken::new()
    }
}

#[derive(Debug)]
pub struct FrameAllocator {
    regions: FixedSizeVec<MemoryRegion, MAX_MEMORY_REGION_COUNT>
}

impl FrameAllocator {
    fn new(memory_map: boot::MemoryMap) -> FrameAllocator {
        _ = memory_map;
        todo!()
    }

    pub fn allocate(&self, frame_count: usize) -> Option<PhysicalAddress> {
        _ = frame_count;
        todo!()
    }

    // pub fn allocate_dma(&self, frame_count: usize) -> Option<PhysicalAddress> {
    //     _ = frame_count;
    //     todo!()
    // }

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

// repr(bool)?
// https://internals.rust-lang.org/t/feature-request-repr-bool/16974
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FrameState {
    Free = 0,
    Used = 1
}

impl Into<bool> for FrameState {
    fn into(self) -> bool {
        self == FrameState::Used
    }
}

impl From<bool> for FrameState {
    fn from(value: bool) -> Self {
        if value { FrameState::Used }
        else { FrameState::Free }
    }
}

#[derive(Debug)]
struct MemoryRegion {
    base: PhysicalAddress,
    frames_used: AtomicUsize,
    chunks: &'static [FrameBitmapChunk]
}

impl MemoryRegion {
    const MIN_FRAMES_REQUIRED: usize = 4;

    /// base should be FRAME_SIZE aligned
    pub fn new(base: PhysicalAddress, size: usize, identity_map: IdentityMapToken) -> Self {
        /// size must be a multiple of FrameBitmapChunk::BITS
        fn chunk_array_size(size: usize) -> usize {
            core::mem::size_of::<FrameBitmapChunk>() * (size / FrameBitmapChunk::BITS as usize)
        }

        let base: usize = base.into();

        const ALIGNMENT: usize = FRAME_SIZE * FrameBitmapChunk::BITS as usize;
        // previous multiple of ALIGNMENT
        let region_base = base / ALIGNMENT * ALIGNMENT;
        let region_end = (base + size).next_multiple_of(ALIGNMENT);
        let region_size = region_end - region_base;

        let start_reserved_count = (base % ALIGNMENT).div_ceil(FRAME_SIZE);
        let end_reserved_count = (region_end - (base + size)).div_ceil(FRAME_SIZE);

        // start_reserved_count bits set to 1
        let start_bits = 1_usize << start_reserved_count - 1;
        // end_reserved_count bits set to 1
        let end_bits = 1_usize << end_reserved_count - 1;

        let chunks_size = chunk_array_size(region_size);
        debug_assert!(chunks_size > region_size);
        // TODO: set next chunk_size bits to 1

        todo!()
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
        return None;
    }

    pub fn free(&self, base: PhysicalAddress, frame_count: usize) {
        debug_assert_arg!(base, self.check_if_owned(base));

        debug_assert_arg!(frame_count, frame_count <= usize::BITS as usize);

        let chunk_ix = Self::chunk_index(self.base, base);
        let offset = (Into::<usize>::into(base) / FRAME_SIZE) % FrameBitmapChunk::BITS as usize;
        self.chunks[chunk_ix].free(offset as u8, frame_count as u8);
        self.frames_used.fetch_sub(frame_count as usize, Ordering::Relaxed); // TODO: is relaxed enough?
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

    pub fn allocate_single(&self) -> Option<u8> {
        if self.0.load(Ordering::SeqCst) != usize::MAX {
            for bit in 0..(usize::BITS as usize) {
                if !unsafe { atomic_bit_test_set(self.0.as_mut_ptr(), bit) } {
                    return Some(bit as u8);
                }
            }
        }

        return None;
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

        return None;
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
