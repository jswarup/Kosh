//-- slice.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, U32, USeg };
use	crate::stalks::IWorker;
use	std::ops::{ Deref, DerefMut };
use	std::ptr::NonNull;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ISlice< 'a, T: 'a>: Deref< Target = [T]> + DerefMut {
    fn	Size( &self) -> U32;
    fn	Ptr( &self) -> NonNull< T>;

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
            let  	ptr = self.Ptr().as_ptr().add( k.into().AsUsize());
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	MutAt< K: Into< U32>>( &self, k: K) -> &'a mut T
    {
        unsafe {
            let  	ptr = self.Ptr().as_ptr().add( k.into().AsUsize());
            &mut *ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SetAt< K: Into< U32>>( &self, k: K, a: &T) -> &'a T
    where
        T: Clone,
    {
        unsafe {
            let  	ptr = self.Ptr().as_ptr().add( k.into().AsUsize());
            *ptr = a.clone();
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	SwapAt< K: Into< U32>>( &self, k: K, a: &mut T) -> &'a T
    {
        unsafe {
            let  	ptr = self.Ptr().as_ptr().add( k.into().AsUsize());
            std::ptr::swap( ptr, a);
            &*ptr
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Swap< I: Into< U32>, J: Into< U32>>( &self, i: I, j: J)
    {
        unsafe {
            std::ptr::swap( 
                self.Ptr().add( i.into().AsUsize()).as_ptr(),
                self.Ptr().add( j.into().AsUsize()).as_ptr(),
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
                src.Ptr().as_ptr().add( srcStart.into().AsUsize()),
                self.Ptr().as_ptr().add( dstStart.into().AsUsize()),
                count.AsUsize(),
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	LSnip< C: Into< U32>>( &self, count: C) -> Arr< 'a, T>
    {
        let  	cnt = count.into();
        Arr::New( 
            unsafe { self.Ptr().add( cnt.AsU32() as usize) },
            self.Size() - cnt,
        )
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	RSnip< C: Into< U32>>( &self, count: C) -> Arr< 'a, T>
    {
        let  	cnt = count.into();
        Arr::New( self.Ptr(), self.Size() - cnt)
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

    fn	QuickSorter< Less>( &self, less: Less) -> impl Fn( &dyn IWorker) + Send + Sync + 'a
    where
        Less: Fn( &T, &T) -> bool + Send + Sync + 'a + Copy,
        T: Send + Sync + 'a,
    {
        let  	arr = Arr::New( self.Ptr(), self.Size());
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

     fn	Ptr( &self) -> NonNull< T>
     {
         self._Ptr
     }
}


