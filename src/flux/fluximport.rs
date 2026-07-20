//-- fluximport.rs -----------------------------------------------------------------------------------------------------------------------

use crate::silo::U64;


#[derive( Default)]
pub enum FieldImp< 'a>
{
    #[default]
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
    pub fn	Resolve( &mut self)
    {
        let  	mut temp = FieldImp::Null;
        std::mem::swap( self, &mut temp);
        if let FieldImp::FluxSource( src) = temp {
            src.FetchFieldImp( self);
        } else {
            *self = temp;
        }
    }

    pub fn	PostU64( mut self, val: U64)
    {
        self.Resolve();
        if let      FieldImp::U64( dst) = self {
            *dst = val;
        } else if let      FieldImp::FluxSink( flx) = self {
            let      mut temp = val;
            flx.FromFieldImp( FieldImp::U64( &mut temp));
        }
    }

    pub fn	PostF64( mut self, val: f64)
    {
        self.Resolve();
        if let      FieldImp::F64( dst) = self {
            *dst = val;
        } else if let      FieldImp::U64( dst) = self {
            *dst = U64( val as u64);
        } else if let      FieldImp::FluxSink( flx) = self {
            let      mut temp = val;
            flx.FromFieldImp( FieldImp::F64( &mut temp));
        }
    }

    pub fn	PostStr( mut self, val: &'a str)
    {
        self.Resolve();
        if let      FieldImp::Str( dst) = self {
            *dst = val;
        } else if let      FieldImp::String( dst) = self {
            *dst = val.to_string();
        } else if let      FieldImp::FluxSink( flx) = self {
            let      mut temp = val;
            flx.FromFieldImp( FieldImp::Str( &mut temp));
        }
    }

    pub fn	PostBool( mut self, val: bool)
    {
        self.Resolve();
        if let      FieldImp::Bool( dst) = self {
            *dst = val;
        } else if let      FieldImp::FluxSink( flx) = self {
            let      mut temp = val;
            flx.FromFieldImp( FieldImp::Bool( &mut temp));
        }
    }

    pub fn	PostParsed( mut self, s: &'a str)
    {
        self.Resolve();
        if let Ok( v) = s.parse::< u64>() {
            self.PostU64( U64( v));
        } else if let Ok( v) = s.parse::< f64>() {
            self.PostF64( v);
        } else if let Ok( v) = s.parse::< bool>() {
            self.PostBool( v);
        } else {
            self.PostStr( s);
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


