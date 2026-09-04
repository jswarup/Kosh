//-- buff.rs ----------------------------------------------------------------------------------------------------------------------
use	std::{
    alloc::{ alloc, dealloc, handle_alloc_error, realloc, Layout },
    fmt,
    marker::PhantomData,
    mem::{ forget, size_of, swap },
    ops::{ Deref, DerefMut, Index, IndexMut },
    ptr::{ copy_nonoverlapping, drop_in_place, NonNull, read, slice_from_raw_parts_mut, write },
};
use	serde::{
    de::{ SeqAccess, Visitor },
    Deserialize, Deserializer, Serialize, Serializer,
};
use	crate::silo::{ Arr, IAccess, IArr, Stash, U32 };

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
    pub fn	New() -> Self
    {
        Self {
            _Ptr: NonNull::slice_from_raw_parts( NonNull::dangling(), 0),
        }
    }


    //---------------------------------------------------------------------------------------------------------------------------------



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

    #[inline]
    pub fn	Slice( &self) -> &[T]
    {
        return self.deref();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	SliceMut( &mut self) -> &mut [T]
    {
        return self.deref_mut();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Truncate< S: Into< U32>>( &mut self, newSize: S)
    {
        let  	newSizeUsize = usize::from( newSize.into());
        let  	oldSize = self._Ptr.len();
        if newSizeUsize >= oldSize {
            return;
        }
        if newSizeUsize == 0 {
            *self = Buff::New();
            return;
        }
        let  	isZst = size_of::< T>() == 0;
        if isZst {
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), newSizeUsize);
            return;
        }
        unsafe {
            for i in newSizeUsize..oldSize {
                drop_in_place( self._Ptr.cast::< T>().as_ptr().add( i));
            }
            let  	oldLayout = Layout::array::< T>( oldSize).unwrap();
            let  	newLayout = Layout::array::< T>( newSizeUsize).unwrap();
            let  	rawPtr = realloc( self._Ptr.cast::< u8>().as_ptr(), oldLayout, newLayout.size());
            if rawPtr.is_null() {
                handle_alloc_error( newLayout);
            }
            let  	rawPtrT = rawPtr as *mut T;
            let  	nonNullPtr = NonNull::new_unchecked( rawPtrT);
            self._Ptr = NonNull::slice_from_raw_parts( nonNullPtr, newSizeUsize);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Resize< S: Into< U32>, Dispenser>( &mut self, newSize: S, dispenser: Dispenser)
    where
        Dispenser: Fn( U32) -> T,
    {
        let  	newSizeUsize = usize::from( newSize.into());
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

    pub fn	SwapBuff( &mut self, buff: &mut Buff< T>)
    {
        swap( self, buff);
    }

    //-----------------------------------------------------------------------------------------------------------------------------



    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Concat( a: Arr< '_, T>, b: Arr< '_, T>) -> Self
    where
        T: Copy,
    {
        let  	aSz = a.Size().AsUsize();
        let  	bSz = b.Size().AsUsize();
        let  	totalSz = aSz + bSz;
        if totalSz == 0 {
            return Buff::New();
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

impl< T, const N: usize> From< [T; N]> for Buff< T>
{
    fn	from( arr: [T; N]) -> Self
    {
        let  	mut stash = Stash::WithCapacity( N as u32);
        for item in IntoIterator::into_iter( arr) {
            stash.Push( item);
        }
        return stash.IntoBuff();
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

impl< T: PartialEq> PartialEq for Buff< T>
{
    fn	eq( &self, other: &Self) -> bool
    {
        self.deref() == other.deref()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Eq> Eq for Buff< T>
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: PartialEq> PartialEq< [T]> for Buff< T>
{
    fn	eq( &self, other: &[T]) -> bool
    {
        self.deref() == other
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: PartialEq, const N: usize> PartialEq< [T; N]> for Buff< T>
{
    fn	eq( &self, other: &[T; N]) -> bool
    {
        self.deref() == other
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: PartialEq> PartialEq< Vec< T>> for Buff< T>
{
    fn	eq( &self, other: &Vec< T>) -> bool
    {
        self.deref() == other.as_slice()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: fmt::Debug> fmt::Debug for Buff< T>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        fmt::Debug::fmt( self.deref(), f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Default for Buff< T>
{
    fn	default() -> Self
    {
        Self::New()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T, I: Into< U32>> Index< I> for Buff< T>
{
    type Output = T;

    #[inline]
    fn	index( &self, index: I) -> &Self::Output
    {
        return &self.deref()[index.into().0 as usize];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T, I: Into< U32>> IndexMut< I> for Buff< T>
{
    #[inline]
    fn	index_mut( &mut self, index: I) -> &mut Self::Output
    {
        return &mut self.deref_mut()[index.into().0 as usize];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> FromIterator< T> for Buff< T>
{
    fn	from_iter< I: IntoIterator< Item = T>>( iter: I) -> Self
    {
        let  	iterator = iter.into_iter();
        let  	mut stash = Stash::New();
        for item in iterator {
            stash.Push( item);
        }
        return stash.IntoBuff();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Serialize> Serialize for Buff< T>
{
    fn	serialize< S>( &self, serializer: S) -> Result< S::Ok, S::Error>
    where
        S: Serializer,
    {
        return self.deref().serialize( serializer);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'de, T: Deserialize< 'de>> Deserialize< 'de> for Buff< T>
{
    fn	deserialize< D>( deserializer: D) -> Result< Self, D::Error>
    where
        D: Deserializer< 'de>,
    {
        struct BuffVisitor< T> {
            marker: PhantomData< fn() -> Buff< T>>,
        }
        impl< 'de, T: Deserialize< 'de>> Visitor< 'de> for BuffVisitor< T> {
            type Value = Buff< T>;

            fn	expecting( &self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str( "a sequence")
            }

            fn	visit_seq< A>( self, mut seq: A) -> Result< Self::Value, A::Error>
            where
                A: SeqAccess< 'de>,
            {
                let  	mut stash = if let Some( size) = seq.size_hint() {
                    Stash::WithCapacity( size as u32)
                } else {
                    Stash::New()
                };

                while let Some( value) = seq.next_element()? {
                    stash.Push( value);
                }

                Ok( stash.IntoBuff())
            }
        }
        return deserializer.deserialize_seq( BuffVisitor { marker: PhantomData });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BuffIter< T>
{
    _Buff:   Buff< T>,
    _Index:  usize,
}

impl< T> Iterator for BuffIter< T>
{
    type Item = T;

    fn	next( &mut self) -> Option< Self::Item>
    {
        if self._Index >= self._Buff._Ptr.len() {
            return None;
        }
        let  	val = unsafe { read( self._Buff._Ptr.as_ptr().cast::< T>().add( self._Index)) };
        self._Index += 1;
        Some( val)
    }
}

impl< T> Drop for BuffIter< T>
{
    fn	drop( &mut self)
    {
        unsafe {
            let  	len = self._Buff._Ptr.len();
            for i in self._Index..len {
                drop_in_place( self._Buff._Ptr.as_ptr().cast::< T>().add( i));
            }
            if len > 0 && size_of::< T>() > 0 {
                let  	layout = Layout::array::< T>( len).unwrap();
                dealloc( self._Buff._Ptr.cast::< u8>().as_ptr(), layout);
            }
            forget( std::mem::replace( &mut self._Buff, Buff::New()));
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> IntoIterator for Buff< T>
{
    type Item = T;
    type IntoIter = BuffIter< T>;

    fn	into_iter( self) -> Self::IntoIter
    {
        BuffIter { _Buff: self, _Index: 0 }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> IntoIterator for &'a Buff< T>
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter< 'a, T>;

    fn	into_iter( self) -> Self::IntoIter
    {
        self.deref().iter()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> IntoIterator for &'a mut Buff< T>
{
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut< 'a, T>;

    fn	into_iter( self) -> Self::IntoIter
    {
        self.deref_mut().iter_mut()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! Buff {
    () => {
        $crate::silo::Buff::New()
    };
    ( $( $x:expr ),* ) => {
        $crate::silo::Buff::from( [ $( $x ),* ] )
    };
    ( $( $x:expr ),+ , ) => {
        $crate::Buff![ $( $x ),* ]
    };
    ( $elem:expr ; $n:expr ) => {
        {
            let  	count: u32 = ( $n).try_into().expect( "Count must fit in u32");
            $crate::silo::Buff::Create( $crate::silo::U32( count), |_| $elem)
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
