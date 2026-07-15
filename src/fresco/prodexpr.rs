//-- prodexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use	crate::flux::{ IFluxImportSource, fluximport::FieldImp };
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

impl IFluxExportSource for ProdExpr
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut step = 0u32;
        let  	expr = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Type".to_string();
                *item = FieldExp::Str( "ProdExpr");
                step += 1;
                true
            } else if step == 1 {
                *key = "Poly".to_string();
                expr._Poly.FetchFieldExp( item);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxImportSourceTyped!( ProdExpr, "ProdExpr", _Poly);
