//-- buff.rs ----------------------------------------------------------------------------------------------------------------------

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use crate::arr::Arr;

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

impl<T> Buff<T>
{
    pub fn IsEmpty(&self) -> bool { self.is_empty() }
}

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

            for i in 0.._Size                             // Initialize each element in the contiguous memory block
            {
                std::ptr::write(raw_ptr.add(i), initial_value.clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new( guard);                           // Defuse the guard so memory/elements aren't cleaned up when exiting the block

            Buff { _Ptr: NonNull::new_unchecked(raw_ptr), _Size, _Marker: PhantomData }
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

impl<T: Clone> Clone for Buff<T>
{
    fn clone(&self) -> Self
    {
        if self._Size == 0 || std::mem::size_of::<T>() == 0
        {
            return Buff { _Ptr: NonNull::dangling(), _Size: self._Size, _Marker: PhantomData };
        }

        let _Layout = Layout::array::<T>(self._Size).expect("Layout calculation failed");

        unsafe
        {
            let raw_ptr = alloc(_Layout) as *mut T;
            if raw_ptr.is_null()
            {
                handle_alloc_error(_Layout);
            }

            // Panic guard � same pattern as Buff::new
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

            for i in 0..self._Size
            {
                std::ptr::write(raw_ptr.add(i), self[i].clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new(guard);

            Buff { _Ptr: NonNull::new_unchecked(raw_ptr), _Size: self._Size, _Marker: PhantomData }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
