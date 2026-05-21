//-- buff.rs ----------------------------------------------------------------------------------------------------------------------
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Buff<T>
{
    _Ptr: NonNull<T>,
    _Size: usize,
    _Marker: PhantomData<T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl<T: Send> Send for Buff<T> {}
unsafe impl<T: Sync> Sync for Buff<T> {}

//---------------------------------------------------------------------------------------------------------------------------------

pub use crate::arr::Arr;

//---------------------------------------------------------------------------------------------------------------------------------

impl<T: Clone> Buff<T>
{
    pub fn new(_Size: usize, initial_value: T) -> Self
    {
        let     is_zst = std::mem::size_of::<T>() == 0;

        if _Size == 0 || is_zst
        {
            return Buff { _Ptr: NonNull::dangling(), _Size, _Marker: PhantomData};
        }

        // Calculate _Layout for an array of T with length `_Size`
        let     _Layout = Layout::array::<T>( _Size).expect( "Layout calculation failed");

        unsafe
        {
            let     raw_ptr = alloc(_Layout) as *mut T;  // Allocate memory
            if raw_ptr.is_null()
            {
                handle_alloc_error(_Layout);
            }

            // Drop guard to prevent resource leaks if initial_value.clone() panics during loop
            struct RawAllocationGuard<T>
            {
                _Ptr: *mut T,
                _Layout: Layout,
                _InitCount: usize,
            }

            impl<T> Drop for RawAllocationGuard<T>
            {
                fn drop(&mut self)
                {
                    unsafe
                    {
                        let     slice_ptr = std::ptr::slice_from_raw_parts_mut(self._Ptr, self._InitCount);
                        std::ptr::drop_in_place(slice_ptr);             // Drop already initialized elements
                        dealloc(self._Ptr as *mut u8, self._Layout);              // Deallocate the contiguous chunk of raw memory
                    }
                }
            }

            let mut guard = RawAllocationGuard { _Ptr: raw_ptr, _Layout, _InitCount: 0 };

            for i in 0.._Size                             // Initialize each element in the contiguous memory block
            {
                std::ptr::write(raw_ptr.add(i), initial_value.clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new( guard);                           // Defuse the guard so memory/elements aren't cleaned up when exiting the block

            return Buff { _Ptr: NonNull::new_unchecked(raw_ptr), _Size, _Marker: PhantomData }
        }
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> Buff<T>
{
    pub fn as_arr(&self) -> Arr<'_, T>
    {
        Arr::new(self._Ptr, self._Size)
    }

    pub fn as_mut_arr(&mut self) -> Arr<'_, T>
    {
        Arr::new(self._Ptr, self._Size)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> Deref for Buff<T>
{
    type Target = [T];

    fn deref( &self) -> &Self::Target
    {
        unsafe
        {
            std::slice::from_raw_parts(self._Ptr.as_ptr(), self._Size)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> DerefMut for Buff<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        unsafe
        {
            std::slice::from_raw_parts_mut(self._Ptr.as_ptr(), self._Size)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> Drop for Buff<T>
{
    fn drop(&mut self)
    {
        let is_zst = std::mem::size_of::<T>() == 0;
        if self._Size == 0 || is_zst
        {
            return;
        }

        let _Layout = Layout::array::<T>(self._Size).expect( "Too Big");

        unsafe
        {
            // Drop all elements via slice pointer
            std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(self._Ptr.as_ptr(), self._Size));

            // Deallocate the contiguous chunk of raw memory
            dealloc(self._Ptr.as_ptr() as *mut u8, _Layout);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn TestBuff()
{
    // Allocate a buffer of 5 elements, all initialized to 10.
    let mut buffer = Buff::new(5, 10);

    // Safely mutate an element at a specific index
    buffer[2] = 99;

    // Safely read elements
    println!("Element at index 0: {}", buffer[0]); // Output: 10
    println!("Element at index 2: {}", buffer[2]); // Output: 99

    // This will panic safely instead of causing undefined behavior:
    // buffer[5] = 100;
} // Buffer safely drops here. Elements are dropped, and memory is freed.

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests
{
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_safe_buffer_basic_ops()
    {
        let mut buffer = Buff::new(3, 42);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer[0], 42);
        assert_eq!(buffer[1], 42);
        assert_eq!(buffer[2], 42);

        buffer[1] = 100;
        assert_eq!(buffer[1], 100);

        // Test slice methods made available via Deref
        assert_eq!(buffer.first(), Some(&42));
        assert_eq!(buffer.last(), Some(&42));
    }

    #[test]
    fn test_safe_buffer_zst()
    {
        let buffer = Buff::new(10, ());
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer[5], ());
    }

    #[test]
    fn test_arr_basic_ops()
    {
        let mut buffer = Buff::new(3, 42);
        {
            let mut arr = buffer.as_mut_arr();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], 42);
            arr[1] = 100;
        }
        assert_eq!(buffer[1], 100);

        let arr2 = buffer.as_arr();
        assert_eq!(arr2[1], 100);
    }

    struct PanicOnClone
    {
        drop_counter: Arc<Mutex<usize>>,
        panic_after: usize,
        clone_counter: Arc<Mutex<usize>>,
    }

    impl Clone for PanicOnClone
    {
        fn clone(&self) -> Self
        {
            let mut count = self.clone_counter.lock().unwrap();
            *count += 1;
            if *count >= self.panic_after
            {
                panic!("Panic on clone!");
            }
            PanicOnClone
            {
                drop_counter: self.drop_counter.clone(),
                panic_after: self.panic_after,
                clone_counter: self.clone_counter.clone(),
            }
        }
    }

    impl Drop for PanicOnClone
    {
        fn drop(&mut self)
        {
            let mut count = self.drop_counter.lock().unwrap();
            *count += 1;
        }
    }

    #[test]
    fn test_safe_buffer_panic_safety()
    {
        let drop_counter = Arc::new(Mutex::new(0));
        let clone_counter = Arc::new(Mutex::new(0));

        let initial = PanicOnClone
        {
            drop_counter: drop_counter.clone(),
            panic_after: 3,
            clone_counter: clone_counter.clone(),
        };

        // Run in catch_unwind to capture the panic during cloning
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
        {
            let _buffer = Buff::new(5, initial);
        }));

        assert!(result.is_err());

        // Check drop counts:
        // - The original `initial` instance is dropped: 1
        // - During `new()`, we successfully cloned it 2 times (index 0, 1), and then panic on 3rd clone.
        // - Those 2 successfully cloned instances should be dropped by the RawAllocationGuard.
        // Total drops must be 3 (1 original + 2 clones).
        assert_eq!(*drop_counter.lock().unwrap(), 3);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
