//-- fluxin.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::{ U64, U32, U16, U8 };

pub enum FieldIn< 'a>
{
    Null,
    Str( &'a mut &'a str),
    String( &'a mut String),
    U64( &'a mut U64),
    F64( &'a mut f64),
    Bool( &'a mut bool),
    Arr( Box< dyn FnMut( &mut FieldIn< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &str, &mut FieldIn< 'a>) -> bool + 'a>),
    FluxSink( &'a mut dyn IFluxInSink),
    FluxSource( &'a mut dyn IFluxInSource),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxInSink
{
    fn	FromFieldIn( &mut self, field: FieldIn) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> FieldIn< 'a>
{
    pub fn Resolve( &mut self)
    {
        let  	mut temp = FieldIn::Null;
        std::mem::swap( self, &mut temp);
        if let FieldIn::FluxSource( src) = temp {
            src.ToFieldIn( self);
        } else {
            *self = temp;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxInSource
{
    fn	ToFieldIn< 'a>( &'a mut self, _field: &mut FieldIn< 'a>)
    {
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'r, T: IFluxInSource + ?Sized> IFluxInSource for &'r mut T
{
    fn	ToFieldIn< 'a>( &'a mut self, field: &mut FieldIn< 'a>)
    {
        ( **self).ToFieldIn( field);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplIFluxInSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl $crate::flux::IFluxInSource for $struct_name
        {
            fn	ToFieldIn< 'a>( &'a mut self, field: &mut $crate::flux::fluxin::FieldIn< 'a>)
            {
                let  	ptr = self as *mut Self;
                *field = $crate::flux::fluxin::FieldIn::Obj( Box::new( move |key, item| {
                    let  	obj = unsafe { &mut *ptr };
                    $(
                        if key == stringify!( $field) {
                            $crate::flux::IFluxInSource::ToFieldIn( &mut obj.$field, item);
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

macro_rules! ImplIFluxInSourceUInt
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxInSource for $T
            {
                fn	ToFieldIn< 'a>( &'a mut self, field: &mut FieldIn< 'a>)
                {
                    *field = FieldIn::FluxSource( self);
                }
            }
        )+
    };
}

ImplIFluxInSourceUInt!( U8, U16, U32, U64);

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplIFluxInSourceFloat
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxInSource for $T
            {
                fn	ToFieldIn< 'a>( &'a mut self, field: &mut FieldIn< 'a>)
                {
                    *field = FieldIn::FluxSource( self);
                }
            }
        )+
    };
}

ImplIFluxInSourceFloat!( f32, f64);

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxInSource for String
{
    fn	ToFieldIn< 'a>( &'a mut self, field: &mut FieldIn< 'a>)
    {
        *field = FieldIn::String( self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'b> IFluxInSource for &'b str
{
    fn	ToFieldIn< 'a>( &'a mut self, field: &mut FieldIn< 'a>)
    {
        // This relies on the fact that 'a and 'b are compatible in Kosh's memory arena usage.
        // We cast the mutable reference to &'a mut &'a str.
        let ptr = self as *mut &'b str as *mut &'a str;
        *field = FieldIn::Str( unsafe { &mut *ptr } );
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
