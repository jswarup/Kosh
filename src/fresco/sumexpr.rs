//-- sumexpr.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::BaseExpr;
use	crate::fresco::polyexpr::PolyExpr;
use	core::any::Any;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct SumExpr
{
    pub _Poly: PolyExpr,
}

impl SumExpr
{
    pub fn	New() -> Self
    {
        Self { _Poly: PolyExpr::New() }
    }
}

impl BaseExpr for SumExpr
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
