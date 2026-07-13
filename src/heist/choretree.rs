//-- choretree.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::{
    flux::{ IXFluxSource, xflux::XField },
    stalks::{ IntoWorkPtr, BinNode, DynIWorker, IWork, INode, BinOp },
    silo::{ U16, Buff},
};
use	std::fmt;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug)]
pub struct Chore
{
    pub _DocStr: &'static str,
    _Closure: fn( &DynIWorker< '_>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Chore
{
    fn	default() -> Self
    {
        Self { _DocStr: "", _Closure: |_| {} }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for Chore
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "DocStr".to_string();
                *item = XField::Str( self._DocStr);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self { _DocStr: "", _Closure: f }
    }

    pub fn	NewDoc( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self { _DocStr: docStr, _Closure: f }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Chore
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        ( self._Closure)( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for Chore
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._DocStr.is_empty() {
            write!( f, "Chore")
        } else {
            write!( f, "Chore: {}", self._DocStr)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! Chore {
    ( $closure:expr) => {
        $crate::heist::Chore::New( $closure)
    };
    ( $doc:expr, $closure:expr) => {
        $crate::heist::Chore::NewDoc( $doc, $closure)
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ChoreTree {
    // 1. Pre-parse closures:
    ( | $arg:ident | $body:block ) => {
        $crate::heist::Chore::New( |$arg| $body )
    };
    ( move | $arg:ident | $body:block ) => {
        $crate::heist::Chore::New( move |$arg| $body )
    };
    ( | $arg:ident | $body:block < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::heist::Chore::New( |$arg| $body ), ChoreTree, $( $rest )+ )
    };
    ( | $arg:ident | $body:block | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor, $crate::heist::Chore::New( |$arg| $body ), ChoreTree, $( $rest )+ )
    };
    ( move | $arg:ident | $body:block < $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Less, $crate::heist::Chore::New( move |$arg| $body ), ChoreTree, $( $rest )+ )
    };
    ( move | $arg:ident | $body:block | $( $rest:tt )+ ) => {
        $crate::NodeTree!( @bin Bor, $crate::heist::Chore::New( move |$arg| $body ), ChoreTree, $( $rest )+ )
    };

    // 2. leaf rule
    ( @leaf $( $leaf:tt )+ ) => {
        $( $leaf )+
    };

    // 3. Delegate to NodeTree
    ( $( $tt:tt )+ ) => {
        $crate::NodeTree!( @parse ChoreTree, $( $tt )+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IChoreNode: INode
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut Buff< U16>) -> U16;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IChoreNode + ?Sized> IChoreNode for &T
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut Buff< U16>) -> U16
    {
        ( **self).Post( maestro, tails)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IChoreNode for Chore
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut Buff< U16>) -> U16
    {
        let  	jobId = maestro.ConstructJob( U16::_0, IntoWorkPtr::IntoWorkPtr( *self), self._DocStr);
        tails.Push( jobId);
        jobId
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
impl< L, R> IChoreNode for BinNode< L, R>
where
    L: IChoreNode,
    R: IChoreNode,
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut Buff< U16>) -> U16
    {
        match self._Op {
            BinOp::Bor => {
                let  	mut leftTails = Buff::NewEmpty();
                let  	mut rightTails = Buff::NewEmpty();
                let  	headL = self._Left.Post( maestro, &mut leftTails);
                let  	headR = self._Right.Post( maestro, &mut rightTails);
                while let  	Some( t) = leftTails.Pop() {
                    tails.Push( t);
                }
                while let  	Some( t) = rightTails.Pop() {
                    tails.Push( t);
                }
                let  	mut heads = Buff::NewEmpty();
                heads.Push( headL);
                heads.Push( headR);
                let  	enqId = maestro.ConstructEnqueArr( U16( 0), heads, "EnqPar");
                enqId
            }
            BinOp::Less => {
                let  	mut leftTails = Buff::NewEmpty();
                let  	headL = self._Left.Post( maestro, &mut leftTails);
                let  	headR = self._Right.Post( maestro, tails);
                while let  	Some( leftTail) = leftTails.Pop() {
                    maestro.Atelier().SetSucc( leftTail, headR);
                }
                headL
            }
            _ => panic!( "Unsupported operator in ChoreTree Post"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
