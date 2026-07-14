//-- fluxout.rs -----------------------------------------------------------------------------------------------------------------------
use	std::fmt;

use	super::JsonOutStream;
use	crate::silo::{ Arr, IAccess, U16, U32, U64, U8, USeg };

pub enum FieldOut< 'a>
{
    Null,
    Str( &'a str),
    String( String),
    U64( U64),
    F64( f64),
    Bool( bool),
    Arr( Box< dyn FnMut( &mut FieldOut< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &mut String, &mut FieldOut< 'a>) -> bool + 'a>),
    FluxSource( &'a dyn IFluxOutSource),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxOutSink
{
    fn	FromFieldOut( &mut self, field: FieldOut);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxOutSource
{
    fn	ToFieldOut< 'a>( &'a self, _field: &mut FieldOut< 'a>)
    {
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'r, T: IFluxOutSource + ?Sized> IFluxOutSource for &'r T
{
    fn	ToFieldOut< 'a>( &'a self, field: &mut FieldOut< 'a>)
    {
        ( **self).ToFieldOut( field);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplIFluxOutSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl $crate::flux::IFluxOutSource for $struct_name
        {
            fn	ToFieldOut< 'a>( &'a self, field: &mut $crate::flux::fluxout::FieldOut< 'a>)
            {
                let  	mut step = 0u32;
                let  	obj = self;
                *field = $crate::flux::fluxout::FieldOut::Obj( Box::new( move |key, item| {
                    #[allow( unused_variables, unused_assignments)]
                    let  	mut _curr_step = 0u32;
                    $(
                        if step == _curr_step {
                            *key = stringify!( $field).to_string();
                            *item = $crate::flux::fluxout::FieldOut::FluxSource( &obj.$field);
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

macro_rules! ImplIFluxOutSourceUInt
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxOutSource for $T
            {
                fn	ToFieldOut< 'a>( &'a self, field: &mut FieldOut< 'a>)
                {
                    *field = FieldOut::U64(  U64::From( self.0 as u64));
                }
            }
        )+
    };
}

ImplIFluxOutSourceUInt!( U8, U16, U32, U64);

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplIFluxOutSourceFloat
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxOutSource for $T
            {
                fn	ToFieldOut< 'a>( &'a self, field: &mut FieldOut< 'a>)
                {
                    *field = FieldOut::F64( *self as f64);
                }
            }
        )+
    };
}

ImplIFluxOutSourceFloat!( f32, f64);

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxOutSource for String {
    fn ToFieldOut<'a>(&'a self, field: &mut FieldOut<'a>) {
        *field = FieldOut::Str(self.as_str());
    }
}

impl IFluxOutSource for str {
    fn ToFieldOut<'a>(&'a self, field: &mut FieldOut<'a>) {
        *field = FieldOut::Str(self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IFluxOutSource for Arr<'a, T>
where
    T: IFluxOutSource,
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>)
    {
        let  	mut idx = 0u32;
        let  	arr = *self;
        *field = FieldOut::Arr( Box::new( move |item| {
            if idx < arr.Size().0 {
                *item = FieldOut::FluxSource( arr.At( idx));
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxOutSource for USeg
{
    fn	ToFieldOut< 'a>( &'a self, field: &mut FieldOut< 'a>)
    {
        let  	mut step = 0u32;
        let  	uSeg = self;
        *field = FieldOut::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "First".to_string();
                *item = FieldOut::FluxSource( &uSeg._First);
                step += 1;
                return true;
            }
            if step == 1 {
                *key = "Last".to_string();
                *item = FieldOut::FluxSource( &uSeg._Last);
                step += 1;
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for dyn IFluxOutSource + 'a
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        let  	mut output = String::new();
        {
            let  	mut jsonStream = JsonOutStream::New( &mut output, false);
            jsonStream.FromFieldOut( FieldOut::FluxSource( self));
        }
        return write!( f, "{}", output);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Debug for dyn IFluxOutSource + 'a
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        let  	mut output = String::new();
        {
            let  	mut jsonStream = JsonOutStream::New( &mut output, true);
            jsonStream.FromFieldOut( FieldOut::FluxSource( self));
        }
        return write!( f, "{}", output);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

