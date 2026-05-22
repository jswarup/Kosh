//-- arr.rs -----------------------------------------------------------------------------------------------------------------------

use std::marker::PhantomData;
use std::ops::{ Deref, DerefMut};
use std::ptr::NonNull;

use crate::silo::useg::USeg;


//---------------------------------------------------------------------------------------------------------------------------------

pub struct Arr<'a, T>
{
    _Ptr: NonNull<T>,
    _Size: u32,
    _Marker: PhantomData<&'a T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl<'a, T: Send> Send for Arr<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Arr<'a, T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> Arr<'a, T>
{
    pub fn New( ptr: NonNull<T>, size: u32) -> Self
    {
        Arr
        {
            _Ptr: ptr,
            _Size: size,
            _Marker: PhantomData,
        }
    }

    pub fn Size( &self) -> u32
    {
        self._Size
    }

    fn At( &self, k:u32) -> &T
    {
        unsafe
        {
            self._Ptr.add( k as usize).as_ref()
        }
    }


    fn SetAt( &self, k:u32, a :&T) where T: Clone
    {
        unsafe
        {
            *self._Ptr.add( k as usize).as_mut() = a.clone();
        }
    }

    pub fn IsEmpty( &self) -> bool
    {
        self.Size() == 0
    }

    pub fn  USeg( &self) ->USeg
    {
        USeg::New( 0, self.Size())
    }

    pub fn LSnip( &self, count: u32) -> Self
    {
        Arr::New( unsafe { self._Ptr.add( count as usize) }, self.Size() - count)
    }

    pub fn RSnip( &self, count: u32) -> Self
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
            std::slice::from_raw_parts( self._Ptr.as_ptr(), self._Size as usize)
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
            std::slice::from_raw_parts_mut( self._Ptr.as_ptr(), self._Size as usize)
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
