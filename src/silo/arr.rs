//-- arr.rs -----------------------------------------------------------------------------------------------------------------------
use	std::marker::PhantomData;
use	std::ops::{ Deref, DerefMut };
use	std::ptr::NonNull;
use	crate::silo::uint::U32;
use	crate::silo::useg::USeg;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Arr< 'a, T> {
    _Ptr: NonNull< T>,
    _Size: U32,
    _Marker: PhantomData< &'a T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl< 'a, T: Send> Send for Arr<'a, T>
{ }
unsafe impl< 'a, T: Sync> Sync for Arr<'a, T>
{ }

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Arr<'a, T>
{
    pub fn	New< S: Into< U32>>( ptr: NonNull< T>, size: S) -> Self
    {
        Arr {
            _Ptr: ptr,
            _Size: size.into(),
            _Marker: PhantomData,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> U32
    {
        self._Size
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	IsEmpty( &self) -> bool
    {
        self.Size() == U32( 0)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	USeg( &self) -> USeg
    {
        USeg::Create( U32( 0), self.Size())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	At< K: Into< U32>>( &self, k: K) -> &'a T {
        unsafe {
			let  	ptr = self._Ptr.as_ptr().add( k.into().as_usize());
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MutAt< K: Into< U32>>( &self, k: K) -> &'a mut T {
        unsafe {
			let  	ptr = self._Ptr.as_ptr().add( k.into().as_usize());
            &mut *ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetAt< K: Into< U32>>( &self, k: K, a: &T) -> &'a T
    where
        T: Clone,
    {
        unsafe {
			let  	ptr = self._Ptr.as_ptr().add( k.into().as_usize());
            *ptr = a.clone();
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MoveAt< K: Into< U32>>( &self, k: K, a: &mut T) -> &'a T {
        unsafe {
			let  	ptr = self._Ptr.as_ptr().add( k.into().as_usize());
            std::ptr::swap( ptr, a);
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SwapAt< I: Into< U32>, J: Into< U32>>( &self, i: I, j: J)
    {
        unsafe {
            std::ptr::swap( 
                self._Ptr.add( i.into().as_usize()).as_ptr(),
                self._Ptr.add( j.into().as_usize()).as_ptr(),
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	LSnip< C: Into< U32>>( &self, count: C) -> Self
    {
		let  	cnt = count.into();
        Arr::New( 
            unsafe
            { self._Ptr.add( cnt.as_u32() as usize) },
            self.Size() - cnt,
        )
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RSnip< C: Into< U32>>( &self, count: C) -> Self
    {
		let  	cnt = count.into();
        Arr::New( self._Ptr, self.Size() - cnt)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Span< F>( &self, mut f: F) -> bool
    where
        F: FnMut( &T) -> bool,
    {
        if self.IsEmpty() {
            return true;
        }
        self.USeg().Span( |k| f( self.At( k)))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Deref for Arr<'a, T>
{
    type Target = [T];
    fn	deref( &self) -> &Self::Target
    {
        unsafe
        { std::slice::from_raw_parts( self._Ptr.as_ptr(), usize::from( self._Size)) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> DerefMut for Arr<'a, T>
{
    fn	deref_mut( &mut self) -> &mut Self::Target
    {
        unsafe
        { std::slice::from_raw_parts_mut( self._Ptr.as_ptr(), usize::from( self._Size)) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Clone for Arr<'a, T>
{
    fn	clone( &self) -> Self
    {
        *self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Copy for Arr<'a, T>
{ }

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: std::fmt::Debug> std::fmt::Debug for Arr<'a, T>
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result {
        std::fmt::Debug::fmt( &**self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: PartialEq> PartialEq for Arr<'a, T>
{
    fn	eq( &self, other: &Self) -> bool
    {
        **self == **other
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: Eq> Eq for Arr<'a, T>
{ }

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T, const N: usize> From< &'a [T; N]> for Arr<'a, T>
{
    fn	from( arr: &'a [T; N]) -> Self
    {
        unsafe {
            Arr::New( NonNull::new_unchecked( arr.as_ptr() as *mut T), N)
        }
    }
}

impl< 'a, T, const N: usize> From< &'a mut [T; N]> for Arr<'a, T>
{
    fn	from( arr: &'a mut [T; N]) -> Self
    {
        unsafe {
            Arr::New( NonNull::new_unchecked( arr.as_mut_ptr()), N)
        }
    }
}

impl< 'a, T> From< &'a [T]> for Arr<'a, T>
{
    fn	from( slice: &'a [T]) -> Self
    {
        unsafe {
            Arr::New( NonNull::new_unchecked( slice.as_ptr() as *mut T), slice.len())
        }
    }
}

impl< 'a, T> From< &'a mut [T]> for Arr<'a, T>
{
    fn	from( slice: &'a mut [T]) -> Self
    {
        unsafe {
            Arr::New( NonNull::new_unchecked( slice.as_mut_ptr()), slice.len())
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
