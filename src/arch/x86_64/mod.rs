pub mod interrupts;
pub mod intrinsics;
pub mod paging;
pub mod syscalls;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivilegeLevel(u8);

impl PrivilegeLevel {
    pub const KERNEL: PrivilegeLevel = PrivilegeLevel::from(0);
    pub const USERSPACE: PrivilegeLevel = PrivilegeLevel::from(3);

    pub const fn from(value: u8) -> Self {
        PrivilegeLevel(value)
    }
}

impl From<u8> for PrivilegeLevel {
    fn from(value: u8) -> Self {
        PrivilegeLevel(value)
    }
}

impl Into<u8> for PrivilegeLevel {
    fn into(self) -> u8 {
        self.0
    }
}
