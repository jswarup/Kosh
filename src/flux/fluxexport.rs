//-- fluxexport.rs -----------------------------------------------------------------------------------------------------------------------
use	std::fmt;

use	super::JsonOutStream;
use	crate::silo::U64;

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

