//-- arr.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ U8, U32 };
use	std::marker::PhantomData;
use	std::ops::{ Deref, DerefMut };
use	std::ptr::NonNull;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Arr< 'a, T>
{
    pub(crate) _Ptr: NonNull< T>,
    pub(crate) _Size: U32,
    _Marker: PhantomData< &'a T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl< 'a, T: Send> Send for Arr< 'a, T>
{
}
unsafe impl< 'a, T: Sync> Sync for Arr< 'a, T>
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Arr< 'a, T>
{
    pub fn	New< S: Into< U32>>( ptr: NonNull< T>, size: S) -> Self
    {
        Arr {
            _Ptr: ptr,
            _Size: size.into(),
            _Marker: PhantomData,
        }
    }


}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Deref for Arr< 'a, T>
{
    type Target = [T];
    fn	deref( &self) -> &Self::Target
    {
        unsafe { std::slice::from_raw_parts( self._Ptr.as_ptr(), usize::from( self._Size)) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> DerefMut for Arr< 'a, T>
{
    fn	deref_mut( &mut self) -> &mut Self::Target
    {
        unsafe { std::slice::from_raw_parts_mut( self._Ptr.as_ptr(), usize::from( self._Size)) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Clone for Arr< 'a, T>
{
    fn	clone( &self) -> Self
    {
        *self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> Copy for Arr< 'a, T>
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: std::fmt::Debug> std::fmt::Debug for Arr< 'a, T>
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        std::fmt::Debug::fmt( &**self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: PartialEq> PartialEq for Arr< 'a, T>
{
    fn	eq( &self, other: &Self) -> bool
    {
        **self == **other
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: Eq> Eq for Arr< 'a, T>
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T, const N: usize> From< &'a [T; N]> for Arr< 'a, T>
{
    fn	from( arr: &'a [T; N]) -> Self
    {
        unsafe { Arr::New( NonNull::new_unchecked( arr.as_ptr() as *mut T), N) }
    }
}
impl< 'a, T, const N: usize> From< &'a mut [T; N]> for Arr< 'a, T>
{
    fn	from( arr: &'a mut [T; N]) -> Self
    {
        unsafe { Arr::New( NonNull::new_unchecked( arr.as_mut_ptr()), N) }
    }
}
impl< 'a, T> From< &'a [T]> for Arr< 'a, T>
{
    fn	from( slice: &'a [T]) -> Self
    {
        unsafe {
            Arr::New( 
                NonNull::new_unchecked( slice.as_ptr() as *mut T),
                slice.len(),
            )
        }
    }
}
impl< 'a, T> From< &'a mut [T]> for Arr< 'a, T>
{
    fn	from( slice: &'a mut [T]) -> Self
    {
        unsafe { Arr::New( NonNull::new_unchecked( slice.as_mut_ptr()), slice.len()) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> Arr< 'a, U8>
{
    pub fn	Str( &self) -> &'a str
    {
        unsafe {
            let  	sliceU8: &'a [U8] = std::slice::from_raw_parts( self._Ptr.as_ptr(), self._Size.AsUsize());
            let  	bytes: &'a [u8] = std::mem::transmute( sliceU8);
            std::str::from_utf8_unchecked( bytes)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
