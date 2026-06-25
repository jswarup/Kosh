//-- xflux.rs -----------------------------------------------------------------------------------------------------------------------
 
//---------------------------------------------------------------------------------------------------------------------------------
use	crate::silo::IAccess;
 
pub enum XField< 'a>
{
    Str( &'a str),
    String( String),
    U64( u64),
    F64( f64),
    Bool( bool),
    Arr( Box< dyn FnMut( &mut XField< 'a>) -> bool + 'a>),
    Obj( Box< dyn FnMut( &mut String, &mut XField< 'a>) -> bool + 'a>),
    Null,
    Fluxable( &'a dyn IXFluxable),
}

//---------------------------------------------------------------------------------------------------------------------------------
 
pub trait IXFlux 
{
    fn	OStream( &mut self) -> &mut dyn std::fmt::Write;

    fn	Field( &mut self, field: XField); 

    fn	KeyField( &mut self, _key: &str, _value: XField< '_>) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------
 
pub trait IXFluxable
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>);
}

//---------------------------------------------------------------------------------------------------------------------------------
 

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U8
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>)
    {
        *field = XField::U64( self.0 as u64);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U16
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>)  
    {
        *field = XField::U64( self.0 as u64);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U32
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>) 
    {
        *field = XField::U64( self.0 as u64);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U64
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>)
    {
        *field = XField::U64( self.0 as u64);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for f32
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>)
    {
        *field = XField::F64( *self as f64);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for f64
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>)
    {
        *field = XField::F64( *self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IXFluxable for crate::silo::Arr<'a, T>
where
    T: IXFluxable,
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut idx = 0u32;
        let  	arr = *self;
        *field = XField::Arr( Box::new( move |item| {
            if idx < arr.Size().0 {
                *item = XField::Fluxable( arr.At( idx));
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
