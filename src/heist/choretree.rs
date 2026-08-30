//-- choretree.rs ---------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    heist::{ IAtelier, IMaestro },
    silo::{ Stash, U16 },
    stalks::{ BinNode, BinOp, DynIWorker, IntoWorkPtr, IWork },
    swarm::BackendKind,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Execution target affinity for a chore.
#[derive( Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChoreTarget
{
    Cpu,
    Gpu( BackendKind),
    GpuAuto,
}

impl Default for ChoreTarget
{
    fn	default() -> Self
    {
        ChoreTarget::Cpu
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug)]
pub struct Chore
{
    pub _DocStr: &'static str,
    pub _Target: ChoreTarget,
    _Closure:    fn( &DynIWorker< '_>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Chore
{
    fn	default() -> Self
    {
        Self {
            _DocStr:  "",
            _Target:  ChoreTarget::Cpu,
            _Closure: |_| {},
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  "",
            _Target:  ChoreTarget::Cpu,
            _Closure: f,
        }
    }

    pub fn	NewDoc( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Cpu,
            _Closure: f,
        }
    }

    pub fn	Cpu( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Cpu,
            _Closure: f,
        }
    }

    pub fn	Gpu( docStr: &'static str, backend: BackendKind, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Gpu( backend),
            _Closure: f,
        }
    }

    pub fn	GpuAuto( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::GpuAuto,
            _Closure: f,
        }
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IChore: IWork + IChoreNode
{
    fn	Target( &self) -> ChoreTarget;
    fn	DocStr( &self) -> &'static str;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IChore for Chore
{
    fn	Target( &self) -> ChoreTarget
    {
        self._Target
    }

    fn	DocStr( &self) -> &'static str
    {
        self._DocStr
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
        let  	targetStr = match self._Target {
            ChoreTarget::Cpu => "CPU",
            ChoreTarget::Gpu( b) => match b {
                BackendKind::Cpu => "CPU",
                BackendKind::RustGpu => "Rust-GPU",
                BackendKind::CudaOxide => "CUDA",
            },
            ChoreTarget::GpuAuto => "GPU-Auto",
        };

        if self._DocStr.is_empty() {
            write!( f, "Chore[{}]", targetStr)
        } else {
            write!( f, "Chore[{}]: {}", targetStr, self._DocStr)
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
macro_rules! CpuChore {
    ( $closure:expr) => {
        $crate::heist::Chore::Cpu( "", $closure)
    };
    ( $doc:expr, $closure:expr) => {
        $crate::heist::Chore::Cpu( $doc, $closure)
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! GpuChore {
    ( $doc:expr, $backend:expr, $closure:expr) => {
        $crate::heist::Chore::Gpu( $doc, $backend, $closure)
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! GpuAutoChore {
    ( $closure:expr) => {
        $crate::heist::Chore::GpuAuto( "", $closure)
    };
    ( $doc:expr, $closure:expr) => {
        $crate::heist::Chore::GpuAuto( $doc, $closure)
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

pub trait IChoreNode
{
    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IChoreNode> IChoreNode for &T
{
    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    {
        return ( **self).Post( maestro, tails);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IChoreNode for Chore
{
    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    {
        let  	jobId = maestro.ConstructJob( U16::_0, IntoWorkPtr::IntoWorkPtr( *self), self._DocStr);
        tails.Push( jobId);
        return jobId;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IChoreNode for BinNode< L, R>
where
    L: IChoreNode,
    R: IChoreNode,
{
    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    {
        match self._Op {
            BinOp::Bor => {
                let  	mut leftTails = Stash::New();
                let  	mut rightTails = Stash::New();
                let  	headL = self._Left.Post( maestro, &mut leftTails);
                let  	headR = self._Right.Post( maestro, &mut rightTails);
                while let  	Some( t) = leftTails.Pop() {
                    tails.Push( t);
                }
                while let  	Some( t) = rightTails.Pop() {
                    tails.Push( t);
                }
                let  	mut heads = Stash::New();
                heads.Push( headL);
                heads.Push( headR);
                let  	enqId = maestro.ConstructEnqueArr( U16( 0), heads.IntoBuff(), "EnqPar");
                return enqId;
            }
            BinOp::Less => {
                let  	mut leftTails = Stash::New();
                let  	headL = self._Left.Post( maestro, &mut leftTails);
                let  	headR = self._Right.Post( maestro, tails);
                while let  	Some( leftTail) = leftTails.Pop() {
                    maestro.Atelier().SetSucc( leftTail, headR);
                }
                return headL;
            }
            _ => panic!( "Unsupported operator in ChoreTree Post"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
