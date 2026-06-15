//-- slice.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, U32, USeg };
use	crate::stalks::IWorker;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ISlice< 'a, T: 'a> {
    fn	Size( &self) -> U32;
    fn	Ptr( &self) -> *const T;

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	IsEmpty( &self) -> bool
    {
        self.Size() == U32( 0)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	USeg( &self) -> USeg
    {
        USeg::Create( U32( 0), self.Size())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	At< K: Into< U32>>( &self, k: K) -> &'a T
    {
        unsafe {
            let  	ptr = self.Ptr().add( k.into().AsUsize());
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Span< F>( &self, mut f: F) -> bool
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

pub trait IArr< 'a, T: 'a>: ISlice< 'a, T> {
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

impl< 'a, T> ISlice< 'a, T> for Arr< 'a, T>
{
     fn	Size( &self) -> U32
     {
         self._Size
     }

     fn	Ptr( &self) -> *const T
     {
         self._Ptr.as_ptr()
     }
}

impl< 'a, T> IArr< 'a, T> for Arr< 'a, T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: 'a> ISlice< 'a, T> for &'a [T]
{
    fn Size( &self) -> U32
    {
        U32( self.len() as u32)
    }

    fn Ptr( &self) -> *const T
    {
        self.as_ptr()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: 'a, const N: usize> ISlice< 'a, T> for &'a [T; N]
{
    fn Size( &self) -> U32
    {
        U32( N as u32)
    }

    fn Ptr( &self) -> *const T
    {
        self.as_ptr()
    }
}

