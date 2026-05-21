//-- buff.rs ----------------------------------------------------------------------------------------------------------------------

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use crate::arr::Arr;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Buff<T>
{
    _Ptr: NonNull<[T]>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl<T: Send> Send for Buff<T> {}
unsafe impl<T: Sync> Sync for Buff<T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> Buff<T>
{
    pub fn IsEmpty(&self) -> bool { self.is_empty() }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T: Clone> Buff<T>
{
    pub fn new(size: usize, initial_value: T) -> Self
    {
        let     is_zst = std::mem::size_of::<T>() == 0;

        if size == 0 || is_zst
        {
            let dangling = NonNull::slice_from_raw_parts(NonNull::dangling(), size);
            return Buff { _Ptr: dangling };
        }

        // Calculate _Layout for an array of T with length `size`
        let     _Layout = Layout::array::<T>(size).expect( "Layout calculation failed");

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
                        if self._InitCount > 0
                        {
                            let     slice_ptr = std::ptr::slice_from_raw_parts_mut(self._Ptr, self._InitCount);
                            std::ptr::drop_in_place(slice_ptr);             // Drop already initialized elements
                        }
                        dealloc(self._Ptr as *mut u8, self._Layout);              // Deallocate the contiguous chunk of raw memory
                    }
                }
            }

            let mut guard = RawAllocationGuard { _Ptr: raw_ptr, _Layout, _InitCount: 0 };

            for i in 0..size                             // Initialize each element in the contiguous memory block
            {
                std::ptr::write(raw_ptr.add(i), initial_value.clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new( guard);                           // Defuse the guard so memory/elements aren't cleaned up when exiting the block

            let non_null_ptr = NonNull::new_unchecked(raw_ptr);
            let slice_ptr = NonNull::slice_from_raw_parts(non_null_ptr, size);
            Buff { _Ptr: slice_ptr }
        }
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> Buff<T>
{
    pub fn as_arr(&self) -> Arr<'_, T>
    {
        Arr::new(self._Ptr.cast::<T>(), self._Ptr.len())
    }

    pub fn as_mut_arr(&mut self) -> Arr<'_, T>
    {
        Arr::new(self._Ptr.cast::<T>(), self._Ptr.len())
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
            self._Ptr.as_ref()
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
            self._Ptr.as_mut()
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T> Drop for Buff<T>
{
    fn drop(&mut self)
    {
        let size = self._Ptr.len();
        let is_zst = std::mem::size_of::<T>() == 0;
        if size == 0 || is_zst
        {
            return;
        }

        let _Layout = Layout::array::<T>(size).expect( "Too Big");

        unsafe
        {
            // Drop all elements via slice pointer
            std::ptr::drop_in_place(self._Ptr.as_ptr());

            // Deallocate the contiguous chunk of raw memory
            dealloc(self._Ptr.cast::<u8>().as_ptr(), _Layout);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<T: Clone> Clone for Buff<T>
{
    fn clone(&self) -> Self
    {
        let size = self._Ptr.len();
        if size == 0 || std::mem::size_of::<T>() == 0
        {
            let dangling = NonNull::slice_from_raw_parts(NonNull::dangling(), size);
            return Buff { _Ptr: dangling };
        }

        let _Layout = Layout::array::<T>(size).expect("Layout calculation failed");

        unsafe
        {
            let raw_ptr = alloc(_Layout) as *mut T;
            if raw_ptr.is_null()
            {
                handle_alloc_error(_Layout);
            }

            // Panic guard – same pattern as Buff::new
            struct CloneGuard<T>
            {
                _Ptr: *mut T,
                _Layout: Layout,
                _InitCount: usize,
            }

            impl<T> Drop for CloneGuard<T>
            {
                fn drop(&mut self)
                {
                    unsafe
                    {
                        if self._InitCount > 0
                        {
                            let slice_ptr = std::ptr::slice_from_raw_parts_mut(self._Ptr, self._InitCount);
                            std::ptr::drop_in_place(slice_ptr);
                        }
                        dealloc(self._Ptr as *mut u8, self._Layout);
                    }
                }
            }

            let mut guard = CloneGuard { _Ptr: raw_ptr, _Layout, _InitCount: 0 };

            for i in 0..size
            {
                std::ptr::write(raw_ptr.add(i), self[i].clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new(guard);

            let non_null_ptr = NonNull::new_unchecked(raw_ptr);
            let slice_ptr = NonNull::slice_from_raw_parts(non_null_ptr, size);
            Buff { _Ptr: slice_ptr }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
