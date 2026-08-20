use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::mem::size_of;
use std::ptr::{copy_nonoverlapping, NonNull, read, write, drop_in_place};
use std::sync::atomic::Ordering;
use crate::silo::access::IAccess;
use crate::stalks::Atm;
use crate::silo::{Arr, Buff, Stk, U32, U8};

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Stash< T>
{
    _Ptr: NonNull< T>,
    _Cap: U32,
    _Sz: Atm< U32>,
}

unsafe impl< T: Send> Send for Stash< T> {}
unsafe impl< T: Sync> Sync for Stash< T> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Stash< T>
{
    pub fn	New() -> Self
    {
        Self {
            _Ptr: NonNull::dangling(),
            _Cap: U32( 0),
            _Sz: Atm::New( U32( 0)),
        }
    }

pub fn	Create< Sz: Into< U32>, SzStk: Into< U32>, Dispenser>(
        sz: Sz,
        szStk: SzStk,
        dispenser: Dispenser,
    ) -> Self
    where
        Dispenser: Fn( U32) -> T,
    {
        let  	capacity = sz.into();
        let  	stack_size = szStk.into();
        let  	mut stash = Self::WithCapacity( capacity);
        for i in 0..stack_size.AsUsize() {
            stash.Push( dispenser( crate::silo::U32( i as u32)));
        }
        stash
    }

    pub fn	FromBuff( buff: Buff<T>, szStk: U32) -> Self
    {
        // To remove Vec, Stash needs to support taking ownership of a Buff
        // Wait, Buff uses NonNull<[T]>, Stash uses NonNull<T>
        // But Stash is our Vec clone, so we can't easily take over a Buff without transmuting or allocating.
        // For simplicity, we just allocate a new Stash and move elements, or extract ptr from Buff.
        // Let's implement this properly:
        let  	cap = buff.Size();
        let  	ptr = buff._Ptr.cast::< T>();
        std::mem::forget( buff);
        Self {
            _Ptr: ptr,
            _Cap: cap,
            _Sz: Atm::New( szStk),
        }
    }

    pub fn	WithCapacity< C: Into< U32>>( cap: C) -> Self
    {
        let  	capacity = cap.into();
        let  	mut stash = Self::New();
        stash.Reserve( capacity);
        stash
    }

    pub fn	WithCapacityVal< C: Into< U32>>( cap: C, fillVal: T) -> Self
    where
        T: Clone,
    {
        let  	capacity = cap.into();
        let  	mut stash = Self::WithCapacity( capacity);
        for _ in 0..capacity.AsUsize() {
            stash.Push( fillVal.clone());
        }
        stash
    }

    pub fn	Size( &self) -> U32
    {
        self._Sz.Load( Ordering::Acquire)
    }

    pub fn	Capacity( &self) -> U32
    {
        self._Cap
    }

    pub fn	Reserve( &mut self, newCap: U32)
    {
        if newCap <= self._Cap || size_of::< T>() == 0 {
            return;
        }
        let  	newCapUsize = newCap.AsUsize();
        let  	layout = Layout::array::< T>( newCapUsize).unwrap();
        unsafe {
            let  	newPtr = if self._Cap == U32( 0) {
                alloc( layout)
            } else {
                let  	oldLayout = Layout::array::< T>( self._Cap.AsUsize()).unwrap();
                realloc( self._Ptr.as_ptr().cast::< u8>(), oldLayout, layout.size())
            };
            if newPtr.is_null() {
                handle_alloc_error( layout);
            }
            self._Ptr = NonNull::new_unchecked( newPtr.cast::< T>());
            self._Cap = newCap;
        }
    }

    pub fn	Clear( &mut self)
    {
        let  	sz = self.Size().AsUsize();
        self._Sz.Store( U32( 0), Ordering::Release);
        unsafe {
            for i in 0..sz {
                drop_in_place( self._Ptr.as_ptr().add( i));
            }
        }
    }

    pub fn	Push( &mut self, val: T)
    {
        let  	sz = self.Size();
        if sz == self._Cap {
            let  	newCap = if self._Cap == U32( 0) { U32( 4) } else { self._Cap * U32( 2) };
            self.Reserve( newCap);
        }
        unsafe {
            write( self._Ptr.as_ptr().add( sz.AsUsize()), val);
        }
        self._Sz.Store( sz + U32( 1), Ordering::Release);
    }

    pub fn	PushVal( &mut self, val: T)
    {
        self.Push( val);
    }

pub fn	Stk( &self) -> Stk< '_, '_, T>
    {
        let  	arr = crate::silo::Arr::New( self._Ptr, self._Cap);
        Stk::Create( &self._Sz, arr)
    }

    pub fn	PushX( &mut self, val: &mut T)
    where
        T: Default
    {
        self.Push( std::mem::take( val));
    }

    pub fn	Pop( &mut self) -> Option< T>
    {
        let  	sz = self.Size();
        if sz == U32( 0) {
            return None;
        }
        let  	newSz = sz - U32( 1);
        self._Sz.Store( newSz, Ordering::Release);
        unsafe {
            Some( read( self._Ptr.as_ptr().add( newSz.AsUsize())))
        }
    }
    
    pub fn	PopToSize( &mut self, targetSz: U32)
    {
        while self.Size() > targetSz {
            self.Pop();
        }
    }

