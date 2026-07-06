//-- xflux.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, U16, U32, U64, U8 };

//---------------------------------------------------------------------------------------------------------------------------------
use	crate::silo::IAccess;

pub enum XField< 'a>
{
    Null,
    Str( &'a str),
    String( String),
    U64( u64),
    F64( f64),
    Bool( bool),
    Arr( Box< dyn FnMut( &mut XField< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &mut String, &mut XField< 'a>) -> bool + 'a>),
    FluxSource( &'a dyn IXFluxSource),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IXFluxSink
{
    fn	FromXField( &mut self, field: XField);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IXFluxSource
{
    fn	ToXField< 'a>( &'a self, field: &mut XField< 'a>);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplIXFluxSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl $crate::flux::IXFluxSource for $struct_name
        {
            fn	ToXField< 'a>( &'a self, field: &mut $crate::flux::xflux::XField< 'a>)
            {
                let  	mut step = 0u32;
                let  	obj = self;
                *field = $crate::flux::xflux::XField::Obj( Box::new( move |key, item| {
                    #[allow( unused_variables, unused_assignments)]
                    let  	mut _curr_step = 0u32;
                    $(
                        if step == _curr_step {
                            *key = stringify!( $field).to_string();
                            *item = $crate::flux::xflux::XField::FluxSource( &obj.$field);
                            step += 1;
                            return true;
                        }
                        _curr_step += 1;
                    )*
                    false
                }));
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplIXFluxSourceUInt
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IXFluxSource for $T
            {
                fn	ToXField< 'a>( &'a self, field: &mut XField< 'a>)
                {
                    *field = XField::U64( self.0 as u64);
                }
            }
        )+
    };
}

ImplIXFluxSourceUInt!( U8, U16, U32, U64);

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplIXFluxSourceFloat
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IXFluxSource for $T
            {
                fn	ToXField< 'a>( &'a self, field: &mut XField< 'a>)
                {
                    *field = XField::F64( *self as f64);
                }
            }
        )+
    };
}

ImplIXFluxSourceFloat!( f32, f64);

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IXFluxSource for Arr<'a, T>
where
    T: IXFluxSource,
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut idx = 0u32;
        let  	arr = *self;
        *field = XField::Arr( Box::new( move |item| {
            if idx < arr.Size().0 {
                *item = XField::FluxSource( arr.At( idx));
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
