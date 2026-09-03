//-- engine.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        layout::Layout,
        module::{ CustomWarp, FastWarp },
        port::PortId,
        reg::Reg,
        trigger::{ ITriggerWad, TriggerId, TriggerWad },
    },
    silo::{ Buff, IAccess, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct SimEngine
{
    pub _Triggers:      TriggerWad,
    pub _FastWarps:     Buff< FastWarp>,
    pub _CustomWarps:   Buff< CustomWarp>,
    pub _PortToTrigger: Buff< TriggerId>,
    pub _ReadyWords:    Buff< u64>,
    pub _CycleCount:    usize,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimEngine
{
    pub fn	Create( layout: &Layout) -> Self
    {
        let  	portToTrigger = layout.PortToTrigger();
        let  	triggers = layout.BuildTriggers( &portToTrigger);
        let  	( fastWarps, customWarps) = layout.CompileWarps( &portToTrigger);
        let  	modCount = layout._Modules.Size().AsUsize();
        let  	wordCount = ( modCount + 63) / 64;
        let  	readyWords = Buff::Create( U32( wordCount as u32), |_| 0u64);

        return Self {
            _Triggers:      triggers,
            _FastWarps:     fastWarps,
            _CustomWarps:   customWarps,
            _PortToTrigger: portToTrigger,
            _ReadyWords:    readyWords,
            _CycleCount:    0,
        };
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Executes a single synchronous discrete-event simulation cycle with ZERO heap allocations:
    /// 1. Phase 1: Pure evaluation reading immutable Present values ( T) from AoS cells and writing directly to Future slots ( T+1).
    /// 2. Phase 2: Custom module evaluations.
    /// 3. Phase 3: Clock tick latching ( Past <- Present, Present <- Future).
    #[inline]
    pub fn	Drive( &mut self) -> usize
    {
        let  	hasReady = self.ResolveReadyModules();

        if hasReady {
            self.EvalFastWarps();
            self.EvalCustomWarps();
        }

        self._Triggers.AdvanceAll();
        self._CycleCount += 1;
        return self._CycleCount;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	ResolveReadyModules( &mut self) -> bool
    {
        let  	wordCount = self._ReadyWords.Size();
        if self._CycleCount == 0 {
            // First cycle: evaluate all modules to initialize combinational logic
            USeg::New( U32::_0, wordCount).Traverse( |i| {
                self._ReadyWords[i] = !0u64;
            });
            return true;
        }

        // Reset readiness words
        USeg::New( U32::_0, wordCount).Traverse( |i| {
            self._ReadyWords[i] = 0u64;
        });

        let  	mut hasReady = false;
        // Find triggers that changed and mark sensitive modules bitwise
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

    fn	EvalFastWarps( &mut self)
    {
        self._FastWarps.Arr().Traverse( |warp| {
            let  	op = warp._Op;
            let  	mask = warp._Mask;
            let  	count = warp._Count.AsUsize();
            let  	modStart = warp._ModStart.AsUsize();

            let  	mut lane = 0;
            while lane < count {
                let  	mIdx = modStart + lane;
                let  	wIdx = mIdx / 64;
                let  	bitOffset = mIdx % 64;
                let  	activeWord = self._ReadyWords[wIdx] >> bitOffset;

                let  	chunkLen = ( count - lane).min( 64 - bitOffset);
                let  	chunkMask = if chunkLen >= 64 { !0u64 } else { ( 1u64 << chunkLen) - 1 };
                if ( activeWord & chunkMask) == 0 {
                    lane += chunkLen;
                    continue;
                }

                for b in 0..chunkLen {
                    if ( activeWord & ( 1u64 << b)) != 0 {
                        let  	l = lane + b;
                        let  	in1 = self._Triggers._CurrentVals[warp._In1[l]];
                        let  	in2 = self._Triggers._CurrentVals[warp._In2[l]];
                        self._Triggers._FutureVals[warp._Out[l]] = op.Eval( in1, in2, mask);
                    }
                }
                lane += chunkLen;
            }
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalCustomWarps( &mut self)
    {
        self._CustomWarps.Arr().Traverse( |warp| {
            let  	count = warp._Count.AsUsize();
            let  	modStart = warp._ModStart.AsUsize();

            let  	mut lane = 0;
            while lane < count {
                let  	mIdx = modStart + lane;
                let  	wIdx = mIdx / 64;
                let  	bitOffset = mIdx % 64;
                let  	activeWord = self._ReadyWords[wIdx] >> bitOffset;

                let  	chunkLen = ( count - lane).min( 64 - bitOffset);
                let  	chunkMask = if chunkLen >= 64 { !0u64 } else { ( 1u64 << chunkLen) - 1 };
                if ( activeWord & chunkMask) == 0 {
                    lane += chunkLen;
                    continue;
                }

                for b in 0..chunkLen {
                    if ( activeWord & ( 1u64 << b)) != 0 {
                        let  	l = lane + b;
                        let  	cb = &warp._Instances[l];
                        let  	inTrigs = &warp._InTriggers[l];
                        let  	outTrigs = &warp._OutTriggers[l];
                        Self::EvalCustomInstance( cb, inTrigs, outTrigs, &mut self._Triggers);
                    }
                }
                lane += chunkLen;
            }
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalCustomInstance(
        cb: &Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>,
        inTriggers: &Buff< TriggerId>,
        outTriggers: &Buff< TriggerId>,
        triggers: &mut TriggerWad,
    )
    {
        let  	inLen = inTriggers.Size();
        let  	outLen = outTriggers.Size();

        // Stack-allocated buffers for modules with up to 16 inputs/outputs
        if inLen.0 <= 16 && outLen.0 <= 16 {
            let  	mut inBuf = [Reg::default(); 16];
            let  	mut outBuf = [Reg::default(); 16];

            USeg::New( U32::_0, inLen).Traverse( |k| {
                inBuf[k.AsUsize()] = triggers._CurrentVals[inTriggers[k]];
            });
            USeg::New( U32::_0, outLen).Traverse( |k| {
                outBuf[k.AsUsize()] = triggers._FutureVals[outTriggers[k]];
            });

            ( cb)( &inBuf[..inLen.AsUsize()], &mut outBuf[..outLen.AsUsize()]);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers._FutureVals[outTriggers[k]] = outBuf[k.AsUsize()];
            });
        } else {
            let  	inVals = Buff::Create( inLen, |k| triggers._CurrentVals[inTriggers[k]]);
            let  	mut outVals = Buff::Create( outLen, |k| triggers._FutureVals[outTriggers[k]]);

            ( cb)( &inVals, &mut outVals);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers._FutureVals[outTriggers[k]] = outVals[k];
            });
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub const fn	CycleCount( &self) -> usize
    {
        return self._CycleCount;
    }

    #[inline]
    pub fn	Triggers( &self) -> &TriggerWad
    {
        return &self._Triggers;
    }

    #[inline]
    pub fn	TriggersMut( &mut self) -> &mut TriggerWad
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
}

//---------------------------------------------------------------------------------------------------------------------------------
