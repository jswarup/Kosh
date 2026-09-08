//-- engine.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::{
    atomic::{ AtomicPtr, Ordering },
    Arc,
};
use	crate::{
    rube::{
        coro_kernel::{ CoroCell, CoroPorts, CoroWarp, CORO_MAX_PORTS },
        layout::Layout,
        module::{ BehavioralWarp, CustomWarp, FastWarp },
        port::PortId,
        reg::Reg,
        registry::{ CustomKernelFn, KernelRegistry },
        trigger::{ ITriggerVal, ITriggerWad, TriggerId, TriggerWad },
    },
    silo::{ arr::IArr, Buff, IAccess, U8, U32, U64, USeg },
    stalks::{ CoroRes, DynIWorker, ICoro, Spinlock },
    heist::{ Atelier, IAtelier, IMaestro },
    CpuSpawnQuell, ChoreTree,
};

//---------------------------------------------------------------------------------------------------------------------------------

static CURRENT_SIM_ENGINE: AtomicPtr< SimEngine< U64>> = AtomicPtr::new( std::ptr::null_mut() );
static SIM_DRIVE_LOCK: Spinlock = Spinlock::New();

//---------------------------------------------------------------------------------------------------------------------------------

fn	fast_warp_spawn< 'a>( chunk: crate::silo::Arr< 'a, FastWarp>, _w: &DynIWorker< '_>)
{
    let  	enginePtr = CURRENT_SIM_ENGINE.load( Ordering::Acquire);
    let  	engine = unsafe { &mut *enginePtr };
    let  	readyWords = &engine._ReadyWords;
    let  	triggers = &mut engine._Triggers;
    chunk.USeg().Traverse( |i| {
        let  	warp = chunk.At( i);
        let  	op = warp._Op;
        let  	mask = warp._Mask;
        let  	count = warp._Count.AsUsize();
        let  	modStart = warp._ModStart.AsUsize();
        SimEngine::< U64>::ForEachReadyLane( readyWords, modStart, count, |l| {
            let  	in1Trig = warp._In1[l];
            let  	in2Trig = warp._In2[l];
            let  	outTrig = warp._Out[l];
            let  	in1 = triggers._CurrentVals[in1Trig];
            let  	in2 = triggers._CurrentVals[in2Trig];
            let  	f1 = triggers._Flags[in1Trig].0;
            let  	f2 = triggers._Flags[in2Trig].0;
            if ( ( f1 | f2 ) & crate::rube::trigger::CURR_MASK ) == 0 {
                let  	raw = op.EvalRaw( in1.0, in2.0, mask);
                triggers._FutureVals[outTrig] = U64( raw);
                triggers._Flags[outTrig] = U8( triggers._Flags[outTrig].0 & !crate::rube::trigger::FUTR_MASK );
            } else {
                let  	r1 = Reg {
                    _Val: in1,
                    _X:   ( f1 & crate::rube::trigger::CURR_X) != 0,
                    _I:   ( f1 & crate::rube::trigger::CURR_I) != 0,
                };
                let  	r2 = Reg {
                    _Val: in2,
                    _X:   ( f2 & crate::rube::trigger::CURR_X) != 0,
                    _I:   ( f2 & crate::rube::trigger::CURR_I) != 0,
                };
                let  	res = op.Eval( r1, r2, mask);
                triggers._FutureVals[outTrig] = res._Val;
                let  	mut f = triggers._Flags[outTrig].0 & !crate::rube::trigger::FUTR_MASK;
                if res.IsX() { f |= crate::rube::trigger::FUTR_X; }
                if res.IsI() { f |= crate::rube::trigger::FUTR_I; }
                triggers._Flags[outTrig] = U8( f);
            }
        });
    });
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	custom_warp_spawn< 'a>( chunk: crate::silo::Arr< 'a, CustomWarp>, _w: &DynIWorker< '_>)
{
    let  	enginePtr = CURRENT_SIM_ENGINE.load( Ordering::Acquire);
    let  	engine = unsafe { &mut *enginePtr };
    let  	readyWords = &engine._ReadyWords;
    let  	triggers = &mut engine._Triggers;
    let  	customCallbacks = &engine._CustomCallbacks;
    chunk.USeg().Traverse( |i| {
        let  	warp = chunk.At( i);
        let  	count = warp._Count.AsUsize();
        let  	modStart = warp._ModStart.AsUsize();
        let  	basePtr = engine._CustomWarps.Arr().Ptr();
        let  	warpPtr = warp as *const CustomWarp;
        let  	globalIdx = unsafe { warpPtr.offset_from( basePtr) as usize };
        let  	cb = &customCallbacks[globalIdx];

        SimEngine::< U64>::ForEachReadyLane( readyWords, modStart, count, |l| {
            let  	inTrigs = &warp._InTriggers[l];
            let  	outTrigs = &warp._OutTriggers[l];
            SimEngine::< U64>::EvalCustomInstance( cb, inTrigs, outTrigs, triggers);
        });
    });
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	behavioral_warp_spawn< 'a>( chunk: crate::silo::Arr< 'a, BehavioralWarp>, _w: &DynIWorker< '_>)
{
    let  	enginePtr = CURRENT_SIM_ENGINE.load( Ordering::Acquire);
    let  	engine = unsafe { &mut *enginePtr };
    let  	readyWords = &engine._ReadyWords;
    let  	triggers = &mut engine._Triggers;
    chunk.USeg().Traverse( |i| {
        let  	warp = chunk.At( i);
        let  	count = warp._Count.AsUsize();
        let  	modStart = warp._ModStart.AsUsize();
        SimEngine::< U64>::ForEachReadyLane( readyWords, modStart, count, |l| {
            let  	cb = &warp._Instances[l];
            let  	inTrigs = &warp._InTriggers[l];
            let  	outTrigs = &warp._OutTriggers[l];
            SimEngine::< U64>::EvalCustomInstance( cb, inTrigs, outTrigs, triggers);
        });
    });
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	trait_warp_spawn< 'a>( chunk: crate::silo::Arr< 'a, crate::rube::module::TraitWarp>, _w: &DynIWorker< '_>)
{
    let  	enginePtr = CURRENT_SIM_ENGINE.load( Ordering::Acquire);
    let  	engine = unsafe { &mut *enginePtr };
    let  	readyWords = &engine._ReadyWords;
    let  	triggers = &mut engine._Triggers;
    chunk.USeg().Traverse( |i| {
        let  	warp = chunk.At( i);
        let  	count = warp._Count.AsUsize();
        let  	modStart = warp._ModStart.AsUsize();
        SimEngine::< U64>::ForEachReadyLane( readyWords, modStart, count, |l| {
            let  	cb = &warp._Instances[l];
            let  	inTrigs = &warp._InTriggers[l];
            let  	outTrigs = &warp._OutTriggers[l];
            SimEngine::< U64>::EvalTraitInstance( cb, inTrigs, outTrigs, triggers);
        });
    });
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	coro_warp_spawn< 'a>( chunk: crate::silo::Arr< 'a, CoroWarp>, _w: &DynIWorker< '_>)
{
    let  	enginePtr = CURRENT_SIM_ENGINE.load( Ordering::Acquire);
    let  	engine = unsafe { &mut *enginePtr };
    let  	triggers = &mut engine._Triggers;
    chunk.USeg().Traverse( |i| {
        let  	warp = chunk.At( i);
        USeg::New( U32::_0, warp._Count).Traverse( |l| {
            let  	inTrigs = &warp._InTriggers[l];
            let  	outTrigs = &warp._OutTriggers[l];
            let  	coroCell = &warp._Instances[l];
            SimEngine::< U64>::EvalCoroInstance( coroCell, inTrigs, outTrigs, triggers);
        });
    });
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, PartialEq, Eq)]
pub enum SimEngineMode
{
    Serial,
    Parallel( U8),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct SimEngine< T: ITriggerVal = U64>
{
    pub _Triggers:        TriggerWad< T>,
    pub _FastWarps:       Buff< FastWarp>,
    pub _CustomWarps:     Buff< CustomWarp>,
    pub _CustomCallbacks: Buff< CustomKernelFn>,
    pub _BehavioralWarps: Buff< BehavioralWarp>,
    pub _TraitWarps:      Buff< crate::rube::module::TraitWarp>,
    pub _CoroWarps:       Buff< CoroWarp>,
    pub _PortToTrigger:   Buff< TriggerId>,
    pub _ReadyWords:      Buff< u64>,
    pub _CycleCount:      usize,
    pub _Mode:            SimEngineMode,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimEngine< U64>
{
    pub fn	Create( layout: &Layout) -> Self
    {
        return Self::CreateWithRegistry( layout, &KernelRegistry::Default());
    }

    pub fn	CreateWithRegistry( layout: &Layout, registry: &KernelRegistry) -> Self
    {
        return Self::CreateTypedWithRegistry( layout, registry);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: ITriggerVal> SimEngine< T>
{
    pub fn	CreateTyped( layout: &Layout) -> Self
    {
        return Self::CreateTypedWithRegistry( layout, &KernelRegistry::Default());
    }

    pub fn	CreateTypedWithRegistry( layout: &Layout, registry: &KernelRegistry) -> Self
    {
        let  	portToTrigger = layout.PortToTrigger();
        let  	triggers = layout.BuildTriggersTyped::< T>( &portToTrigger);
        let  	( fastWarps, customWarps, behavioralWarps, coroWarps, traitWarps) = layout.CompileWarps( &portToTrigger);
        let  	modCount = layout._Modules.Size().AsUsize();
        let  	wordCount = ( modCount + 63) / 64;
        let  	readyWords = Buff::Create( U32( wordCount as u32), |_| 0u64);

        let  	mut callbacks = crate::silo::Stash::WithCapacity( customWarps.Size());
        customWarps.Arr().Traverse( |warp| {
            if let Some( cb) = registry._Map.get( warp._KernelName) {
                callbacks.Push( Arc::clone( cb));
            } else {
                panic!( "Missing custom kernel: {}", warp._KernelName);
            }
        });

        return Self {
            _Triggers:        triggers,
            _FastWarps:       fastWarps,
            _CustomWarps:     customWarps,
            _CustomCallbacks: callbacks.IntoBuff(),
            _BehavioralWarps: behavioralWarps,
            _TraitWarps:      traitWarps,
            _CoroWarps:       coroWarps,
            _PortToTrigger:   portToTrigger,
            _ReadyWords:      readyWords,
            _CycleCount:      0,
            _Mode:            SimEngineMode::Serial,
        };
    }

    pub fn	WithMode( mut self, mode: SimEngineMode) -> Self
    {
        self._Mode = mode;
        self
    }

    #[inline]
    fn	ForEachReadyLane( readyWords: &Buff< u64>, modStart: usize, count: usize, mut f: impl FnMut( usize))
    {
        let  	mut lane = 0;
        while lane < count {
            let  	mIdx = modStart + lane;
            let  	wIdx = mIdx / 64;
            let  	bitOffset = mIdx % 64;
            let  	activeWord = readyWords[wIdx] >> bitOffset;

            let  	chunkLen = ( count - lane).min( 64 - bitOffset);
            let  	chunkMask = if chunkLen >= 64 { !0u64 } else { ( 1u64 << chunkLen) - 1 };
            let  	mut masked = activeWord & chunkMask;

            while masked != 0 {
                let  	b = masked.trailing_zeros() as usize;
                f( lane + b);
                masked &= masked - 1;
            }
            lane += chunkLen;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	Drive( &mut self) -> usize
    {
        let  	_guard = SIM_DRIVE_LOCK.Lock();

        let  	hasReady = self.ResolveReadyModules();
        let  	hasCoros = self._CoroWarps.Size() > U32::_0;

        if hasReady || hasCoros {
            let  	numWorkers = match self._Mode {
                SimEngineMode::Serial => U32( 0),
                SimEngineMode::Parallel( n) => U32::from( n),
            };

            Atelier::Init( numWorkers);
            let  	atelier = Atelier::Get();

            CURRENT_SIM_ENGINE.store( self as *mut SimEngine< T> as *mut SimEngine< U64>, Ordering::Release);

            let  	fastNode = CpuSpawnQuell!(
                self._FastWarps.Arr(),
                fast_warp_spawn,
                |_chunk, _w| {}
            );

            let  	customNode = CpuSpawnQuell!(
                self._CustomWarps.Arr(),
                custom_warp_spawn,
                |_chunk, _w| {}
            );

            let  	behavioralNode = CpuSpawnQuell!(
                self._BehavioralWarps.Arr(),
                behavioral_warp_spawn,
                |_chunk, _w| {}
            );

            let  	traitNode = CpuSpawnQuell!(
                self._TraitWarps.Arr(),
                trait_warp_spawn,
                |_chunk, _w| {}
            );

            let  	coroNode = CpuSpawnQuell!(
                self._CoroWarps.Arr(),
                coro_warp_spawn,
                |_chunk, _w| {}
            );

            let  	warpPhase = ChoreTree!( fastNode | customNode | behavioralNode | traitNode | coroNode );
            atelier.MainMaestro().PostChoreTree( &warpPhase );
            atelier.DoLaunch();
        }

        self._Triggers.AdvanceAll();
        self._CycleCount += 1;
        return self._CycleCount;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Drive_Parallel( &mut self, numWorkers: U8) -> usize
    {
        self._Mode = SimEngineMode::Parallel( numWorkers);
        return self.Drive();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	ResolveReadyModules( &mut self) -> bool
    {
        let  	wordCount = self._ReadyWords.Size();
        if self._CycleCount == 0 {
            USeg::New( U32::_0, wordCount).Traverse( |i| {
                self._ReadyWords[i] = !0u64;
            });
            return true;
        }

        USeg::New( U32::_0, wordCount).Traverse( |i| {
            self._ReadyWords[i] = 0u64;
        });

        let  	mut hasReady = false;
        USeg::New( U32::_0, self._Triggers.Size()).Traverse( |tIdx| {
            let  	trigId = tIdx;
            if self._Triggers.IsEdge( trigId) {
                let  	spans = self._Triggers._SubscriberSpans[tIdx];
                USeg::New( spans.First(), spans.Size()).Traverse( |sIdx| {
                    let  	mIdx = self._Triggers._Subscribers[sIdx].AsUsize();
                    let  	wIdx = mIdx / 64;
                    let  	bIdx = mIdx % 64;
                    if wIdx < self._ReadyWords.Size().AsUsize() {
                        self._ReadyWords[wIdx] |= 1u64 << bIdx;
                        hasReady = true;
                    }
                });
            }
        });

        return hasReady;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalCustomInstance(
        cb: &Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>,
        inTriggers: &Buff< TriggerId>,
        outTriggers: &Buff< TriggerId>,
        triggers: &mut TriggerWad< T>,
    )
    {
        let  	inLen = inTriggers.Size();
        let  	outLen = outTriggers.Size();

        if inLen.0 <= 16 && outLen.0 <= 16 {
            let  	mut inBuf = [Reg::default(); 16];
            let  	mut outBuf = [Reg::default(); 16];

            USeg::New( U32::_0, inLen).Traverse( |k| {
                inBuf[k.AsUsize()] = triggers.Current( inTriggers[k] );
            });
            USeg::New( U32::_0, outLen).Traverse( |k| {
                outBuf[k.AsUsize()] = triggers.Future( outTriggers[k] );
            });

            ( cb)( &inBuf[..inLen.AsUsize()], &mut outBuf[..outLen.AsUsize()]);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers.SetFuture( outTriggers[k], outBuf[k.AsUsize()] );
            });
        } else {
            let  	inVals = Buff::Create( inLen, |k| triggers.Current( inTriggers[k] ));
            let  	mut outVals = Buff::Create( outLen, |k| triggers.Future( outTriggers[k] ));

            ( cb)( inVals.Slice(), outVals.SliceMut());

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers.SetFuture( outTriggers[k], outVals[k] );
            });
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalTraitInstance(
        cb: &Arc< dyn crate::rube::kernel::IKernel>,
        inTriggers: &Buff< TriggerId>,
        outTriggers: &Buff< TriggerId>,
        triggers: &mut TriggerWad< T>,
    )
    {
        let  	inLen = inTriggers.Size();
        let  	outLen = outTriggers.Size();

        if inLen.0 <= 16 && outLen.0 <= 16 {
            let  	mut inBuf = [Reg::default(); 16];
            let  	mut outBuf = [Reg::default(); 16];

            USeg::New( U32::_0, inLen).Traverse( |k| {
                inBuf[k.AsUsize()] = triggers.Current( inTriggers[k] );
            });
            USeg::New( U32::_0, outLen).Traverse( |k| {
                outBuf[k.AsUsize()] = triggers.Future( outTriggers[k] );
            });

            let _ = cb.Execute( &inBuf[..inLen.AsUsize()], &mut outBuf[..outLen.AsUsize()]);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers.SetFuture( outTriggers[k], outBuf[k.AsUsize()] );
            });
        } else {
            let  	inVals = Buff::Create( inLen, |k| triggers.Current( inTriggers[k] ));
            let  	mut outVals = Buff::Create( outLen, |k| triggers.Future( outTriggers[k] ));

            let _ = cb.Execute( &inVals, &mut outVals);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers.SetFuture( outTriggers[k], outVals[k] );
            });
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalCoroInstance(
        coroCell: &CoroCell,
        inTriggers: &Buff< TriggerId>,
        outTriggers: &Buff< TriggerId>,
        triggers: &mut TriggerWad< T>,
    )
    {
        let  	inLen = inTriggers.Size();
        let  	outLen = outTriggers.Size();

        let  	mut inPorts = CoroPorts::New();
        let  	inCount = inLen.min( U32( CORO_MAX_PORTS as u32));
        USeg::New( U32::_0, inCount).Traverse( |k| {
            inPorts._Vals[k.AsUsize()] = triggers.Current( inTriggers[k] );
        });
        inPorts._Len = inCount;

        let  	coro = coroCell.GetMut();
        if coro.IsDone() {
            return;
        }

        match coro.Resume( inPorts) {
            CoroRes::Yield( outPorts) => {
                if outLen > U32::_0 {
                    let  	outCount = outLen.min( outPorts.Len());
                    USeg::New( U32::_0, outCount).Traverse( |k| {
                        triggers.SetFuture( outTriggers[k], outPorts._Vals[k.AsUsize()] );
                    });
                }
            }
            CoroRes::Done( _) => {}
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub const fn	CycleCount( &self) -> usize
    {
        return self._CycleCount;
    }

    #[inline]
    pub fn	Triggers( &self) -> &TriggerWad< T>
    {
        return &self._Triggers;
    }

    #[inline]
    pub fn	TriggersMut( &mut self) -> &mut TriggerWad< T>
    {
        return &mut self._Triggers;
    }

    #[inline]
    pub fn	GetTrigger( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.Current( id);
    }

    #[inline]
    pub fn	GetPastTrigger( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.Past( id);
    }

    #[inline]
    pub fn	GetFutureTrigger( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.Future( id);
    }

    #[inline]
    pub fn	SetTrigger( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers.SetFuture( id, val);
    }

    #[inline]
    pub fn	InitTrigger( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers.Init( id, val);
    }

    #[inline]
    pub fn	SetTriggerImmediate( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers.SetImmediate( id, val);
    }

    #[inline]
    pub fn	GetTriggerVal( &self, id: TriggerId) -> T
    {
        return self._Triggers.CurrentVal( id);
    }

    #[inline]
    pub fn	SetTriggerVal( &mut self, id: TriggerId, val: T)
    {
        self._Triggers.SetFutureVal( id, val);
    }

    #[inline]
    pub fn	SetTriggerImmediateVal( &mut self, id: TriggerId, val: T)
    {
        self._Triggers.SetImmediateVal( id, val);
    }

    #[inline]
    pub fn	GetPortTrigger( &self, portId: PortId) -> Option< TriggerId>
    {
        let  	idx = portId.Index();
        if idx >= self._PortToTrigger.Size() {
            return None;
        }
        return Some( self._PortToTrigger[idx]);
    }

    #[inline]
    pub fn	GetPortValue( &self, portId: PortId) -> Option< Reg>
    {
        let  	trigId = self.GetPortTrigger( portId)?;
        return Some( self.GetTrigger( trigId));
    }

    #[inline]
    pub fn	SetPortValue( &mut self, portId: PortId, val: Reg) -> bool
    {
        if let Some( trigId) = self.GetPortTrigger( portId) {
            self.SetTriggerImmediate( trigId, val);
            return true;
        }
        return false;
    }

    #[inline]
    pub fn	StagePortValue( &mut self, portId: PortId, val: Reg) -> bool
    {
        if let Some( trigId) = self.GetPortTrigger( portId) {
            self.SetTrigger( trigId, val);
            return true;
        }
        return false;
    }

    #[inline]
    pub fn	GetPortBool( &self, portId: PortId) -> Option< Reg>
    {
        return self.GetPortValue( portId).map( |v| v.AsBool());
    }

    #[inline]
    pub fn	SetPortBool( &mut self, portId: PortId, val: Reg) -> bool
    {
        return self.SetPortValue( portId, val.AsBool());
    }

    #[inline]
    pub fn	GetPortU32( &self, portId: PortId) -> Option< Reg>
    {
        return self.GetPortValue( portId).map( |v| v.Masked( 0xFFFF_FFFF));
    }

    #[inline]
    pub fn	SetPortU32( &mut self, portId: PortId, val: Reg) -> bool
    {
        return self.SetPortValue( portId, val.Masked( 0xFFFF_FFFF));
    }

    #[inline]
    pub fn	IsPosedge( &self, id: TriggerId) -> bool
    {
        return self._Triggers.IsPosedge( id);
    }

    #[inline]
    pub fn	IsNegedge( &self, id: TriggerId) -> bool
    {
        return self._Triggers.IsNegedge( id);
    }

    #[inline]
    pub fn	IsEdge( &self, id: TriggerId) -> bool
    {
        return self._Triggers.IsEdge( id);
    }

    #[inline]
    pub fn	GetPortOutput( &self, id: PortId) -> Reg
    {
        return self.GetTrigger( self._PortToTrigger[id.Index()]);
    }
}
