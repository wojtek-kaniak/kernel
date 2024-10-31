use core::cell::{SyncUnsafeCell, UnsafeCell};

use spin::Once;

/// A primitive that provides lazy one-time mutable initialization,
/// to avoid copying large structures (and stack-overflowing)
pub struct InitOnce<T> {
    data: SyncUnsafeCell<T>,
    init_lock: Once,
}

impl<T> InitOnce<T> {
    pub const fn new(preinit_value: T) -> InitOnce<T> {
        Self {
            data: SyncUnsafeCell::new(preinit_value),
            init_lock: Once::new(),
        }
    }

    /// Performs the initialization once, subsequent calls will be ignored \
    /// Returns an immutable reference to the inner `T`
    pub fn initialize(&self, f: impl FnOnce(&mut T)) -> &T {
        self.init_lock.call_once(|| unsafe {
            // SAFETY:
            // immutable references may exist only after initialization,
            // only a single mutable reference may exist at a time, only before initialization.
            f(&mut *self.data.get())
        });

        // SAFETY: initialization completed, no mutable references may exist at this point
        unsafe {
            &*self.data.get()
        }
    }

    /// Similar to [InitOnce::initialize] bu allows the closure to fail, leaving the object uninitialized
    /// (but still possibly mutated by the failed initializer closure).
    pub fn try_initialize(&self, f: impl FnOnce(&mut T) -> Result<(), ()>) -> Result<&T, ()> {
        self.init_lock.try_call_once(|| unsafe {
            // SAFETY:
            // immutable references may exist only after initialization,
            // only a single mutable reference may exist at a time, only before initialization.
            f(&mut *self.data.get())
        }).map(|_| unsafe {
            // SAFETY: initialization succeeded, no mutable references may exist at this point
            &*self.data.get()
        })
    }

    /// Will panic if not initialized
    pub fn get(&self) -> &T {
        if !self.init_lock.is_completed() {
            panic!("not initialized");
        }

        // SAFETY: is initialized
        unsafe { self.get_unchecked() }
    }

    /// # Safety
    /// must be initialized
    pub unsafe fn get_unchecked(&self) -> &T {
        // SAFETY: only immutable references may exist after initialization

        unsafe {
            &*self.data.get()
        }
    }

    /// Check if initialization has been completed. \
    /// Safe to call [InitOnce::get_unchecked] if `true` is returned.
    pub fn is_completed(&self) -> bool {
        self.init_lock.is_completed()
    }
}

// pub struct InitOnce<T> {
//     data: SyncUnsafeCell<MaybeUninit<T>>,
//     initialized: AtomicBool,
//     lock: AtomicBool
// }

// impl<T> InitOnce<T> {
//     pub const fn new() -> Self {
//         Self {
//             data: SyncUnsafeCell::new(MaybeUninit::uninit()),
//             initialized: AtomicBool::new(false),
//             lock: AtomicBool::new(false)
//         }
//     }

//     pub fn initialize(&self, value: T) -> Result<(), &T> {
//         match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
//             Ok(_) => {
//                 unsafe {
//                     let mut_ref = self.data.get().as_mut().unwrap_unchecked();
//                     *mut_ref = MaybeUninit::new(value);
//                 }
//                 self.initialized.store(true, Ordering::SeqCst);
//                 Ok(())
//             },
//             Err(_) => {
//                 unsafe {
//                     Err(self.get_unchecked())
//                 }
//             }
//         }
//     }

//     pub fn initialize_with(&self, func: impl FnOnce() -> T) -> Result<(), &T> {
//         match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
//             Ok(_) => {
//                 unsafe {
//                     let mut_ref = self.data.get().as_mut().unwrap_unchecked();
//                     *mut_ref = MaybeUninit::new(func());
//                 }
//                 self.initialized.store(true, Ordering::SeqCst);
//                 Ok(())
//             },
//             Err(_) => {
//                 unsafe {
//                     Err(self.get_unchecked())
//                 }
//             }
//         }
//     }

//     pub fn get(&self) -> Option<&T> {
//         // TODO: relax ordering?
//         if self.initialized.load(Ordering::SeqCst) {
//             Some(unsafe { self.get_unchecked() })
//         } else {
//             None
//         }
//     }

//     pub unsafe fn get_unchecked(&self) -> &T {
//         unsafe {
//             self.data.get().as_ref().unwrap_unchecked().assume_init_ref()
//         }
//     }

//     pub fn is_initialized(&self) -> bool {
//         self.initialized.load(Ordering::SeqCst)
//     }
// }

// impl<T> Default for InitOnce<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// TODO: remove
#[repr(transparent)]
pub struct UnsafeSync<T>(UnsafeCell<T>);

impl<T> UnsafeSync<T> {
    pub fn new(value: T) -> Self {
        UnsafeSync(UnsafeCell::from(value))
    }

    /// # Safety
    /// Mutable references cannot exist
    pub unsafe fn get(&self) -> &T {
        unsafe { self.0.get().as_ref().unwrap_unchecked() }
    }

    /// # Safety
    /// No references may not exist
    pub unsafe fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

impl<T> Default for UnsafeSync<T>
where
    T: Default,
{
    fn default() -> Self {
        UnsafeSync(Default::default())
    }
}

unsafe impl<T> Sync for UnsafeSync<T> {}
unsafe impl<T> Send for UnsafeSync<T> {}

impl<T> From<T> for UnsafeSync<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
