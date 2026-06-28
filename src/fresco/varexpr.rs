//-- varexpr.rs ---------------------------------------------------------------------------------------------------------------------------

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
