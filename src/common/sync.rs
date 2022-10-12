use core::{cell::{UnsafeCell, SyncUnsafeCell}, mem::MaybeUninit, sync::atomic::{AtomicBool, Ordering}};

pub struct InitOnce<T> {
    data: SyncUnsafeCell<MaybeUninit<T>>,
    initialized: AtomicBool
}

impl<T> InitOnce<T> {
    pub const fn new() -> Self {
        Self {
            data: SyncUnsafeCell::new(MaybeUninit::uninit()),
            initialized: AtomicBool::new(false)
        }
    }

    pub fn initialize(&self, value: T) -> Result<(), &T> {
        match self.initialized.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => {
                unsafe {
                    let mut_ref = self.data.get().as_mut().unwrap_unchecked();
                    *mut_ref = MaybeUninit::new(value);
                }
                Ok(())
            },
            Err(_) => {
                unsafe {
                    Err(self.get_unchecked())
                }
            }
        }
    }

    pub fn get(&self) -> Option<&T> {
        // TODO: relax ordering?
        if self.initialized.load(Ordering::SeqCst) {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe {
            self.data.get().as_ref().unwrap_unchecked().assume_init_ref()
        }
    }
}

// TODO: remove
#[repr(transparent)]
pub struct UnsafeSync<T>(UnsafeCell<T>);

impl<T> UnsafeSync<T> {
    pub unsafe fn new(value: T) -> Self {
        UnsafeSync(UnsafeCell::from(value))
    }

    pub unsafe fn get(&self) -> &T {
        unsafe { self.0.get().as_ref().unwrap_unchecked() }
    }

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
        unsafe {
            Self::new(value)
        }
    }
}
