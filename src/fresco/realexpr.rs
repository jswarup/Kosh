//-- realexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IXFluxable, xflux::XField };
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

impl IXFluxable for RealExpr
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	expr = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Type".to_string();
                *item = XField::Str( "RealExpr");
                step += 1;
                true
            } else if step == 1 {
                *key = "Value".to_string();
                *item = XField::F64( expr._Value);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
