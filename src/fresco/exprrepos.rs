//-- exprrepos.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ IAccess, IArr, Stash, U32 };
use	crate::fresco::varexpr::{ VarAttrib, VarExpr };
use	crate::fresco::realexpr::RealExpr;
use	crate::fresco::sumexpr::SumExpr;
use	crate::fresco::prodexpr::ProdExpr;

//---------------------------------------------------------------------------------------------------------------------------------

use	core::any::Any;

pub trait BaseExpr: Any + crate::segue::IXFluxable
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

impl crate::segue::IXFluxable for ExprEntry
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::segue::xflux::XField< 'b>)
    {
        match self {
            ExprEntry::Empty => *field = crate::segue::xflux::XField::Null,
            ExprEntry::Expr( expr) => expr.ToXFlux( field),
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

    pub fn	PowCreate( &mut self, bases: &[ U32], exps: &[ U32]) -> U32
    {
        let  	mut childs = Vec::with_capacity( bases.len() + exps.len());
        childs.extend_from_slice( bases);
        childs.extend_from_slice( exps);

        let  	mut powExpr = crate::fresco::powexpr::PowExpr::New();
        powExpr._Poly.DoInitArr( U32( bases.len() as u32), childs);
        self.Store( Box::new( powExpr))
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

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PostTermTree( &mut self, node: &crate::stalks::DynINode< '_>) -> U32
    {
        let  	exprStash = Stash::<U32>::New( U32( 1024), 0, U32( 0));
        let  	mut exprStk = exprStash.Stk();
        let  	opStash = Stash::<(crate::stalks::ChildOp, U32)>::New( U32( 1024), 0, (crate::stalks::ChildOp::None, U32( 0)));
        let  	opStk = opStash.Stk();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	curNode = probe.CurNode().unwrap();
            let  	curOp = curNode.ChildOp();
            if enterFlg {
                if curOp != crate::stalks::ChildOp::None {
                    opStk.Push( ( curOp, exprStk.Size()));
                    return;
                } 
                let  	term = curNode.AsAny().unwrap().downcast_ref::<crate::fresco::Term>().unwrap();
                let  	exprId = match term {
                    crate::fresco::Term::String( s) => self.VarCreate( s.clone(), false),
                    crate::fresco::Term::Real( v) => self.RealCreate( *v),
                };
                exprStk.Push( exprId);
                return;
            }
            if curOp == crate::stalks::ChildOp::None { 
                return; 
            }
            let  	mut opCtx = ( crate::stalks::ChildOp::None, U32( 0));
            opStk.Pop( &mut opCtx); 

            let  	parentOp = if opStk.Size() != 0 { opStk.Arr().Last().0 } else { crate::stalks::ChildOp::None };
            if parentOp == curOp { 
                return; 
            }

            let  	arr = exprStk.Arr().Subset( opCtx.1, exprStk.Size() - opCtx.1);
            exprStk.SetSize( opCtx.1);
            let  	exprId = match curOp {
                crate::stalks::ChildOp::Sum => self.SumCreate( &arr, &[]),
                crate::stalks::ChildOp::Prod => self.ProdCreate( &arr, &[]),
                crate::stalks::ChildOp::Sub => self.SumCreate( &arr[0..1], &arr[1..]),
                crate::stalks::ChildOp::Div => self.ProdCreate( &arr[0..1], &arr[1..]),
                crate::stalks::ChildOp::Pow => self.PowCreate( &arr[0..1], &arr[1..]),
                _ => panic!( "Unsupported ChildOp in PostTermTree: {:?}", curOp),
            };
            exprStk.Push( exprId);
        }); 
        
        if exprStk.Size() == 0 {
            U32( 0)
        } else {
            *exprStk.Arr().Last()
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::segue::IXFluxable for ExprRepos
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::segue::xflux::XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	repos = self;
        *field = crate::segue::xflux::XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Exprs".to_string();
                let  	mut iterStep = 0u32;
                *item = crate::segue::xflux::XField::Arr( Box::new( move |elem| {
                    if iterStep < repos._Exprs.Size().0 {
                        let  	expr = repos._Exprs.Stk().Arr().At( crate::silo::U32( iterStep));
                        expr.ToXFlux( elem);
                        iterStep += 1;
                        true
                    } else {
                        false
                    }
                }));
                step += 1;
                true
            } else if step == 1 {
                *key = "VarAttribs".to_string();
                let  	mut iterStep = 0u32;
                *item = crate::segue::xflux::XField::Arr( Box::new( move |elem| {
                    if iterStep < repos._VarAttribs.Size().0 {
                        let  	attr = repos._VarAttribs.Stk().Arr().At( crate::silo::U32( iterStep));
                        attr.ToXFlux( elem);
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
