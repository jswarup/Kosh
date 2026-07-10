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
