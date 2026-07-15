//-- buff.rs ----------------------------------------------------------------------------------------------------------------------
use	std::{ alloc::realloc, marker::PhantomData, mem::{ forget, size_of, swap }, ptr::{ copy_nonoverlapping, drop_in_place, read, slice_from_raw_parts_mut, write } };
use	crate::flux::{ IFluxImportSource, fluximport::FieldImp };
use	crate::silo::{ Arr, IAccess, IArr, U32 };
use	std::alloc::{ Layout, alloc, dealloc, handle_alloc_error };

use	std::ops::{ Deref, DerefMut };
use	std::ptr::NonNull;

//---------------------------------------------------------------------------------------------------------------------------------

/// Panic-safe guard for freshly allocated, partially-initialized memory.
/// On drop (i.e. during a panic), it drops the already-initialized elements
/// and then deallocates the raw memory.
struct InitGuard< T>
{
    _Ptr: *mut T,
    _Layout: Layout,
    _InitCount: usize,
}

impl< T> Drop for InitGuard< T>
{
    fn	drop( &mut self)
    {
        unsafe {
            if self._InitCount > 0 {
                let  	slicePtr =
                    slice_from_raw_parts_mut( self._Ptr, self._InitCount);
                drop_in_place( slicePtr);
            }
            dealloc( self._Ptr as *mut u8, self._Layout);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Allocate `size` elements, initialize via `f(index)`, return the NonNull slice pointer.
/// Panic-safe: if `f` panics, already-initialized elements are dropped and memory is freed.
unsafe fn	AllocInit< T, F>( size: usize, f: F) -> NonNull< [T]>
where
    F: Fn( usize) -> T,
{
    unsafe {
        let  	layout = Layout::array::< T>( size).expect( "Layout calculation failed");
        let  	rawPtr = alloc( layout) as *mut T;
        if rawPtr.is_null() {
            handle_alloc_error( layout);
        }
        let  	mut guard = InitGuard {
            _Ptr: rawPtr,
            _Layout: layout,
            _InitCount: 0,
        };
        for i in 0..size {
            write( rawPtr.add( i), f( i));
            guard._InitCount += 1;
        }
        forget( guard);
        let  	nonNullPtr = NonNull::new_unchecked( rawPtr);
        NonNull::slice_from_raw_parts( nonNullPtr, size)
    }
}
//---------------------------------------------------------------------------------------------------------------------------------

impl<T> IFluxImportSource for Buff<T>
where
    T: IFluxImportSource + Default,
{
    fn	FetchFieldImp< 'b>( &'b mut self, field: &mut FieldImp< 'b>)
    {
        let  	mut idx = 0usize;
        let  	ptr = self as *mut Self;
        *field = FieldImp::Arr( Box::new( move |item| {
            let  	buff = unsafe { &mut *ptr };
            if idx >= buff._Ptr.len() {
                buff.Push( T::default());
            }
            let  	elem = unsafe { &mut *buff._Ptr.as_ptr().cast::<T>().add( idx) };
            *item = FieldImp::FluxSource( elem);
            idx += 1;
            true
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
pub struct Buff< T>
{
    pub(crate) _Ptr: NonNull< [T]>,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl< T: Send> Send for Buff< T>
{
}
unsafe impl< T: Sync> Sync for Buff< T>
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Buff< T>
{
    pub fn	NewEmpty() -> Self
    {
        Self {
            _Ptr: NonNull::slice_from_raw_parts( NonNull::dangling(), 0),
        }
    }
    pub fn	Push( &mut self, val: T)
    {
        let  	oldSize = self._Ptr.len();
        let  	newSize = oldSize + 1;
        let  	isZst = size_of::< T>() == 0;
        if isZst {
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), newSize);
            return;
        }
        unsafe {
            let  	oldLayout = Layout::array::< T>( oldSize).unwrap();
            let  	newLayout = Layout::array::< T>( newSize).unwrap();
            let  	rawPtr = if oldSize == 0 {
                alloc( newLayout)
            } else {
                realloc( self._Ptr.cast::< u8>().as_ptr(), oldLayout, newLayout.size())
            };
            if rawPtr.is_null() {
                handle_alloc_error( newLayout);
            }
            let  	rawPtrT = rawPtr as *mut T;
            write( rawPtrT.add( oldSize), val);
            let  	nonNullPtr = NonNull::new_unchecked( rawPtrT);
            self._Ptr = NonNull::slice_from_raw_parts( nonNullPtr, newSize);
        }
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    pub fn	Pop( &mut self) -> Option< T>
    {
        let  	oldSize = self._Ptr.len();
        if oldSize == 0 {
            return None;
        }
        let  	newSize = oldSize - 1;
        let  	isZst = size_of::< T>() == 0;
        if isZst {
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), newSize);
            return Some( unsafe { read( NonNull::<T>::dangling().as_ptr()) });
        }
        unsafe {
            let  	rawPtrT = self._Ptr.as_ptr() as *mut T;
            let  	val = read( rawPtrT.add( newSize));
            if newSize == 0 {
                let  	layout = Layout::array::< T>( oldSize).unwrap();
                dealloc( rawPtrT as *mut u8, layout);
                self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), 0);
            } else {
                let  	oldLayout = Layout::array::< T>( oldSize).unwrap();
                let  	newLayout = Layout::array::< T>( newSize).unwrap();
                let  	rawPtr = realloc( rawPtrT as *mut u8, oldLayout, newLayout.size());
                if rawPtr.is_null() {
                    handle_alloc_error( newLayout);
                }
                let  	nonNullPtr = NonNull::new_unchecked( rawPtr as *mut T);
                self._Ptr = NonNull::slice_from_raw_parts( nonNullPtr, newSize);
            }
            Some( val)
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	IsEmpty( &self) -> bool
    {
        self.is_empty()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> U32
    {
        U32( self._Ptr.len() as u32)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Resize< Dispenser>( &mut self, newSize: U32, dispenser: Dispenser)
    where
        Dispenser: Fn( U32) -> T,
    {
        let  	newSizeUsize = usize::from( newSize);
        let  	oldSize = self._Ptr.len();
        if newSizeUsize <= oldSize {
            return;
        }
        let  	isZst = size_of::< T>() == 0;
        if isZst {
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), newSizeUsize);
            return;
        }
        unsafe {
            let  	oldLayout = Layout::array::< T>( oldSize).unwrap();
            let  	newLayout = Layout::array::< T>( newSizeUsize).unwrap();
            let  	rawPtr = if oldSize == 0 {
                alloc( newLayout)
            } else {
                realloc( self._Ptr.cast::< u8>().as_ptr(), oldLayout, newLayout.size())
            };
            if rawPtr.is_null() {
                handle_alloc_error( newLayout);
            }
            let  	rawPtrT = rawPtr as *mut T;
            // Defuse Buff::drop in case of panic during initialization
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), 0);
            struct ResizeGuard< T>
            {
                _RawPtr: *mut u8,
                _NewLayout: Layout,
                _OldSize: usize,
                _InitCount: usize,
                _Phantom: PhantomData< T>,
            }
            impl< T> Drop for ResizeGuard< T>
            {
                fn	drop( &mut self)
                {
                    unsafe {
                        let  	totalValid = self._OldSize + self._InitCount;
                        if totalValid > 0 {
                            let  	slicePtr = slice_from_raw_parts_mut(
                                self._RawPtr as *mut T,
                                totalValid,
                            );
                            drop_in_place( slicePtr);
                        }
                        dealloc( self._RawPtr, self._NewLayout);
                    }
                }
            }
            let  	mut guard = ResizeGuard::< T> {
                _RawPtr: rawPtr,
                _NewLayout: newLayout,
                _OldSize: oldSize,
                _InitCount: 0,
                _Phantom: PhantomData,
            };
            for i in oldSize..newSizeUsize {
                write( rawPtrT.add( i), dispenser( U32( i as u32)));
                guard._InitCount += 1;
            }
            forget( guard);
            let  	nonNullPtr = NonNull::new_unchecked( rawPtrT);
            self._Ptr = NonNull::slice_from_raw_parts( nonNullPtr, newSizeUsize);
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ExtendFromArr( &mut self, arr: Arr<'_, T>)
    where
        T: Copy,
    {
        self.ExtendFromSlice( &*arr);
    }

    pub fn	ExtendFromSlice( &mut self, slice: &[ T])
    where
        T: Copy,
    {
        if slice.is_empty() {
            return;
        }
        let  	oldSize = self._Ptr.len();
        let  	addSize = slice.len();
        let  	newSize = oldSize + addSize;
        let  	isZst = size_of::< T>() == 0;

        if isZst {
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), newSize);
            return;
        }

        unsafe {
            let  	oldLayout = Layout::array::< T>( oldSize).unwrap();
            let  	newLayout = Layout::array::< T>( newSize).unwrap();

            let  	rawPtr = if oldSize == 0 {
                alloc( newLayout)
            } else {
                realloc( self._Ptr.cast::< u8>().as_ptr(), oldLayout, newLayout.size())
            };

            if rawPtr.is_null() {
                handle_alloc_error( newLayout);
            }

            let  	rawPtrT = rawPtr as *mut T;
            copy_nonoverlapping( slice.as_ptr(), rawPtrT.add( oldSize), addSize);

            let  	nonNullPtr = NonNull::new_unchecked( rawPtrT);
            self._Ptr = NonNull::slice_from_raw_parts( nonNullPtr, newSize);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Create< S: Into< U32>, Dispenser>( sz: S, dispenser: Dispenser) -> Self
    where
        Dispenser: Fn( U32) -> T,
    {
        let  	size = sz.into();
        let  	isZst = size_of::< T>() == 0;
        if size == 0 || isZst {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), size.AsUsize());
            return Buff { _Ptr: dangling };
        }
        unsafe {
            Buff { _Ptr: AllocInit( size.AsUsize(), |i| dispenser( U32( i as u32))) }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	New< S: Into< U32>>( sz: S, initialValue: T) -> Self
    where
        T: Clone,
    {
        let  	sz = sz.into();
        Buff::Create( sz, |_| initialValue.clone())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SwapBuff( &mut self, buff: &mut Buff< T>)
    {
        swap( self, buff);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Concat( a: Arr< '_, T>, b: Arr< '_, T>) -> Self
    where
        T: Copy,
    {
        let  	aSz = a.Size().AsUsize();
        let  	bSz = b.Size().AsUsize();
        let  	totalSz = aSz + bSz;
        if totalSz == 0 {
            return Buff::NewEmpty();
        }
        let  	isZst = size_of::< T>() == 0;
        if isZst {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), totalSz);
            return Buff { _Ptr: dangling };
        }
        let  	layout = Layout::array::< T>( totalSz).expect( "Layout calculation failed");
        unsafe {
            let  	rawPtr = alloc( layout) as *mut T;
            if rawPtr.is_null() {
                handle_alloc_error( layout);
            }
            copy_nonoverlapping( a.Ptr(), rawPtr, aSz);
            copy_nonoverlapping( b.Ptr(), rawPtr.add( aSz), bSz);
            let  	nonNullPtr = NonNull::new_unchecked( rawPtr);
            Buff { _Ptr: NonNull::slice_from_raw_parts( nonNullPtr, totalSz) }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Clone> Buff< T>
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Buff< T>
{
    pub fn	Arr< 'a>( &self) -> Arr< 'a, T>
    {
        Arr::New( self._Ptr.cast::< T>(), U32( self._Ptr.len() as u32))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Deref for Buff< T>
{
    type Target = [T];
    fn	deref( &self) -> &Self::Target
    {
        unsafe { self._Ptr.as_ref() }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> DerefMut for Buff< T>
{
    fn	deref_mut( &mut self) -> &mut Self::Target
    {
        unsafe { self._Ptr.as_mut() }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Drop for Buff< T>
{
    fn	drop( &mut self)
    {
        let  	size = self._Ptr.len();
        let  	isZst = size_of::< T>() == 0;
        if size == 0 || isZst {
            return;
        }
        let  	layout = Layout::array::< T>( size).expect( "Too Big");
        unsafe {
            // Drop all elements via slice pointer
            drop_in_place( self._Ptr.as_ptr());
            // Deallocate the contiguous chunk of raw memory
            dealloc( self._Ptr.cast::< u8>().as_ptr(), layout);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Clone> Clone for Buff< T>
{
    fn	clone( &self) -> Self
    {
        let  	size = self._Ptr.len();
        if size == 0 || size_of::< T>() == 0 {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), size);
            return Buff { _Ptr: dangling };
        }
        unsafe {
            Buff { _Ptr: AllocInit( size, |i| self[i].clone()) }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Clone> From< &[T]> for Buff< T>
{
    fn	from( slice: &[T]) -> Self
    {
        let  	size = slice.len();
        if size == 0 || size_of::< T>() == 0 {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), size);
            return Buff { _Ptr: dangling };
        }
        unsafe {
            Buff { _Ptr: AllocInit( size, |i| slice[i].clone()) }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Clone, const N: usize> From< [T; N]> for Buff< T>
{
    fn	from( arr: [T; N]) -> Self
    {
        Self::from( &arr[..])
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Clone> From< Arr< '_, T> > for Buff< T>
{
    fn	from( arr: Arr< '_, T>) -> Self
    {
        Self::from( &*arr)
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: 'a> IAccess< 'a, T> for &'a Buff< T>
{
    fn	Size( &self) -> U32
    {
        U32( self._Ptr.len() as u32)
    }

    fn	At< K: Into< U32>>( &self, k: K) -> &'a T
    {
        unsafe { &*self._Ptr.cast::< T>().as_ptr().add( k.into().AsUsize()) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: 'a> IAccess< 'a, T> for &'a mut Buff< T>
{
    fn	Size( &self) -> U32
    {
        U32( self._Ptr.len() as u32)
    }

    fn	At< K: Into< U32>>( &self, k: K) -> &'a T
    {
        unsafe { &*self._Ptr.cast::< T>().as_ptr().add( k.into().AsUsize()) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: 'a> IArr< 'a, T> for &'a mut Buff< T> {
    fn	Ptr( &self) -> *const T
    {
        self._Ptr.cast::< T>().as_ptr()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! Buff {
    ( $( $x:expr ),* ) => {
        {
            let  	temp = [$( $x ),*];
            $crate::silo::Buff::from( temp)
        }
    };
    ( $( $x:expr ),+ , ) => {
        $crate::Buff![ $( $x ),* ]
    };
    ( $elem:expr ; $n:expr ) => {
        {
            let  	count: u32 = ( $n).try_into().expect( "Count must fit in u32");
            $crate::silo::Buff::New( $crate::silo::U32( count), $elem)
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
