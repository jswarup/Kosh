//-- fluximport.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::{ U64, U32, U16, U8 };


pub enum FieldImp< 'a>
{
    Null,
    Str( &'a mut &'a str),
    String( &'a mut String),
    U64( &'a mut U64),
    F64( &'a mut f64),
    Bool( &'a mut bool),
    Arr( Box< dyn FnMut( &mut FieldImp< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &str, &mut FieldImp< 'a>) -> bool + 'a>),
    FluxSink( &'a mut dyn IFluxImportSink),
    FluxSource( &'a mut dyn IFluxImportSource),
    ExpectedType( &'static str),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxImportSink
{
    fn	FromFieldImp( &mut self, field: FieldImp) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> FieldImp< 'a>
{
    pub fn Resolve( &mut self)
    {
        let  	mut temp = FieldImp::Null;
        std::mem::swap( self, &mut temp);
        if let FieldImp::FluxSource( src) = temp {
            src.FetchFieldImp( self);
        } else {
            *self = temp;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxImportSource
{
    fn	FetchFieldImp< 'a>( &'a mut self, _field: &mut FieldImp< 'a>)
    {
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'r, T: IFluxImportSource + ?Sized> IFluxImportSource for &'r mut T
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        ( **self).FetchFieldImp( field);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplFluxImportSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl IFluxImportSource for $struct_name
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
            {
                let  	ptr = self as *mut Self;
                *field = FieldImp::Obj( Box::new( move |key, item| {
                    let  	obj = unsafe { &mut *ptr };
                    let _ = &obj; let _ = &key; let _ = &item;
                    $(
                        if key == stringify!( $field) {
                            IFluxImportSource::FetchFieldImp( &mut obj.$field, item);
                            return true;
                        }
                    )*
                    false
                }));
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplFluxImportSourceTyped
{
    ( $struct_name:ident, $type_name:literal $( , $field:ident )* ) =>
    {
        impl IFluxImportSource for $struct_name
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
            {
                let  	ptr = self as *mut Self;
                *field = FieldImp::Obj( Box::new( move |key, item| {
                    let  	obj = unsafe { &mut *ptr };
                    let _ = &obj; let _ = &key; let _ = &item;
                    if key == "Type" {
                        *item = FieldImp::ExpectedType( $type_name);
                        return true;
                    }
                    $(
                        if key == stringify!( $field) {
                            IFluxImportSource::FetchFieldImp( &mut obj.$field, item);
                            return true;
                        }
                    )*
                    false
                }));
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplFluxImportSourceUInt
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxImportSource for $T
            {
                fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
                {
                    *field = FieldImp::FluxSource( self);
                }
            }
        )+
    };
}

ImplFluxImportSourceUInt!( U8, U16, U32, U64);

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplFluxImportSourceFloat
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxImportSource for $T
            {
                fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
                {
                    *field = FieldImp::FluxSource( self);
                }
            }
        )+
    };
}

ImplFluxImportSourceFloat!( f32, f64);

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxImportSource for String
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        *field = FieldImp::String( self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'b> IFluxImportSource for &'b str
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        // This relies on the fact that 'a and 'b are compatible in Kosh's memory arena usage.
        // We cast the mutable reference to &'a mut &'a str.
        let ptr = self as *mut &'b str as *mut &'a str;
        *field = FieldImp::Str( unsafe { &mut *ptr } );
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxImportSource for crate::silo::USeg
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        let  	ptr = self as *mut Self;
        *field = FieldImp::Obj( Box::new( move |key, item| {
            let  	obj = unsafe { &mut *ptr };
            if key == "First" {
                *item = FieldImp::FluxSource( &mut obj._First);
                return true;
            }
            if key == "Last" {
                *item = FieldImp::FluxSource( &mut obj._Last);
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IFluxImportSource for crate::silo::Arr<'a, T>
where
    T: IFluxImportSource,
{
    fn	FetchFieldImp< 'b>( &'b mut self, field: &mut FieldImp< 'b>)
    {
        let  	mut idx = 0u32;
        let  	ptr = self as *mut Self;
        *field = FieldImp::Arr( Box::new( move |item| {
            let  	arr = unsafe { &mut *ptr };
            if idx < arr._Size.0 {
                let  	elem = unsafe { &mut *arr._Ptr.as_ptr().add( idx as usize) };
                *item = FieldImp::FluxSource( elem);
                idx += 1;
                true
            } else {
                assert!( idx < arr._Size.0, "Arr capacity exceeded during import. Use Buff instead.");
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
