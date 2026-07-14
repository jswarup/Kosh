//-- polyexpr.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IFluxOutSource, fluxout::FieldOut };
use	crate::fresco::exprrepos::BaseExpr;
use	crate::silo::{ U32, U64, Buff };
use	core::any::Any;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct PolyExpr
{
    _Childs: Buff< U32>,
    _CoSz: U32,
}

impl PolyExpr
{
    pub fn	New() -> Self
    {
        Self {
            _Childs: Buff::NewEmpty(),
            _CoSz: U32( 0),
        }
    }

    pub fn	DoInitSize( &mut self, coSz: U32, sz: usize)
    {
        self._CoSz = coSz;
        self._Childs.Resize( U32( sz as u32), |_| U32( 0));
    }

    pub fn	DoInitArr( &mut self, coSz: U32, arr: Buff< U32>)
    {
        self._CoSz = coSz;
        self._Childs = arr;
    }

    pub fn	SzChild( &self) -> U32
    {
        self._Childs.Size()
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

impl IFluxOutSource for PolyExpr
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>)
    {
        let  	mut step = 0u32;
        let  	poly = self;
        *field = FieldOut::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "CoSz".to_string();
                *item = FieldOut::U64( U64::From( poly._CoSz.0 as u64));
                step += 1;
                true
            } else if step == 1 {
                *key = "Childs".to_string();
                let  	mut iterStep = 0usize;
                *item = FieldOut::Arr( Box::new( move |elem| {
                    if iterStep < poly._Childs.len() {
                        *elem = FieldOut::U64( U64::From( poly._Childs[iterStep].0 as u64));
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

