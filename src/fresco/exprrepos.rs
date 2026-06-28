//-- exprrepos.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ IAccess, Stash, U32 };
use	crate::fresco::varexpr::{ VarAttrib, VarExpr };
use	crate::fresco::realexpr::RealExpr;
use	crate::fresco::sumexpr::SumExpr;
use	crate::fresco::prodexpr::ProdExpr;

//---------------------------------------------------------------------------------------------------------------------------------

use	core::any::Any;

pub trait BaseExpr: Any
{
    fn	SizeChild( &self, _chart: &ExprRepos) -> U32
    {
        U32( 0)
    }

    fn	IsBinary( &self) -> bool
    {
        false
    }

    fn	CloneBox( &self) -> Box< dyn BaseExpr>;
    fn	AsAny( &self) -> &dyn Any;
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Default)]
pub enum ExprEntry
{
    #[default]
    Empty,
    Expr( Box< dyn BaseExpr>),
}

impl Clone for ExprEntry
{
    fn	clone( &self) -> Self
    {
        match self {
            ExprEntry::Empty => ExprEntry::Empty,
            ExprEntry::Expr( expr) => ExprEntry::Expr( expr.CloneBox()),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Clone for Box< dyn BaseExpr>
{
    fn	clone( &self) -> Self
    {
        self.CloneBox()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ExprRepos
{
    _Exprs: Stash< ExprEntry>,
    _VarAttribs: Stash< VarAttrib>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ExprRepos
{
    pub fn	NewEmpty() -> Self
    {
        Self {
            _Exprs: Stash::NewEmpty(),
            _VarAttribs: Stash::NewEmpty(),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> U32
    {
        self._Exprs.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Store( &mut self, expr: Box< dyn BaseExpr>) -> U32
    {
        let  	ind = self.Size();
        let  	mut entry = ExprEntry::Expr( expr);
        self._Exprs.PushX( &mut entry);
        ind
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	StoreVar( &mut self, varStr: String) -> U32
    {
        let  	id = self._VarAttribs.Size();
        let  	mut attr = VarAttrib::default();
        attr._Name = varStr;
        self._VarAttribs.PushX( &mut attr);
        id
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	VarCreate( &mut self, varStr: String, _reuseFlg: bool) -> U32
    {
        let  	varInd = self.StoreVar( varStr);
        self.Store( Box::new( VarExpr::New( varInd)))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RealCreate( &mut self, val: f64) -> U32
    {
        self.Store( Box::new( RealExpr::New( val)))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SumCreate( &mut self, adds: &[ U32], subs: &[ U32]) -> U32
    {
        let  	mut childs = Vec::with_capacity( adds.len() + subs.len());
        childs.extend_from_slice( adds);
        childs.extend_from_slice( subs);

        let  	mut sumExpr = SumExpr::New();
        sumExpr._Poly.DoInitArr( U32( adds.len() as u32), childs);
        self.Store( Box::new( sumExpr))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.SumCreate( &[ tok0, tok1], &[])
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DiffCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.SumCreate( &[ tok0], &[ tok1])
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ProdCreate( &mut self, numers: &[ U32], denoms: &[ U32]) -> U32
    {
        let  	mut childs = Vec::with_capacity( numers.len() + denoms.len());
        childs.extend_from_slice( numers);
        childs.extend_from_slice( denoms);

        let  	mut prodExpr = ProdExpr::New();
        prodExpr._Poly.DoInitArr( U32( numers.len() as u32), childs);
        self.Store( Box::new( prodExpr))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MultCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.ProdCreate( &[ tok0, tok1], &[])
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DivCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.ProdCreate( &[ tok0], &[ tok1])
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SzVar( &self) -> U32
    {
        self._VarAttribs.Size()
    }

    pub fn	At< T: BaseExpr>( &self, tag: U32) -> &T
    {
        match self._Exprs.Stk().Arr().At( tag) {
            ExprEntry::Expr( expr) => expr.AsAny().downcast_ref::<T>().unwrap(),
            ExprEntry::Empty => panic!( "Empty ExprEntry"),
        }
    }
    
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	VarNameAt( &self, vInd: U32) -> &str
    {
        self._VarAttribs.Stk().Arr().At( vInd)._Name.as_str()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	VarAttrAt( &self, vInd: U32) -> &VarAttrib
    {
        self._VarAttribs.Stk().Arr().At( vInd)
    }
}

