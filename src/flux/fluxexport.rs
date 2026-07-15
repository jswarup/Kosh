//-- fluxexport.rs -----------------------------------------------------------------------------------------------------------------------
use	std::fmt;

use	super::JsonOutStream;
use	crate::silo::{ Arr, IAccess, U16, U32, U64, U8, USeg };

pub enum FieldExp< 'a>
{
    Null,
    Str( &'a str),
    String( String),
    U64( U64),
    F64( f64),
    Bool( bool),
    Arr( Box< dyn FnMut( &mut FieldExp< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &mut String, &mut FieldExp< 'a>) -> bool + 'a>),
    FluxSource( &'a dyn IFluxExportSource),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxExportSink
{
    fn	DispatchFieldExp( &mut self, field: FieldExp);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxExportSource
{
    fn	FetchFieldExp< 'a>( &'a self, _field: &mut FieldExp< 'a>)
    {
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'r, T: IFluxExportSource + ?Sized> IFluxExportSource for &'r T
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        ( **self).FetchFieldExp( field);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplFluxExportSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl IFluxExportSource for $struct_name
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
            {
                let  	mut step = 0u32;
                let  	obj = self;
                *field = FieldExp::Obj( Box::new( move |key, item| {
                    #[allow( unused_variables, unused_assignments)]
                    let  	mut _curr_step = 0u32;
                    $(
                        if step == _curr_step {
                            *key = stringify!( $field).to_string();
                            *item = FieldExp::FluxSource( &obj.$field);
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

macro_rules! ImplFluxExportSourceUInt
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxExportSource for $T
            {
                fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
                {
                    *field = FieldExp::U64(  U64::From( self.0 as u64));
                }
            }
        )+
    };
}

ImplFluxExportSourceUInt!( U8, U16, U32, U64);

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplFluxExportSourceFloat
{
    ( $( $T:ty ),+ ) =>
    {
        $(
            impl IFluxExportSource for $T
            {
                fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
                {
                    *field = FieldExp::F64( *self as f64);
                }
            }
        )+
    };
}

ImplFluxExportSourceFloat!( f32, f64);

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for String {
    fn FetchFieldExp<'a>(&'a self, field: &mut FieldExp<'a>) {
        *field = FieldExp::Str(self.as_str());
    }
}

impl IFluxExportSource for str {
    fn FetchFieldExp<'a>(&'a self, field: &mut FieldExp<'a>) {
        *field = FieldExp::Str(self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IFluxExportSource for Arr<'a, T>
where
    T: IFluxExportSource,
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut idx = 0u32;
        let  	arr = *self;
        *field = FieldExp::Arr( Box::new( move |item| {
            if idx < arr.Size().0 {
                *item = FieldExp::FluxSource( arr.At( idx));
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for USeg
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        let  	mut step = 0u32;
        let  	uSeg = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "First".to_string();
                *item = FieldExp::FluxSource( &uSeg._First);
                step += 1;
                return true;
            }
            if step == 1 {
                *key = "Last".to_string();
                *item = FieldExp::FluxSource( &uSeg._Last);
                step += 1;
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for dyn IFluxExportSource + 'a
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        let  	mut output = String::new();
        {
            let  	mut jsonStream = JsonOutStream::New( &mut output, false);
            jsonStream.DispatchFieldExp( FieldExp::FluxSource( self));
        }
        return write!( f, "{}", output);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Debug for dyn IFluxExportSource + 'a
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        let  	mut output = String::new();
        {
            let  	mut jsonStream = JsonOutStream::New( &mut output, true);
            jsonStream.DispatchFieldExp( FieldExp::FluxSource( self));
        }
        return write!( f, "{}", output);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

