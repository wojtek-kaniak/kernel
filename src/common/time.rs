#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixEpochTime(/* UNIX millis */ u64);

impl UnixEpochTime {
    pub const UNIX_EPOCH: UnixEpochTime = Self::new(0);

    pub const fn new(milliseconds: u64) -> Self {
        UnixEpochTime(milliseconds)
    }

    pub const fn millis(self) -> u64 {
        self.0
    }

    pub const fn seconds(self) -> u64 {
        self.0 / 1000
    }
}

impl From<u64> for UnixEpochTime {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<UnixEpochTime> for u64 {
    fn from(val: UnixEpochTime) -> Self {
        val.0
    }
}
