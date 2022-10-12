// Move to time::unix module if more added

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixEpochTime(u64);

impl UnixEpochTime {
    pub const UNIX_EPOCH: UnixEpochTime = Self::new(0);

    pub const fn new(value: u64) -> Self {
        UnixEpochTime(value)
    }
}

impl From<u64> for UnixEpochTime {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl Into<u64> for UnixEpochTime {
    fn into(self) -> u64 {
        self.0
    }
}
