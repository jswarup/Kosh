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
 
#[macro_export]
macro_rules! ImplIXFluxable 
{
    ( $struct_name:ident $( , $field:ident )* ) => 
    {
        impl $crate::segue::IXFluxable for $struct_name
        {
            fn	ToXFlux< 'a>( &'a self, field: &mut $crate::segue::xflux::XField< 'a>)
            {
                let  	mut step = U32( 0);
                let  	obj = self;
                *field = $crate::segue::xflux::XField::Obj( Box::new( move |key, item| {
                    #[allow( unused_variables, unused_assignments)]
                    let  	mut _curr_step = U32( 0);
                    $(
                        if step == _curr_step {
                            *key = stringify!( $field).to_string();
                            *item = $crate::segue::xflux::XField::Fluxable( &obj.$field);
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
