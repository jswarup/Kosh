//-- choretree.rs ---------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    heist::{ IAtelier, IMaestro },
    silo::{ Arr, IAccess, IArr, Stash, U16, U32, USeg },
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
    pub _DocStr:  &'static str,
    pub _Target:  ChoreTarget,
    pub _Weight:  U32,
    _Closure:     fn( &DynIWorker< '_>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Chore
{
    fn	default() -> Self
    {
        Self {
            _DocStr:  "",
            _Target:  ChoreTarget::Cpu,
            _Weight:  U32( 1),
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
            _Weight:  U32( 1),
            _Closure: f,
        }
    }

    pub fn	NewDoc( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Cpu,
            _Weight:  U32( 1),
            _Closure: f,
        }
    }

    pub fn	Cpu( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Cpu,
            _Weight:  U32( 1),
            _Closure: f,
        }
    }

    pub fn	Gpu( docStr: &'static str, backend: BackendKind, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::Gpu( backend),
            _Weight:  U32( 1),
            _Closure: f,
        }
    }

    pub fn	GpuAuto( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self {
            _DocStr:  docStr,
            _Target:  ChoreTarget::GpuAuto,
            _Weight:  U32( 1),
            _Closure: f,
        }
    }

    pub fn	WithWeight< W: Into< U32>>( mut self, weight: W) -> Self
    {
        self._Weight = weight.into();
        self
    }

    pub fn	Weight( &self) -> U32
    {
        self._Weight
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

pub trait IChoreNode: Copy + Send + Sync
{
    fn	Weight( &self) -> U32 { U32( 1) }
    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    where
        Self: 'a;
    fn  Exec( &self, worker: &DynIWorker< '_>);
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IChoreNode> IChoreNode for &T
{
    fn	Weight( &self) -> U32
    {
        ( **self).Weight()
    }

    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    where
        Self: 'a,
    {
        return ( **self).Post( maestro, tails);
    }

    fn	Exec( &self, worker: &DynIWorker< '_>)
    {
        ( **self).Exec( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IChoreNode for Chore
{
    fn	Weight( &self) -> U32
    {
        self._Weight
    }

    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    where
        Self: 'a,
    {
        let  	jobId = maestro.ConstructJob( U16::_0, IntoWorkPtr::IntoWorkPtr( *self), self._DocStr);
        tails.Push( jobId);
        return jobId;
    }

    fn	Exec( &self, worker: &DynIWorker< '_>)
    {
        ( self._Closure)( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone)]
pub struct FusedChore< T>
{
    pub _Node: T,
}

impl< T: IChoreNode> IWork for FusedChore< T>
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        self._Node.Exec( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IChoreNode for BinNode< L, R>
where
    L: IChoreNode,
    R: IChoreNode,
{
    fn	Weight( &self) -> U32
    {
        self._Left.Weight() + self._Right.Weight()
    }

    fn	Post< 'a, M: IMaestro< 'a>>( &self, maestro: &M, tails: &mut Stash< U16>) -> U16
    where
        Self: 'a,
    {
        if self.Weight() <= maestro.Atelier().FusionThres() {
            let  	fused = FusedChore { _Node: *self };
            let  	jobId = maestro.ConstructJob( U16( 0), fused, "FusedBinNode");
            tails.Push( jobId);
            return jobId;
        }

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

    fn	Exec( &self, worker: &DynIWorker< '_>)
    {
        match self._Op {
            BinOp::Bor | BinOp::Less => {
                self._Left.Exec( worker);
                self._Right.Exec( worker);
            }
            _ => panic!( "Unsupported operator in ChoreTree Exec"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone)]
pub struct SpawnQuellNode< 'a, T>
{
    pub _Data:       Arr< 'a, T>,
    pub _Target:     ChoreTarget,
    pub _DocStr:     &'static str,
    pub _ItemWeight: U32,
    pub _SpawnFn:    fn( Arr< 'a, T>, &DynIWorker< '_>),
    pub _QuellFn:    fn( Arr< 'a, T>, &DynIWorker< '_>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T> SpawnQuellNode< 'a, T>
{
    pub fn	New< W: Into< U32>>(
        data: Arr< 'a, T>,
        target: ChoreTarget,
        itemWeight: W,
        docStr: &'static str,
        spawnFn: fn( Arr< 'a, T>, &DynIWorker< '_>),
        quellFn: fn( Arr< 'a, T>, &DynIWorker< '_>),
    ) -> Self
    {
        Self {
            _Data:       data,
            _Target:     target,
            _DocStr:     docStr,
            _ItemWeight: itemWeight.into(),
            _SpawnFn:    spawnFn,
            _QuellFn:    quellFn,
        }
    }
}

#[derive( Copy, Clone)]
pub struct SpawnChunkWork< 'a, T>
{
    pub _Data:    Arr< 'a, T>,
    pub _SpawnFn: fn( Arr< 'a, T>, &DynIWorker< '_>),
}

impl< 'a, T: Copy + Send + Sync> IWork for SpawnChunkWork< 'a, T>
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        ( self._SpawnFn)( self._Data, worker);
    }
}

#[derive( Copy, Clone)]
pub struct QuellWork< 'a, T>
{
    pub _Data:    Arr< 'a, T>,
    pub _QuellFn: fn( Arr< 'a, T>, &DynIWorker< '_>),
}

impl< 'a, T: Copy + Send + Sync> IWork for QuellWork< 'a, T>
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        ( self._QuellFn)( self._Data, worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, T: Copy + Send + Sync> IChoreNode for SpawnQuellNode< 'a, T>
{
    fn	Weight( &self) -> U32
    {
        self._Data.Size() * self._ItemWeight
    }

    fn	Post< 'm, MA: IMaestro< 'm>>( &self, maestro: &MA, tails: &mut Stash< U16>) -> U16
    where
        Self: 'm,
    {
        let  	totalWeight = self.Weight();
        let  	isCpu = matches!( self._Target, ChoreTarget::Cpu);
        let  	fusionThres = maestro.Atelier().FusionThres();

        let  	quellWork = QuellWork {
            _Data:    self._Data,
            _QuellFn: self._QuellFn,
        };
        let  	quellJobId = maestro.ConstructJob( U16( 0), quellWork, "Quell");
        tails.Push( quellJobId);

        if !isCpu || totalWeight <= fusionThres {
            let  	spawnWork = SpawnChunkWork {
                _Data:    self._Data,
                _SpawnFn: self._SpawnFn,
            };
            let  	spawnJobId = maestro.ConstructJob( quellJobId, spawnWork, self._DocStr);
            return spawnJobId;
        }

        let  	numMaestros = maestro.Atelier().Maestros().Size();
        let  	mut c = numMaestros * U32( 2);
        let  	maxChunks = totalWeight / fusionThres;
        if c > maxChunks {
            c = maxChunks;
        }
        if c == U32( 0) {
            c = U32( 1);
        }

        let  	totalSz = self._Data.Size();
        let  	mut heads = Stash::New();

        let  	chunkSize = ( totalSz + c - U32( 1)) / c;
        let  	mut start = U32( 0);
        let  	end = totalSz;

        while start < end {
            let  	rem = end - start;
            let  	sz = if rem < chunkSize { rem } else { chunkSize };
            let  	chunkArr = self._Data.Subset( start, sz);

            let  	spawnWork = SpawnChunkWork {
                _Data:    chunkArr,
                _SpawnFn: self._SpawnFn,
            };
            let  	spawnJobId = maestro.ConstructJob( quellJobId, spawnWork, self._DocStr);
            heads.Push( spawnJobId);

            start += sz;
        }

        let  	enqId = maestro.ConstructEnqueArr( U16( 0), heads.IntoBuff(), "EnqPar");
        enqId
    }

    fn	Exec( &self, worker: &DynIWorker< '_>)
    {
        ( self._SpawnFn)( self._Data, worker);
        ( self._QuellFn)( self._Data, worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! WeightedChore {
    ( $weight:expr, $doc:expr, $closure:expr) => {
        $crate::heist::Chore::NewDoc( $doc, $closure).WithWeight( $weight)
    };
}

#[macro_export]
macro_rules! SpawnQuell {
    ( $data:expr, $target:expr, $spawnFn:expr, $quellFn:expr) => {
        $crate::heist::SpawnQuellNode::New( $data, $target, $crate::silo::U32( 1), "SpawnQuell", $spawnFn, $quellFn)
    };
}

#[macro_export]
macro_rules! WeightedSpawnQuell {
    ( $data:expr, $itemWeight:expr, $target:expr, $spawnFn:expr, $quellFn:expr) => {
        $crate::heist::SpawnQuellNode::New( $data, $target, $itemWeight, "WeightedSpawnQuell", $spawnFn, $quellFn)
    };
}

#[macro_export]
macro_rules! CpuSpawnQuell {
    ( $data:expr, $spawnFn:expr, $quellFn:expr) => {
        $crate::heist::SpawnQuellNode::New( $data, $crate::heist::ChoreTarget::Cpu, $crate::silo::U32( 1), "CpuSpawnQuell", $spawnFn, $quellFn)
    };
}

#[macro_export]
macro_rules! GpuSpawnQuell {
    ( $data:expr, $spawnFn:expr, $quellFn:expr) => {
        $crate::heist::SpawnQuellNode::New( $data, $crate::heist::ChoreTarget::GpuAuto, $crate::silo::U32( 1), "GpuSpawnQuell", $spawnFn, $quellFn)
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
