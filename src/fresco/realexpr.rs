//-- realexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::BaseExpr;
use	core::any::Any;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct RealExpr
{
    _Value: f64,
}

impl RealExpr
{
    pub fn	New( value: f64) -> Self
    {
        Self { _Value: value }
    }

    pub fn	Value( &self) -> f64
    {
        self._Value
    }

    pub fn	SetValue( &mut self, value: f64)
    {
        self._Value = value;
    }
}

impl BaseExpr for RealExpr
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

crate::ImplFluxSourceTyped!( RealExpr, "RealExpr", _Value);
