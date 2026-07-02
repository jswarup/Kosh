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
