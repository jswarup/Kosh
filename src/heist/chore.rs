//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ IXFluxSource, xflux::XField }, stalks::IntoWorkPtr };
use	std::fmt;
use	crate::stalks::{ DynIWorker, IWork };

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
    // 1. Grouping with remainder
    ( ( $( $inner:tt )+ ) < $( $rest:tt )+ ) => {
        &$crate::stalks::node::CatNode {
            _Left: $crate::ChoreTree!( ( $( $inner )+ ) ),
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };
    ( ( $( $inner:tt )+ ) | $( $rest:tt )+ ) => {
        &$crate::stalks::node::ParNode {
            _Left: $crate::ChoreTree!( ( $( $inner )+ ) ),
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };

    // 2. Closure with remainder
    ( | $arg:ident | $body:block < $( $rest:tt )+ ) => {
        &$crate::stalks::node::CatNode {
            _Left: &$crate::heist::Chore::New( |$arg| $body ),
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };
    ( | $arg:ident | $body:block | $( $rest:tt )+ ) => {
        &$crate::stalks::node::ParNode {
            _Left: &$crate::heist::Chore::New( |$arg| $body ),
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };
    ( move | $arg:ident | $body:block < $( $rest:tt )+ ) => {
        &$crate::stalks::node::CatNode {
            _Left: &$crate::heist::Chore::New( move |$arg| $body ),
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };
    ( move | $arg:ident | $body:block | $( $rest:tt )+ ) => {
        &$crate::stalks::node::ParNode {
            _Left: &$crate::heist::Chore::New( move |$arg| $body ),
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };

    // 3. Ident with remainder
    ( $l:ident < $( $rest:tt )+ ) => {
        &$crate::stalks::node::CatNode {
            _Left: &$l,
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };
    ( $l:ident | $( $rest:tt )+ ) => {
        &$crate::stalks::node::ParNode {
            _Left: &$l,
            _Right: $crate::ChoreTree!( $( $rest )+ ),
        }
    };

    // Base Case: Group
    ( ( $( $inner:tt )+ ) ) => {
        $crate::ChoreTree!( $( $inner )+ )
    };

    // Base Case: Closure
    ( | $arg:ident | $body:block ) => {
        &$crate::heist::Chore::New( |$arg| $body )
    };
    ( move | $arg:ident | $body:block ) => {
        &$crate::heist::Chore::New( move |$arg| $body )
    };

    // Base Case: Expr
    ( $leaf:expr ) => {
        &$leaf
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IChoreNode
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut crate::silo::Buff< u16>) -> u16;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'r, T: IChoreNode + ?Sized> IChoreNode for &'r T
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut crate::silo::Buff< u16>) -> u16
    {
        return (**self).Post( maestro, tails);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IChoreNode for Chore
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut crate::silo::Buff< u16>) -> u16
    {
        let  	jobId = maestro.ConstructJob( crate::silo::U16( 0), IntoWorkPtr::IntoWorkPtr( *self), self._DocStr);
        tails.Push( jobId.0);
        return jobId.0;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IChoreNode for crate::stalks::node::ParNode< L, R>
where
    L: IChoreNode,
    R: IChoreNode,
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut crate::silo::Buff< u16>) -> u16
    {
        let  	mut leftTails = crate::silo::Buff::NewEmpty();
        let  	mut rightTails = crate::silo::Buff::NewEmpty();
        let  	headL = self._Left.Post( maestro, &mut leftTails);
        let  	headR = self._Right.Post( maestro, &mut rightTails);
        while let  	Some( t) = leftTails.Pop() {
            tails.Push( t);
        }
        while let  	Some( t) = rightTails.Pop() {
            tails.Push( t);
        }
        let  	mut heads = crate::silo::Buff::NewEmpty();
        heads.Push( crate::silo::U16( headL));
        heads.Push( crate::silo::U16( headR));
        let  	enqId = maestro.ConstructEnqueArr( crate::silo::U16( 0), heads, "EnqPar");
        return enqId.0;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IChoreNode for crate::stalks::node::CatNode< L, R>
where
    L: IChoreNode,
    R: IChoreNode,
{
    fn	Post( &self, maestro: &crate::heist::Maestro, tails: &mut crate::silo::Buff< u16>) -> u16
    {
        let  	mut leftTails = crate::silo::Buff::NewEmpty();
        let  	headL = self._Left.Post( maestro, &mut leftTails);
        let  	headR = self._Right.Post( maestro, tails);
        while let  	Some( leftTail) = leftTails.Pop() {
            maestro.Atelier().SetSucc( crate::silo::U16( leftTail), crate::silo::U16( headR));
        }
        return headL;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
