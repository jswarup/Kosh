//-- polyexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::BaseExpr;
use	crate::silo::U32;
use	core::any::Any;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct PolyExpr
{
    _Childs: Vec< U32>,
    _CoSz: U32,
}

impl PolyExpr
{
    pub fn	New() -> Self
    {
        Self {
            _Childs: Vec::new(),
            _CoSz: U32( 0),
        }
    }

    pub fn	DoInitSize( &mut self, coSz: U32, sz: usize)
    {
        self._CoSz = coSz;
        self._Childs.resize( sz, U32( 0));
    }

    pub fn	DoInitArr( &mut self, coSz: U32, arr: Vec< U32>)
    {
        self._CoSz = coSz;
        self._Childs = arr;
    }

    pub fn	SzChild( &self) -> U32
    {
        U32( self._Childs.len() as u32)
    }

    pub fn	Child( &self, k: usize) -> U32
    {
        self._Childs[ k]
    }

    pub fn	SetChild( &mut self, k: usize, childToken: U32)
    {
        self._Childs[ k] = childToken;
    }

    pub fn	IsFlip( &self, k: U32) -> bool
    {
        k >= self._CoSz
    }
}

impl BaseExpr for PolyExpr
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

impl crate::segue::IXFluxable for PolyExpr
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::segue::xflux::XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	poly = self;
        *field = crate::segue::xflux::XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "CoSz".to_string();
                *item = crate::segue::xflux::XField::U64( poly._CoSz.0 as u64);
                step += 1;
                true
            } else if step == 1 {
                *key = "Childs".to_string();
                let  	mut iterStep = 0usize;
                *item = crate::segue::xflux::XField::Arr( Box::new( move |elem| {
                    if iterStep < poly._Childs.len() {
                        *elem = crate::segue::xflux::XField::U64( poly._Childs[iterStep].0 as u64);
                        iterStep += 1;
                        true
                    } else {
                        false
                    }
                }));
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

