#![allow(dead_code)] // TODO (WIP)
mod structs;

use spin::Once;
use structs::*;
pub use structs::PAGE_SIZE;

use crate::{
    allocator::physical::FrameAllocator,
    arch::{intrinsics::write_cr, PhysicalAddress, VirtualAddress},
    common::macros::{token_from, token_type}
};

use super::intrinsics::read_cr;

// u64 on private api
// usize on public api (same public interface on various architectures)

static IDENTITY_MAP_BASE: Once<PhysicalAddress> = Once::new();

const CR3_ADDRESS_MASK: u64 = 0xFFFFFFFFFF000;

token_type!{
    /// Asserts that paging is set up
    PagingToken
}

token_type!{
    /// Asserts that the identity map is initialized
    IdentityMapToken
}

// TODO
token_from!(PagingToken, IdentityMapToken);

/// This function may only be called once, all subsequent calls will panic or be ignored
pub fn initialize_identity_map(identity_map_base: PhysicalAddress) -> IdentityMapToken {
    // best effort panic
    if IDENTITY_MAP_BASE.is_completed() {
        panic!("Identity map already initialized.");
    }

    IDENTITY_MAP_BASE.call_once(|| identity_map_base);

    unsafe {
        IdentityMapToken::new()
    }
}

pub fn initialize(frame_allocator: FrameAllocator, identity_map: IdentityMapToken) {
    let _ = (identity_map, frame_allocator);
    todo!()
}

/// Returns corresponding virtual address from the identity mapping
pub fn to_virtual(address: PhysicalAddress, token: IdentityMapToken) -> VirtualAddress {
    (Into::<usize>::into(identity_map_base(token)) + address.0).into()
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

fn identity_map_base(#[allow(unused_variables)] token: IdentityMapToken) -> PhysicalAddress {
    debug_assert!(IDENTITY_MAP_BASE.is_completed());
    unsafe {
        *IDENTITY_MAP_BASE.get_unchecked()
    }
}

fn get_kernel_map_virtual_address<T: PageMapLevel>(physical_address: PhysicalAddress, token: IdentityMapToken) -> *const T {
    let identity_map: usize = identity_map_base(token).into();
    let physical_address: usize = physical_address.into();
    (identity_map + physical_address) as *const T
}
