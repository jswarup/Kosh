//-- powexpr.rs ----------------------------------------------------------------------------------------------------------------------
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

crate::ImplFluxSourceTyped!( PowExpr, "PowExpr", _Poly);
