//-- powexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::fresco::exprrepos::BaseExpr;
use	crate::fresco::polyexpr::PolyExpr;
use	core::any::Any;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct PowExpr
{
    pub _Poly: PolyExpr,
}

impl PowExpr
{
    pub fn	New() -> Self
    {
        Self { _Poly: PolyExpr::New() }
    }
}

impl BaseExpr for PowExpr
{
    fn	CloneBox( &self) -> Box< dyn BaseExpr>
    {
        Box::new( self.clone())
    }

    fn	AsAny( &self) -> &dyn Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for PowExpr
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	expr = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Type".to_string();
                *item = XField::Str( "PowExpr");
                step += 1;
                true
            } else if step == 1 {
                *key = "Poly".to_string();
                expr._Poly.ToXField( item);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
