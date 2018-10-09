use std::alloc::{self, Alloc, Global, Layout};
use std::fmt;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::marker::{Sync, Send};

const CACHE_LINE_SIZE: usize = 64;
unsafe fn allocate_cache_line(size: usize) -> *mut u8 {
    alloc::Global.alloc_zeroed(Layout::from_size_align(size, CACHE_LINE_SIZE).unwrap())
        .unwrap().as_ptr() as *mut u8
}

pub struct CacheAligned<T: Sized> {
    ptr: NonNull<T>,
}

impl<T: Sized> Drop for CacheAligned<T> {
    fn drop(&mut self) {
        unsafe {
            alloc::Global.dealloc(
                NonNull::<u8>::new_unchecked(self.ptr.as_ptr() as *mut u8),
                Layout::from_size_align(size_of::<T>(), CACHE_LINE_SIZE).unwrap(),
            );
        }
    }
}

unsafe impl<T: Sized> Send for CacheAligned<T> {}

unsafe impl<T: Sized> Sync for CacheAligned<T> {}

impl<T: Sized> Deref for CacheAligned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Sized> DerefMut for CacheAligned<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Sized> CacheAligned<T> {
    pub fn allocate(src: T) -> CacheAligned<T> {
        unsafe {
            let alloc = allocate_cache_line(size_of::<T>()) as *mut T;
            ptr::write(alloc, src);
            CacheAligned {
                ptr: NonNull::new(alloc).unwrap(),
            }
        }
    }
}

impl<T: Sized> Clone for CacheAligned<T>
where
    T: Clone,
{
    fn clone(&self) -> CacheAligned<T> {
        unsafe {
            let alloc = allocate_cache_line(size_of::<T>()) as *mut T;
            ptr::copy(self.ptr.as_ptr() as *const T, alloc, 1);
            CacheAligned {
                ptr: NonNull::new(alloc).unwrap(),
            }
        }
    }
}

impl<T: Sized> fmt::Display for CacheAligned<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(&*self, f)
    }
}
