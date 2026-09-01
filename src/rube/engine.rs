//-- engine.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::{
    rube::{
        layout::Layout,
        module::{ CustomModule, FastModule },
        port::PortId,
        reg::Reg,
        trigger::{ ITriggerWad, TriggerId, TriggerWad },
    },
    silo::{ Buff, IAccess, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// High-density AoS Synchronous Simulation Engine.
/// Stores hot trigger states in `_Triggers` ( 48 bytes per cell, cache-aligned).
/// Stores cold metadata separately in `_Meta`.
/// Evaluates fast gates via 16-byte `FastModule` descriptors without heap overhead.
#[derive( Clone)]
pub struct SimEngine
{
    pub _Triggers: TriggerWad,
    pub _FastModules: Buff< FastModule>,
    pub _CustomModules: Buff< CustomModule>,
    pub _PortToTrigger: Buff< TriggerId>,
    pub _ModuleReady: Buff< bool>,
    pub _CycleCount: usize,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimEngine
{
    pub fn	Create( layout: &Layout) -> Self
    {
        let  	( broadcast, portToTrigger) = layout.PartitionNets();
        let  	triggers = layout.BuildTriggers( &broadcast, &portToTrigger);
        let  	( fastModules, customModules) = layout.CompileModules( &portToTrigger);
        let  	modCount = layout._Modules.Size();
        let  	moduleReady = Buff::Create( modCount, |_| false);

        return Self {
            _Triggers:      triggers,
            _FastModules:   fastModules,
            _CustomModules: customModules,
            _PortToTrigger: portToTrigger,
            _ModuleReady:   moduleReady,
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
        let  	readyCount = self.ResolveReadyModules();

        if readyCount > U32( 0) {
            self.EvalFastModules();
            self.EvalCustomModules();
        }

        self._Triggers.AdvanceAll();
        self._CycleCount += 1;
        return self._CycleCount;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	ResolveReadyModules( &mut self) -> U32
    {
        let  	sz = self._ModuleReady.Size();
        if self._CycleCount == 0 {
            // First cycle: evaluate all modules to initialize combinational logic
            USeg::New( U32::_0, sz).Traverse( |i| {
                self._ModuleReady[i.AsUsize()] = true;
            });
            return sz;
        }

        // Reset readiness
        USeg::New( U32::_0, sz).Traverse( |i| {
            self._ModuleReady[i.AsUsize()] = false;
        });

        let  	mut readyCount = U32( 0);
        // Find triggers that changed and mark sensitive modules
        USeg::New( U32::_0, self._Triggers.Size()).Traverse( |tIdx| {
            let  	trigId = tIdx;
            if self._Triggers.IsEdge( trigId) {
                let  	spans = self._Triggers._SubscriberSpans[tIdx];
                USeg::New( spans.First(), spans.Size()).Traverse( |sIdx| {
                    let  	mIdx = self._Triggers._Subscribers[sIdx].AsUsize();
                    if !self._ModuleReady[mIdx] {
                        self._ModuleReady[mIdx] = true;
                        readyCount += U32( 1);
                    }
                });
            }
        });

        return readyCount;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalFastModules( &mut self)
    {
        self._FastModules.Arr().Traverse( |fm| {
            if self._ModuleReady[fm._ModuleId.0.AsUsize()] {
                let  	in1 = self._Triggers._CurrentVals[fm._In1];
                let  	in2 = self._Triggers._CurrentVals[fm._In2];
                self._Triggers._FutureVals[fm._Out] = fm._Op.Eval( in1, in2, fm._Mask);
            }
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalCustomModules( &mut self)
    {
        self._CustomModules.Arr().Traverse( |cm| {
            if self._ModuleReady[cm._ModuleId.0.AsUsize()] {
                Self::EvalCustomModule( cm, &mut self._Triggers);
            }
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EvalCustomModule( cm: &CustomModule, triggers: &mut TriggerWad)
    {
        let  	inLen = cm._InTriggers.Size();
        let  	outLen = cm._OutTriggers.Size();

        // Stack-allocated buffers for modules with up to 16 inputs/outputs
        if inLen.0 <= 16 && outLen.0 <= 16 {
            let  	mut inBuf = [Reg::default(); 16];
            let  	mut outBuf = [Reg::default(); 16];

            USeg::New( U32::_0, inLen).Traverse( |k| {
                inBuf[k.AsUsize()] = triggers._CurrentVals[cm._InTriggers[k]];
            });
            USeg::New( U32::_0, outLen).Traverse( |k| {
                outBuf[k.AsUsize()] = triggers._FutureVals[cm._OutTriggers[k]];
            });

            ( cm._Callback)( &inBuf[..inLen.AsUsize()], &mut outBuf[..outLen.AsUsize()]);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers._FutureVals[cm._OutTriggers[k]] = outBuf[k.AsUsize()];
            });
        } else {
            let  	inVals = Buff::Create( inLen, |k| triggers._CurrentVals[cm._InTriggers[k]]);
            let  	mut outVals = Buff::Create( outLen, |k| triggers._FutureVals[cm._OutTriggers[k]]);

            ( cm._Callback)( &inVals, &mut outVals);

            USeg::New( U32::_0, outLen).Traverse( |k| {
                triggers._FutureVals[cm._OutTriggers[k]] = outVals[k];
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
