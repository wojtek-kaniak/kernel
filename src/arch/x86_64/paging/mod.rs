#![allow(dead_code)] // TODO (WIP)
mod structs;
use core::mem::MaybeUninit;

use structs::*;
pub use structs::PAGE_SIZE;

use crate::{allocator::physical::FrameAllocator, common::macros::{token_type, token_from_unsafe}, arch::intrinsics::write_cr};

use super::{PhysicalAddress, intrinsics::read_cr};

// u64 on private api
// usize on public api (same public interface on different architectures)

static mut IDENTITY_MAP_BASE: MaybeUninit<u64> = MaybeUninit::uninit();

const CR3_ADDRESS_MASK: u64 = 0xFFFFFFFFFF000;

token_type!(PagingToken);

token_type!(IdentityMapToken);

token_from_unsafe!(PagingToken, IdentityMapToken);

pub fn initialize_identity_map(identity_map_base: usize) -> IdentityMapToken {
    unsafe { IDENTITY_MAP_BASE.write(identity_map_base as u64); }
    IdentityMapToken
}

pub fn initialize(identity_map: IdentityMapToken, frame_allocator: FrameAllocator) {
    let _ = (identity_map, frame_allocator);
    todo!()
}

/// Returns corresponding virtual address from the identity mapping
pub fn to_virtual(token: IdentityMapToken, address: PhysicalAddress) -> usize {
    identity_map_base(token) as usize + address.0
}

unsafe fn read_pml4_address() -> PhysicalAddress {
    unsafe {
        (read_cr!(3) & CR3_ADDRESS_MASK).into()
    }
}

unsafe fn write_pml4_address(address: PhysicalAddress) {
    unsafe {
        write_cr!(3, address.0 as u64 & CR3_ADDRESS_MASK);
    }
}

fn identity_map_base(token: IdentityMapToken) -> u64 {
    let _ = token;
    unsafe { IDENTITY_MAP_BASE.assume_init() }
}

fn get_kernel_map_virtual_address<T: PageMapLevel>(token: IdentityMapToken, physical_address: PhysicalAddress) -> *const T {
    (physical_address.0 as u64 + identity_map_base(token)) as *const T
}
