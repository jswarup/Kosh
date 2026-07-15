//-- varexpr.rs ---------------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };

use	core::any::Any;
use	crate::silo::{ U32, U64};
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

impl IFluxExportSource for VarAttrib
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut step = 0u32;
        let  	attr = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Name".to_string();
                *item = FieldExp::Str( &attr._Name);
                step += 1;
                true
            } else if step == 1 {
                *key = "DepTok".to_string();
                *item = FieldExp::U64( U64::From( attr._DepTok.0 as u64));
                step += 1;
                true
            } else if step == 2 {
                *key = "AggrIndex".to_string();
                *item = FieldExp::U64( U64::From( attr._AggrIndex.0 as u64));
                step += 1;
                true
            } else if step == 3 {
                *key = "VarFlags".to_string();
                *item = FieldExp::U64( U64::From( attr._VarFlags.0 as u64));
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for VarExpr
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut step = 0u32;
        let  	expr = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Type".to_string();
                *item = FieldExp::Str( "VarExpr");
                step += 1;
                true
            } else if step == 1 {
                *key = "VarIndex".to_string();
                *item = FieldExp::U64( U64::From( expr._VarIndex.0 as u64));
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxImportSource!( VarAttrib, _Name, _DepTok, _AggrIndex, _VarFlags);
crate::ImplFluxImportSourceTyped!( VarExpr, "VarExpr", _VarIndex);
