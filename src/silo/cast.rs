//-- silo/cast.rs -----------------------------------------------------------------------------------------------------------------

pub trait ICastExt: Sized
{
    /// Casts a value to another type, asserting size equivalence at runtime in debug mode.
    /// Acts as a postfix wrapper around std::mem::transmute.
    fn	Cast< U>( self) -> U;
}

impl< T: Sized> ICastExt for T
{
    #[inline( always)]
    fn	Cast< U>( self) -> U
    {
        debug_assert_eq!( std::mem::size_of::< T>(), std::mem::size_of::< U>(), "Cast size mismatch");
        let  	res = unsafe { std::mem::transmute_copy( &self) };
        std::mem::forget( self);
        res
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IPtrExt
{
    // Casts a raw pointer to another raw pointer type. Primarily used for transmuting lifetimes of fat pointers.
    fn	CastLife< U: ?Sized>( self) -> *mut U;
}

impl< T: ?Sized> IPtrExt for *mut T
{
    #[inline( always)]
    fn	CastLife< U: ?Sized>( self) -> *mut U
    {
        unsafe { std::mem::transmute_copy( &self) }
    }
}

pub trait IConstPtrExt
{
    // Casts a raw pointer to another raw pointer type. Primarily used for transmuting lifetimes of fat pointers.
    fn	CastLife< U: ?Sized>( self) -> *const U;
}

impl< T: ?Sized> IConstPtrExt for *const T
{
    #[inline( always)]
    fn	CastLife< U: ?Sized>( self) -> *const U
    {
        unsafe { std::mem::transmute_copy( &self) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IConstPtrMutRefExt< T: ?Sized>
{
    fn	MutRef< 'a>( self) -> &'a mut T;
}

impl< T: ?Sized> IConstPtrMutRefExt< T> for *const T
{
    #[inline( always)]
    #[allow( invalid_reference_casting)]
    fn	MutRef< 'a>( self) -> &'a mut T
    {
        unsafe { &mut *( self as *mut T) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Converts a mutable raw pointer into a mutable reference.
pub trait IPtrRefExt< T: ?Sized>
{
    fn	MutRef< 'a>( self) -> &'a mut T;
}

impl< T: ?Sized> IPtrRefExt< T> for *mut T
{
    #[inline( always)]
    fn	MutRef< 'a>( self) -> &'a mut T
    {
        unsafe { &mut *self }
    }
}

/// Converts a raw const pointer into a shared reference.
pub trait IConstPtrRefExt< T: ?Sized>
{
    fn	Ref< 'a>( self) -> &'a T;
}

impl< T: ?Sized> IConstPtrRefExt< T> for *const T
{
    #[inline( always)]
    fn	Ref< 'a>( self) -> &'a T
    {
        unsafe { &*self }
    }
}

/// Accesses elements through a mutable raw pointer.
pub trait IPtrAtExt< T>
{
    fn	RefAt< 'a>( self, index: usize) -> &'a T;
    fn	MutRefAt< 'a>( self, index: usize) -> &'a mut T;
}

impl< T> IPtrAtExt< T> for *mut T
{
    #[inline( always)]
    fn	RefAt< 'a>( self, index: usize) -> &'a T
    {
        unsafe { &*self.add( index) }
    }

    #[inline( always)]
    fn	MutRefAt< 'a>( self, index: usize) -> &'a mut T
    {
        unsafe { &mut *self.add( index) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IAllocRawExt
{
    // Allocates a value on the heap and returns a raw pointer to it.
    fn	AllocRaw( self) -> *mut Self;
}

impl< T> IAllocRawExt for T
{
    #[inline( always)]
    fn	AllocRaw( self) -> *mut Self
    {
        Box::into_raw( Box::new( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IVoidPtrExt
{
    fn	MutRef< 'a, T>( self) -> &'a mut T;
    fn	Ref< 'a, T>( self) -> &'a T;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IVoidPtrExt for *mut ()
{
    #[inline( always)]
    fn	MutRef< 'a, T>( self) -> &'a mut T
    {
        unsafe { &mut *( self as *mut T) }
    }

    #[inline( always)]
    fn	Ref< 'a, T>( self) -> &'a T
    {
        unsafe { &*( self as *const T) }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
