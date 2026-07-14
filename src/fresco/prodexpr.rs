//-- prodexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IFluxOutSource, fluxout::FieldOut };
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

impl IFluxOutSource for ProdExpr
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>)
    {
        let  	mut step = 0u32;
        let  	expr = self;
        *field = FieldOut::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Type".to_string();
                *item = FieldOut::Str( "ProdExpr");
                step += 1;
                true
            } else if step == 1 {
                *key = "Poly".to_string();
                expr._Poly.ToFieldOut( item);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
