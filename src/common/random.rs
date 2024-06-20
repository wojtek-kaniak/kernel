use core::sync::atomic::{AtomicU64, Ordering};

use spin::Once;

use crate::{common::time::UnixEpochTime, arch::intrinsics::time_stamp_counter};

static WEAK_RNG: Once<XorshiftStar> = Once::new();

/// This function may be only called once, all subsequent calls will panic or be ignored
pub fn weak_initialize(time: UnixEpochTime) {
    // best effort panic
    if WEAK_RNG.is_completed() {
        panic!("weak RNG already initialized");
    }

    WEAK_RNG.call_once(|| {
        let mut seed: u64 = time.into();
        seed ^= time_stamp_counter();

        XorshiftStar::new(seed)
    });
}

pub fn weak() -> WeakRng {
    WeakRng::new(WEAK_RNG.get().expect("Weak RNG uninitialized"))
}

#[derive(Clone, Copy, Debug)]
pub struct WeakRng(&'static XorshiftStar);

impl WeakRng {
    fn new(rng: &'static XorshiftStar) -> Self {
        WeakRng(rng)
    }

    pub fn next(&self) -> u64 {
        self.0.next()
    }

    /// Uniformly distributed value in range [0:1)
    pub fn next_f64(&self) -> f64 {
        // Taken from https://github.com/lcrocker/ojrandlib, CC0
        // Explanation at https://stackoverflow.com/a/5022920/

        // [1:2)
        let val = (self.next() & 0xFFFFFFFFFFFFF_u64) | 0x3FF0000000000000_u64;
        // [0:1)
        f64::from_bits(val) - 1_f64
    }
}

/// Xorshift*
#[derive(Debug)]
struct XorshiftStar(AtomicU64);

impl XorshiftStar {
    const M: u64 = 0x2545f4914f6cdd1d;

    pub fn new(seed: u64) -> Self {
        // seed must be nonzero
        let seed = if seed > 0 { seed } else { u64::MAX };
        Self(AtomicU64::new(seed))
    }

    pub fn next(&self) -> u64 {
        // TODO: Relax ordering
        let old = self.0.load(Ordering::SeqCst);
        let mut value = old;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        match self.0.compare_exchange(old, value, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => value * Self::M,
            Err(_) => self.next()
        }
    }
}
