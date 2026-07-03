//-- exprrepos.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ IAccess, IArr, Stash, U32, Arr, Buff };
use	crate::flux::{ IXFluxable, xflux::XField };
use	crate::fresco::varexpr::{ VarAttrib, VarExpr };
use	crate::fresco::realexpr::RealExpr;
use	crate::fresco::sumexpr::SumExpr;
use	crate::fresco::prodexpr::ProdExpr;
use	crate::fresco::powexpr::PowExpr;
use	crate::fresco::Term;
use	crate::stalks::{ ChildOp, DynINode };

//---------------------------------------------------------------------------------------------------------------------------------

use	core::any::Any;

pub trait BaseExpr: Any + IXFluxable
{
    fn	SizeChild( &self, _chart: &ExprRepos) -> U32
    {
        0.into()
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

impl IXFluxable for ExprEntry
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        match self {
            ExprEntry::Empty => *field = XField::Null,
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

    pub fn	SumCreate( &mut self, adds: Arr<'_, U32>, subs: Arr<'_, U32>) -> U32
    {
        let  	childs = Buff::Concat( adds, subs);
        let  	mut sumExpr = SumExpr::New();
        sumExpr._Poly.DoInitArr( adds.Size(), childs);
        self.Store( Box::new( sumExpr))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.SumCreate( Arr::from( &[tok0, tok1]), Arr::from( &[] as &[U32]))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DiffCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.SumCreate( Arr::from( &[tok0]), Arr::from( &[tok1]))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ProdCreate( &mut self, numers: Arr<'_, U32>, denoms: Arr<'_, U32>) -> U32
    {
        let  	childs = Buff::Concat( numers, denoms);
        let  	mut prodExpr = ProdExpr::New();
        prodExpr._Poly.DoInitArr( numers.Size(), childs);
        self.Store( Box::new( prodExpr))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	MultCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.ProdCreate( Arr::from( &[tok0, tok1]), Arr::from( &[] as &[U32]))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DivCreate( &mut self, tok0: U32, tok1: U32) -> U32
    {
        self.ProdCreate( Arr::from( &[tok0]), Arr::from( &[tok1]))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PowCreate( &mut self, bases: Arr<'_, U32>, exps: Arr<'_, U32>) -> U32
    {
        let  	childs = Buff::Concat( bases, exps);
        let  	mut powExpr = PowExpr::New();
        powExpr._Poly.DoInitArr( bases.Size(), childs);
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

    pub fn	PostTermTree( &mut self, node: &DynINode< '_>) -> U32
    {
        let  	exprStash = Stash::<U32>::New( 1024, 0, 0.into());
        let  	mut exprStk = exprStash.Stk();
        let  	opStash = Stash::<(ChildOp, U32)>::New( 1024, 0, (ChildOp::None, 0.into()));
        let  	opStk = opStash.Stk();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	curNode = probe.CurNode().unwrap();
            let  	curOp = curNode.ChildOp();
            if enterFlg {
                if curOp != ChildOp::None {
                    opStk.Push( ( curOp, exprStk.Size()));
                    return;
                }
                let  	term = curNode.AsAny().unwrap().downcast_ref::<Term>().unwrap();
                let  	exprId = match term {
                    Term::String( s) => self.VarCreate( s.clone(), false),
                    Term::Real( v) => self.RealCreate( *v),
                };
                exprStk.Push( exprId);
                return;
            }
            if curOp == ChildOp::None {
                return;
            }
            let  	mut opCtx = ( ChildOp::None, 0.into());
            opStk.Pop( &mut opCtx);

            let  	parentOp = if opStk.Size() != 0 { opStk.Arr().Last().0 } else { ChildOp::None };
            if parentOp == curOp {
                return;
            }

            let  	arr = exprStk.Arr().Subset( opCtx.1, exprStk.Size() - opCtx.1);
            exprStk.SetSize( opCtx.1);
            let  	emptyArr = Arr::from( &[][..]);
            let  	exprId = match curOp {
                ChildOp::Sum => self.SumCreate( arr, emptyArr),
                ChildOp::Prod => self.ProdCreate( arr, emptyArr),
                ChildOp::Sub => self.SumCreate( arr.Subset( 0, 1), arr.Subset( 1, arr.Size() - 1)),
                ChildOp::Div => self.ProdCreate( arr.Subset( 0, 1), arr.Subset( 1, arr.Size() - 1)),
                ChildOp::Pow => self.PowCreate( arr.Subset( 0, 1), arr.Subset( 1, arr.Size() - 1)),
                _ => panic!( "Unsupported ChildOp in PostTermTree: {:?}", curOp),
            };
            exprStk.Push( exprId);
        });

        if exprStk.Size() == 0 {
            0.into()
        } else {
            *exprStk.Arr().Last()
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for ExprRepos
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	repos = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Exprs".to_string();
                let  	mut iterStep = 0u32;
                *item = XField::Arr( Box::new( move |elem| {
                    if iterStep < repos._Exprs.Size().0 {
                        let  	expr = repos._Exprs.Stk().Arr().At( iterStep);
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
                *item = XField::Arr( Box::new( move |elem| {
                    if iterStep < repos._VarAttribs.Size().0 {
                        let  	attr = repos._VarAttribs.Stk().Arr().At( iterStep);
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
