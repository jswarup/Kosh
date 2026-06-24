//-- xflux.rs -----------------------------------------------------------------------------------------------------------------------
 
//---------------------------------------------------------------------------------------------------------------------------------
use	crate::silo::IAccess;
 
pub enum XField< 'a>
{
    Str( &'a str),
    U64( u64),
    F64( f64),
    Bool( bool),
    Arr( &'a mut dyn FnMut( &mut XField< 'a>) -> bool),
    Obj( &'a mut dyn FnMut( &mut String, &mut XField< 'a>) -> bool),
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
    fn	ToXFlux( &self, flux: &mut dyn IXFlux);
}

//---------------------------------------------------------------------------------------------------------------------------------
 

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U8
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux)
    {
        flux.Field( XField::U64( self.0 as u64));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U16
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux)  
    {
        flux.Field( XField::U64( self.0 as u64));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U32
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux) 
    {
        flux.Field( XField::U64( self.0 as u64));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for crate::silo::U64
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux)
    {
        flux.Field( XField::U64( self.0 as u64));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for f32
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux)
    {
        flux.Field( XField::F64( *self as f64));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for f64
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux)
    {
        flux.Field( XField::F64( *self));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IXFluxable for crate::silo::Arr<'a, T>
where
    T: IXFluxable,
{
    fn	ToXFlux( &self, flux: &mut dyn IXFlux)
    {
        let  	mut idx = 0u32;
        let  	arr = *self;
        flux.Field( XField::Arr( &mut |item| {
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
