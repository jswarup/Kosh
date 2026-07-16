//-- prodexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::BaseExpr;
use	crate::fresco::polyexpr::PolyExpr;
use	core::any::Any;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct ProdExpr
{
    pub _Poly: PolyExpr,
}

impl ProdExpr
{
    pub fn	New() -> Self
    {
        Self { _Poly: PolyExpr::New() }
    }
}

impl BaseExpr for ProdExpr
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

crate::ImplFluxSourceTyped!( ProdExpr, "ProdExpr", _Poly);
