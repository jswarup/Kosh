//-- varexpr.rs ---------------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IXFluxable, xflux::XField };

use	core::any::Any;
use	crate::silo::U32;
use	crate::fresco::exprrepos::BaseExpr;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq)]
pub enum VarKind
{
    Scalar = 0,
    Prime = 1,
    Control = 2,
    Bridge = 3,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct VarAttrib
{
    pub _Name: String,
    _DepTok: U32,
    _AggrIndex: U32,
    _VarFlags: U32,
}

impl Default for VarAttrib
{
    fn	default() -> Self
    {
        Self {
            _Name: String::new(),
            _DepTok: U32::_X,
            _AggrIndex: U32::_X,
            _VarFlags: U32( 0),
        }
    }
}

impl VarAttrib
{
    pub fn	IsAggregate( &self) -> bool
    {
        self._AggrIndex != U32::_X
    }

    pub fn	IsIndependent( &self) -> bool
    {
        !self.IsAggregate() && self._DepTok == U32::_X
    }

    pub fn	IsDependent( &self) -> bool
    {
        !self.IsAggregate() && self._DepTok != U32::_X
    }

    pub fn	HasBits( &self, bit: VarKind) -> bool
    {
        ( self._VarFlags.Get() & ( 1 << ( bit as u32))) != 0
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct VarExpr
{
    _VarIndex: U32,
}

impl VarExpr
{
    pub fn	New( varIndex: U32) -> Self
    {
        Self { _VarIndex: varIndex }
    }

    pub fn	VarIndex( &self) -> U32
    {
        self._VarIndex
    }
}

impl BaseExpr for VarExpr
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

impl IXFluxable for VarAttrib
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	attr = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Name".to_string();
                *item = XField::Str( &attr._Name);
                step += 1;
                true
            } else if step == 1 {
                *key = "DepTok".to_string();
                *item = XField::U64( attr._DepTok.0 as u64);
                step += 1;
                true
            } else if step == 2 {
                *key = "AggrIndex".to_string();
                *item = XField::U64( attr._AggrIndex.0 as u64);
                step += 1;
                true
            } else if step == 3 {
                *key = "VarFlags".to_string();
                *item = XField::U64( attr._VarFlags.0 as u64);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for VarExpr
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	expr = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Type".to_string();
                *item = XField::Str( "VarExpr");
                step += 1;
                true
            } else if step == 1 {
                *key = "VarIndex".to_string();
                *item = XField::U64( expr._VarIndex.0 as u64);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
