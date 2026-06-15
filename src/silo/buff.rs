//-- buff.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, U32 };
use	std::alloc::{ Layout, alloc, dealloc, handle_alloc_error };
use	std::mem::swap;
use	std::ops::{ Deref, DerefMut };
use	std::ptr::NonNull;

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
        let  	isZst = std::mem::size_of::< T>() == 0;
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
                std::alloc::realloc( self._Ptr.cast::< u8>().as_ptr(), oldLayout, newLayout.size())
            };
            if rawPtr.is_null() {
                handle_alloc_error( newLayout);
            }
            let  	rawPtrT = rawPtr as *mut T;
            std::ptr::write( rawPtrT.add( oldSize), val);
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
        let  	isZst = std::mem::size_of::< T>() == 0;
        if isZst {
            self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), newSize);
            return Some( unsafe { std::ptr::read( NonNull::<T>::dangling().as_ptr()) });
        }
        unsafe {
            let  	rawPtrT = self._Ptr.as_ptr() as *mut T;
            let  	val = std::ptr::read( rawPtrT.add( newSize));
            if newSize == 0 {
                let  	layout = Layout::array::< T>( oldSize).unwrap();
                dealloc( rawPtrT as *mut u8, layout);
                self._Ptr = NonNull::slice_from_raw_parts( NonNull::dangling(), 0);
            } else {
                let  	oldLayout = Layout::array::< T>( oldSize).unwrap();
                let  	newLayout = Layout::array::< T>( newSize).unwrap();
                let  	rawPtr = std::alloc::realloc( rawPtrT as *mut u8, oldLayout, newLayout.size());
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
        let  	isZst = std::mem::size_of::< T>() == 0;
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
                std::alloc::realloc( self._Ptr.cast::< u8>().as_ptr(), oldLayout, newLayout.size())
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
                _Phantom: std::marker::PhantomData< T>,
            }
            impl< T> Drop for ResizeGuard< T>
            {
                fn	drop( &mut self)
                {
                    unsafe {
                        let  	totalValid = self._OldSize + self._InitCount;
                        if totalValid > 0 {
                            let  	slicePtr = std::ptr::slice_from_raw_parts_mut(
                                self._RawPtr as *mut T,
                                totalValid,
                            );
                            std::ptr::drop_in_place( slicePtr);
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
                _Phantom: std::marker::PhantomData,
            };
            for i in oldSize..newSizeUsize {
                std::ptr::write( rawPtrT.add( i), dispenser( U32( i as u32)));
                guard._InitCount += 1;
            }
            std::mem::forget( guard);
            let  	nonNullPtr = NonNull::new_unchecked( rawPtrT);
            self._Ptr = NonNull::slice_from_raw_parts( nonNullPtr, newSizeUsize);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Create< S: Into< U32>, Dispenser>( sz: S, dispenser: Dispenser) -> Self
    where
        Dispenser: Fn( U32) -> T,
    {
        let  	size = sz.into();
        let  	isZst = std::mem::size_of::< T>() == 0;
        if size == 0 || isZst {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), size.AsUsize());
            return Buff { _Ptr: dangling };
        }
        // Calculate layout for an array of T with length `size`
        let  	layout = Layout::array::< T>( size.AsUsize()).expect( "Layout calculation failed");
        unsafe {
            let  	rawPtr = alloc( layout) as *mut T;                 // Allocate memory
            if rawPtr.is_null() {
                handle_alloc_error( layout);
            }
            // Drop guard to prevent resource leaks if initialValue.clone() panics during loop
            struct RawAllocationGuard< T>
            {
                _Ptr: *mut T,
                _Layout: Layout,
                _InitCount: usize,
            }
            impl< T> Drop for RawAllocationGuard< T>
            {
                fn	drop( &mut self)
                {
                    unsafe {
                        if self._InitCount > 0 {
                            let  	slicePtr =
                                std::ptr::slice_from_raw_parts_mut( self._Ptr, self._InitCount);
                            std::ptr::drop_in_place( slicePtr);        // Drop already initialized elements
                        }
                        dealloc( self._Ptr as *mut u8, self._Layout);  // Deallocate the contiguous chunk of raw memory
                    }
                }
            }
            let  	mut guard = RawAllocationGuard {
                _Ptr: rawPtr,
                _Layout: layout,
                _InitCount: 0,
            };
            for i in 0..size.AsUsize()
            // Initialize each element in the contiguous memory block
            {
                std::ptr::write( rawPtr.add( i), dispenser( U32( i as u32)));
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new( guard);                   // Defuse the guard so memory/elements aren't cleaned up when exiting the block
            let  	nonNullPtr = NonNull::new_unchecked( rawPtr);
            let  	slicePtr = NonNull::slice_from_raw_parts( nonNullPtr, size.AsUsize());
            Buff { _Ptr: slicePtr }
        }
    }
    pub fn	New< S: Into< U32>>( sz: S, initialValue: T) -> Self
    where
        T: Clone,
    {
        let  	sz = sz.into();
        Buff::Create( sz, |_| initialValue.clone())
    }
    pub fn	SwapBuff( &mut self, buff: &mut Buff< T>)
    {
        swap( self, buff);
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
        let  	isZst = std::mem::size_of::< T>() == 0;
        if size == 0 || isZst {
            return;
        }
        let  	layout = Layout::array::< T>( size).expect( "Too Big");
        unsafe {
            // Drop all elements via slice pointer
            std::ptr::drop_in_place( self._Ptr.as_ptr());
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
        if size == 0 || std::mem::size_of::< T>() == 0 {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), size);
            return Buff { _Ptr: dangling };
        }
        let  	layout = Layout::array::< T>( size).expect( "Layout calculation failed");
        unsafe {
            let  	rawPtr = alloc( layout) as *mut T;
            if rawPtr.is_null() {
                handle_alloc_error( layout);
            }
            // Panic guard – same pattern as Buff::new
            struct CloneGuard< T>
            {
                _Ptr: *mut T,
                _Layout: Layout,
                _InitCount: usize,
            }
            impl< T> Drop for CloneGuard< T>
            {
                fn	drop( &mut self)
                {
                    unsafe {
                        if self._InitCount > 0 {
                            let  	slicePtr =
                                std::ptr::slice_from_raw_parts_mut( self._Ptr, self._InitCount);
                            std::ptr::drop_in_place( slicePtr);
                        }
                        dealloc( self._Ptr as *mut u8, self._Layout);
                    }
                }
            }
            let  	mut guard = CloneGuard {
                _Ptr: rawPtr,
                _Layout: layout,
                _InitCount: 0,
            };
            for i in 0..size {
                std::ptr::write( rawPtr.add( i), self[i].clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new( guard);
            let  	nonNullPtr = NonNull::new_unchecked( rawPtr);
            let  	slicePtr = NonNull::slice_from_raw_parts( nonNullPtr, size);
            Buff { _Ptr: slicePtr }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Clone> From< &[T]> for Buff< T>
{
    fn	from( slice: &[T]) -> Self
    {
        let  	size = slice.len();
        if size == 0 || std::mem::size_of::< T>() == 0 {
            let  	dangling = NonNull::slice_from_raw_parts( NonNull::dangling(), size);
            return Buff { _Ptr: dangling };
        }
        let  	layout = Layout::array::< T>( size).expect( "Layout calculation failed");
        unsafe {
            let  	rawPtr = alloc( layout) as *mut T;
            if rawPtr.is_null() {
                handle_alloc_error( layout);
            }
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
                                std::ptr::slice_from_raw_parts_mut( self._Ptr, self._InitCount);
                            std::ptr::drop_in_place( slicePtr);
                        }
                        dealloc( self._Ptr as *mut u8, self._Layout);
                    }
                }
            }
            let  	mut guard = InitGuard {
                _Ptr: rawPtr,
                _Layout: layout,
                _InitCount: 0,
            };
            #[allow( clippy::needless_range_loop)]
            for i in 0..size {
                std::ptr::write( rawPtr.add( i), slice[i].clone());
                guard._InitCount += 1;
            }
            _ = std::mem::ManuallyDrop::new( guard);
            let  	nonNullPtr = NonNull::new_unchecked( rawPtr);
            let  	slicePtr = NonNull::slice_from_raw_parts( nonNullPtr, size);
            Buff { _Ptr: slicePtr }
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

