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
    Arr( Box< dyn FnMut( FieldIn< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &str, FieldIn< 'a>) -> bool + 'a>),
    FluxSink( &'a mut dyn IFluxInSink),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IFluxInSink
{
    fn	FromFieldIn( &mut self, field: FieldIn) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------
