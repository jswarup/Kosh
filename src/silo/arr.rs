//-- arr.rs -----------------------------------------------------------------------------------------------------------------------

use std::marker::PhantomData;
use std::ops::{ Deref, DerefMut};
use std::ptr::NonNull;

use crate::silo::useg::USeg;
use crate::silo::uint::U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Arr<'a, T>
{
    _Ptr: NonNull<T>,
    _Size: U32,
    _Marker: PhantomData<&'a T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl<'a, T: Send> Send for Arr<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Arr<'a, T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> Arr<'a, T>
{
    pub fn New( ptr: NonNull<T>, size: U32) -> Self
    {
        Arr { _Ptr: ptr, _Size: size, _Marker: PhantomData, }
    }

    pub fn Size( &self) -> U32
    {
        self._Size
    }

    pub fn len( &self) -> U32
    {
        self.Size()
    }

    pub fn At( &self, k:U32) -> &T
    {
        unsafe {
            &*self._Ptr.as_ptr().add(k.as_u32() as usize)
        }
    }


    pub fn SetAt( &self, k:U32, a :&T) where T: Clone
    {
        unsafe {
            *self._Ptr.as_ptr().add(k.as_u32() as usize) = a.clone();
        }
    }

    pub fn MoveAt( &self, k:U32, a: &mut T) where T: Default
    {
        unsafe {
            *self._Ptr.as_ptr().add(k.as_u32() as usize) = std::mem::take(a);
        }
    }

    pub fn SwapAt( &self, i:U32, j:U32)
    {
        unsafe
        {
            std::ptr::swap( self._Ptr.add( i.as_u32() as usize).as_ptr(), self._Ptr.add( j.as_u32() as usize).as_ptr());
        }
    }

    pub fn IsEmpty( &self) -> bool
    {
        self.Size() == U32::from(0)
    }

    pub fn  USeg( &self) ->USeg
    {
        USeg::Create( U32::from(0), self.Size())
    }

    pub fn LSnip( &self, count: U32) -> Self
    {
        Arr::New( unsafe { self._Ptr.add(count.as_u32() as usize) }, self.Size() - count)
    }

    pub fn RSnip( &self, count: U32) -> Self
    {
        Arr::New( self._Ptr, self.Size() - count)
    }


    pub fn Span<F>( &self, mut f: F) -> bool
        where F: FnMut( &T) -> bool,
    {
        if self.IsEmpty()
        {
            return true;
        }
        self.USeg().Span( |k| {
            f( self.At( k))
        })
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> Deref for Arr<'a, T>
{
    type Target = [ T];

    fn deref( &self) -> &Self::Target
    {
        unsafe
        {
            std::slice::from_raw_parts( self._Ptr.as_ptr(), usize::from(self._Size))
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> DerefMut for Arr<'a, T>
{
    fn deref_mut( &mut self) -> &mut Self::Target
    {
        unsafe
        {
            std::slice::from_raw_parts_mut( self._Ptr.as_ptr(), usize::from(self._Size))
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> Clone for Arr<'a, T>
{
    fn clone( &self) -> Self
    {
        *self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> Copy for Arr<'a, T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T: std::fmt::Debug> std::fmt::Debug for Arr<'a, T>
{
    fn fmt( &self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        std::fmt::Debug::fmt( &**self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T: PartialEq> PartialEq for Arr<'a, T>
{
    fn eq( &self, other: &Self) -> bool
    {
        **self == **other
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T: Eq> Eq for Arr<'a, T> {}

//---------------------------------------------------------------------------------------------------------------------------------
