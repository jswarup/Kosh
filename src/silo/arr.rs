//-- arr.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ IAccess, U8, U32 };
use	crate::stalks::IWorker;
use	std::marker::PhantomData;
use	std::ops::{ Deref, DerefMut };
use	std::ptr::NonNull;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IArr< 'a, T: 'a>: IAccess< 'a, T> {
    fn	Ptr( &self) -> *const T;

    fn	MutAt< K: Into< U32>>( &self, k: K) -> &'a mut T
    {
        unsafe {
            let  	ptr = self.Ptr().cast_mut().add( k.into().AsUsize());
            &mut *ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SetAt< K: Into< U32>>( &self, k: K, a: &T) -> &'a T
    where
        T: Clone,
    {
        unsafe {
            let  	ptr = self.Ptr().cast_mut().add( k.into().AsUsize());
            *ptr = a.clone();
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SwapAt< K: Into< U32>>( &self, k: K, a: &mut T) -> &'a T
    {
        unsafe {
            let  	ptr = self.Ptr().cast_mut().add( k.into().AsUsize());
            std::ptr::swap( ptr, a);
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Swap< I: Into< U32>, J: Into< U32>>( &self, i: I, j: J)
    {
        unsafe {
            std::ptr::swap( 
                self.Ptr().cast_mut().add( i.into().AsUsize()),
                self.Ptr().cast_mut().add( j.into().AsUsize()),
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SwapFrom< S: Into< U32>, D: Into< U32>>( 
        &self,
        dstStart: D,
        src: &Arr< '_, T>,
        srcStart: S,
        count: U32,
    ) where
        T: Copy,
    {
        unsafe {
            std::ptr::swap_nonoverlapping( 
                src.Ptr().cast_mut().add( srcStart.into().AsUsize()),
                self.Ptr().cast_mut().add( dstStart.into().AsUsize()),
                count.AsUsize(),
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	LSnip< C: Into< U32>>( &self, count: C) -> Arr< 'a, T>
    {
        let  	cnt = count.into();
        Arr::New( 
            unsafe { std::ptr::NonNull::new_unchecked(self.Ptr().cast_mut().add( cnt.AsU32() as usize)) },
            self.Size() - cnt,
        )
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	RSnip< C: Into< U32>>( &self, count: C) -> Arr< 'a, T>
    {
        let  	cnt = count.into();
        Arr::New( unsafe { std::ptr::NonNull::new_unchecked(self.Ptr().cast_mut()) }, self.Size() - cnt)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	QuickSorter< Less>( &self, less: Less) -> impl Fn( &dyn IWorker) + Send + Sync + 'a
    where
        Less: Fn( &T, &T) -> bool + Send + Sync + 'a + Copy,
        T: Send + Sync + 'a,
    {
        let  	arr = Arr::New( unsafe { std::ptr::NonNull::new_unchecked(self.Ptr().cast_mut()) }, self.Size());
        move |worker: &dyn IWorker| {
            let  	lessFn = move |i, j| less( arr.At( i), arr.At( j));
            let  	swapFn = move |i, j| arr.Swap( i, j);
            arr.USeg().DoQSort( worker, lessFn, swapFn);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SortSanity< Less>( &self, less: Less) -> bool
    where
        Less: Fn( &T, &T) -> bool + Send + Sync + Clone + 'a,
    {
        self.USeg().RSnip( 1).Span( |k| !less( self.At( k + 1), self.At( k)))
    }
}

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

impl< 'a, T> IAccess< 'a, T> for Arr< 'a, T>
{
    fn	Size( &self) -> U32
    {
        self._Size
    }

    fn	At< K: Into< U32>>( &self, k: K) -> &'a T
    {
        unsafe { &*self.Ptr().add( k.into().AsUsize()) }
    }
}

impl< 'a, T> IArr< 'a, T> for Arr< 'a, T> {
    fn	Ptr( &self) -> *const T
    {
        self._Ptr.as_ptr()
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
