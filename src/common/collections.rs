use core::{mem::MaybeUninit, ops::{Index, IndexMut}};

// Switch to fixedvec
#[derive(Debug)]
pub struct FixedSizeVec<T, const MAX_SIZE: usize> {
    data: [MaybeUninit<T>; MAX_SIZE],
    len: usize
}

impl<T, const MAX_SIZE: usize> FixedSizeVec<T, MAX_SIZE> {
    pub fn new() -> Self {
        Self { data: MaybeUninit::uninit_array(), len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            MaybeUninit::slice_assume_init_ref(&self.data[..self.len])
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr().cast::<T>()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr().cast::<T>()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            Some(&mut self[index])
        } else {
            None
        }
    }

    /// index must be less than the len
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe {
            self.data.get_unchecked(index).assume_init_ref()
        }
    }

    /// index must be less than the len
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        unsafe {
            self.data.get_unchecked_mut(index).assume_init_mut()
        }
    }

    /// This function can be used to initialize elements without causing UB \
    /// Destructor is not called on the previous value
    pub unsafe fn set_unchecked(&mut self, index: usize, value: T) {
        unsafe {
            *self.data.get_unchecked_mut(index) = MaybeUninit::new(value);
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), ()> {
        if self.len() == MAX_SIZE {
            Err(())
        } else {
            unsafe {
                let ix = self.len();
                self.set_unchecked(ix, value);
                self.set_len(self.len() + 1);
            }
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let old_len = self.len();
                self.set_len(old_len - 1);
                Some(self.data.get_unchecked(old_len - 1).assume_init_read())
            }
        }
    }

    pub fn insert(&mut self, index: usize, value: T) -> Result<(), ()> {
        if index > self.len() {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), index);
        } else if index > MAX_SIZE - 1 {
            return Err(())
        } else {
            unsafe {
                let start = self.data.as_mut_ptr().add(index);
                core::ptr::copy(start, start.add(1), self.len() - index);
                self.set_unchecked(index, value);
            }
            Ok(())
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        // matching Vec::truncate behaviour
        if self.len() > new_len {
            unsafe {
                for ix in new_len..self.len() {
                    self.data.get_unchecked_mut(ix).assume_init_drop();
                }
                self.set_len(new_len);
            }
        }
    }
}

impl<T, const MAX_SIZE: usize> Default for FixedSizeVec<T, MAX_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const MAX_SIZE: usize> FixedSizeVec<T, MAX_SIZE> {
    pub fn resize(&mut self, new_len: usize, fill_value: T) -> Result<(), ()> {
        if new_len > MAX_SIZE {
            return Err(())
        }

        if new_len > self.len() {
            unsafe {
                let old_len = self.len();
                for ix in old_len..new_len {
                    self.set_unchecked(ix, fill_value.clone());
                }
                self.set_len(new_len);
            }
        } else {
            self.truncate(new_len);
        }

        Ok(())
    }
}

impl<T: Copy, const MAX_SIZE: usize> FixedSizeVec<T, MAX_SIZE> {
    pub fn from_slice(slice: &[T]) -> Self {
        let mut result = Self { data: MaybeUninit::uninit_array(), len: slice.len() };
        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), result.data.as_mut_ptr().cast::<T>(), slice.len());
        }
        result
    }
}

impl<T: Copy, const MAX_SIZE: usize> Clone for FixedSizeVec<T, MAX_SIZE> {
    fn clone(&self) -> Self {
        Self::from_slice(self.as_slice())
    }
}

impl<T, const MAX_SIZE: usize> Index<usize> for FixedSizeVec<T, MAX_SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index > self.len() {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), index);
        }

        unsafe {
            self.data.get_unchecked(index).assume_init_ref()
        }
    }
}

impl<T, const MAX_SIZE: usize> IndexMut<usize> for FixedSizeVec<T, MAX_SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > self.len() {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), index);
        }

        unsafe {
            self.data.get_unchecked_mut(index).assume_init_mut()
        }
    }
}

impl<'a, T, const MAX_SIZE: usize> IntoIterator for &'a FixedSizeVec<T, MAX_SIZE> {
    type Item = &'a T;

    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            MaybeUninit::slice_assume_init_ref(self.data.get_unchecked(..self.len())).iter()
        }
    }
}

impl<T, const MAX_SIZE: usize> Drop for FixedSizeVec<T, MAX_SIZE> {
    fn drop(&mut self) {
        for i in 0..self.len() {
            unsafe {
                self.data.get_unchecked_mut(i).assume_init_drop();
            }
        }
    }
}
