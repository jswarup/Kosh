//-- exprrepos.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ IAccess, Stash, U32 };
use	crate::fresco::varexpr::{ VarAttrib, VarExpr };

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