    pub fn	SliceMut( &mut self) -> &mut [T]
    {
        if self.Size() == crate::silo::U32( 0) {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut( self._Ptr.as_ptr(), self.Size().AsUsize()) }
        }
    }

    pub fn	sort_by< F>( &mut self, compare: F)
    where
        F: FnMut( &T, &T) -> std::cmp::Ordering,
    {
        self.SliceMut().sort_by( compare);
    }

    pub fn	AppendStash( &mut self, other: Stash< T>)
    {
        let  	n = other.Size();
        if n == crate::silo::U32( 0) {
            return;
        }
        let  	neededSz = self.Size() + n;
        if neededSz > self._Cap {
            let  	growTo = neededSz.max( self._Cap * crate::silo::U32( 2));
            self.Reserve( growTo);
        }
        let  	startSz = self.Size();
        unsafe {
            std::ptr::copy_nonoverlapping( other._Ptr.as_ptr(), self._Ptr.as_ptr().add( startSz.AsUsize()), n.AsUsize());
        }
        self._Sz.Store( startSz + n, std::sync::atomic::Ordering::Release);
        other._Sz.Store( crate::silo::U32( 0), std::sync::atomic::Ordering::Release);
    }

    pub fn	Arr( &self) -> crate::silo::Arr< '_, T>
    {
        crate::silo::Arr::New( self._Ptr, self.Size())
    }

    pub fn	IntoBuff( self) -> Buff< T>
    {
        let  	sz = self.Size();
        if sz == U32( 0) {
            return Buff::New();
        }
        let  	buff = Buff::Create( sz, |i| unsafe { read( self._Ptr.as_ptr().add( i.AsUsize())) });
        self._Sz.Store( U32( 0), Ordering::Release); // Prevent drop
        buff
    }
    
    pub fn	ToBuff( &self) -> Buff< T>
    where
        T: Clone,
    {
        let  	sz = self.Size();
        Buff::Create( sz, |i| unsafe { (*self._Ptr.as_ptr().add( i.AsUsize())).clone() })
    }

    pub fn	Append( &mut self, arr: Arr< '_, T>)
    where
        T: Clone,
    {
        let  	n = arr.Size();
        if n == U32( 0) {
            return;
        }
        let  	neededSz = self.Size() + n;
        if neededSz > self._Cap {
            let  	growTo = neededSz.max( self._Cap * U32( 2));
            self.Reserve( growTo);
        }
        let  	startSz = self.Size();
        for i in 0..n.AsUsize() {
            unsafe {
                write( self._Ptr.as_ptr().add( startSz.AsUsize() + i), arr[i].clone());
            }
        }
        self._Sz.Store( startSz + n, Ordering::Release);
    }
    
    pub fn	TopMut( &self) -> Option< &mut T>
    {
        let  	sz = self.Size();
        if sz > U32( 0) {
            unsafe { Some( &mut *self._Ptr.as_ptr().add( sz.AsUsize() - 1)) }
        } else {
            None
        }
    }
    
    pub fn	Slice( &self) -> &[T]
    {
        if self.Size() == U32( 0) {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts( self._Ptr.as_ptr(), self.Size().AsUsize()) }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Drop for Stash< T>
{
    fn	drop( &mut self)
    {
        let  	sz = self.Size().AsUsize();
        let  	cap = self._Cap.AsUsize();
        if cap > 0 && size_of::< T>() > 0 {
            unsafe {
                for i in 0..sz {
                    drop_in_place( self._Ptr.as_ptr().add( i));
                }
                let  	layout = Layout::array::< T>( cap).unwrap();
                dealloc( self._Ptr.as_ptr().cast::< u8>(), layout);
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Default for Stash< T>
{
    fn	default() -> Self
    {
        Self::New()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Stash< U8>
{
    pub fn	ReadFrom( &mut self, reader: &mut impl std::io::Read)
    {
        let  	mut temp = [0u8; 4096];
        loop {
            match reader.read( &mut temp) {
                Ok( 0) => break,
                Ok( n) => {
                    let  	neededSz = self.Size() + U32( n as u32);
                    if neededSz > self._Cap {
                        let  	growTo = neededSz.max( self._Cap * U32( 2));
                        self.Reserve( growTo);
                    }
                    unsafe {
                        let  	startSz = self.Size().AsUsize();
                        copy_nonoverlapping( temp.as_ptr(), self._Ptr.as_ptr().add( startSz).cast::< u8>(), n);
                    }
                    self._Sz.Store( self.Size() + U32( n as u32), Ordering::Release);
                }
                Err( _) => break,
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! Stash {
    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.Push($exp);
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.Push($exp);
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                $crate::Stash![@__ $acc, $exp; $($tail)+];
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            $crate::Stash![@__ $acc, $exp; $($tail)+];
        }
    );

    ($exp:expr; $($tail:tt)+) => ({
        let  	mut ret = $crate::silo::Stash::New();
        $crate::Stash![@__ ret, $exp; $($tail)+];
        ret
    });
}

impl< T: Copy> Stash< T>
{
    pub fn	ClearConcurrent( &self)
    {
        self._Sz.Store( crate::silo::U32( 0), std::sync::atomic::Ordering::Release);
    }
}

impl< T> Stash< T>
where
    T: From< usize> + Clone,
{
    pub fn	DoIndexSetup( &self)
    {
        for i in 0..self._Cap.AsUsize() {
            unsafe { std::ptr::write( self._Ptr.as_ptr().add( i), T::from( i)) };
        }
        self._Sz.Store( self._Cap, std::sync::atomic::Ordering::Release);
    }
}

impl< T: Clone + Default> Clone for Stash< T> {
    fn clone( &self) -> Self {
        let mut new_stash = Stash::New();
        new_stash.Reserve( self.Size());
        for i in 0..self.Size().AsUsize() {
            let val = unsafe { &*self._Ptr.as_ptr().add( i) };
            let mut c = val.clone();
            new_stash.PushX( &mut c);
        }
        new_stash
    }
}
