//-- engine.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        module::{ KernelOp, ModuleId },
        port::PortId,
        reg::Reg,
        trigger::{ TriggerId, TriggerMeta, TriggerState },
    },
    silo::{ Buff, IAccess, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Compact 24-byte Copy struct for standard 2-in/1-out ( and 1-in/1-out) gates and bus arithmetic.
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FastModule
{
    pub _In1: TriggerId,
    pub _In2: TriggerId,
    pub _Out: TriggerId,
    pub _Op: KernelOp,
    pub _Mask: u64,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FastModule
{
    #[inline]
    pub const fn	New( in1: TriggerId, in2: TriggerId, out: TriggerId, op: KernelOp, mask: u64) -> Self
    {
        return Self {
            _In1: in1,
            _In2: in2,
            _Out: out,
            _Op: op,
            _Mask: mask,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct CustomModule
{
    pub _ModuleId: ModuleId,
    pub _InTriggers: Buff< TriggerId>,
    pub _OutTriggers: Buff< TriggerId>,
    pub _Callback: Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CustomModule
{
    pub fn	New(
        moduleId: ModuleId,
        inTriggers: Buff< TriggerId>,
        outTriggers: Buff< TriggerId>,
        callback: Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>,
    ) -> Self
    {
        return Self {
            _ModuleId: moduleId,
            _InTriggers: inTriggers,
            _OutTriggers: outTriggers,
            _Callback: callback,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// High-density AoS Synchronous Simulation Engine.
/// Stores hot trigger states in `_Triggers` ( 48 bytes per cell, cache-aligned).
/// Stores cold metadata separately in `_Meta`.
/// Evaluates fast gates via 16-byte `FastModule` descriptors without heap overhead.
#[derive( Clone)]
pub struct SimEngine
{
    pub _Triggers: Buff< TriggerState>,
    pub _Meta: Buff< TriggerMeta>,
    pub _FastModules: Buff< FastModule>,
    pub _CustomModules: Buff< CustomModule>,
    pub _PortToTrigger: Buff< TriggerId>,
    pub _CycleCount: usize,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimEngine
{
    pub fn	New(
        triggers: Buff< TriggerState>,
        meta: Buff< TriggerMeta>,
        fastModules: Buff< FastModule>,
        customModules: Buff< CustomModule>,
        portToTrigger: Buff< TriggerId>,
    ) -> Self
    {
        return Self {
            _Triggers: triggers,
            _Meta: meta,
            _FastModules: fastModules,
            _CustomModules: customModules,
            _PortToTrigger: portToTrigger,
            _CycleCount: 0,
        };
    }

    /// Executes a single synchronous discrete-event simulation cycle with ZERO heap allocations:
    /// 1. Phase 1: Pure evaluation reading immutable Present values ( T) from AoS cells and writing directly to Future slots ( T+1).
    /// 2. Phase 2: Custom module evaluations.
    /// 3. Phase 3: Clock tick latching ( Past <- Present, Present <- Future).
    #[inline]
    pub fn	Tick( &mut self) -> usize
    {
        // Phase 1: Evaluate Fast Gates ( zero-heap-allocation, direct streaming write to _Future)
        self._FastModules.Arr().Traverse( |fm| {
            let  	in1 = self._Triggers[fm._In1]._Current;
            let  	in2 = self._Triggers[fm._In2]._Current;
            self._Triggers[fm._Out]._Future = fm._Op.Eval( in1, in2, fm._Mask);
        });

        // Phase 2: Evaluate Custom Modules
        self._CustomModules.Arr().Traverse( |cm| {
            let  	inLen = cm._InTriggers.Size();
            let  	outLen = cm._OutTriggers.Size();

            // Stack-allocated buffers for modules with up to 16 inputs/outputs
            if inLen.0 <= 16 && outLen.0 <= 16 {
                let  	mut inBuf = [Reg::default(); 16];
                let  	mut outBuf = [Reg::default(); 16];
                USeg::New( U32::_0, inLen).Traverse( |k| {
                    inBuf[k.AsUsize()] = self._Triggers[cm._InTriggers[k]]._Current;
                });
                USeg::New( U32::_0, outLen).Traverse( |k| {
                    outBuf[k.AsUsize()] = self._Triggers[cm._OutTriggers[k]]._Future;
                });

                ( cm._Callback)( &inBuf[..inLen.AsUsize()], &mut outBuf[..outLen.AsUsize()]);

                USeg::New( U32::_0, outLen).Traverse( |k| {
                    self._Triggers[cm._OutTriggers[k]]._Future = outBuf[k.AsUsize()];
                });
            } else {
                let  	inVals = Buff::Create( inLen, |k| self._Triggers[cm._InTriggers[k]]._Current);
                let  	mut outVals = Buff::Create( outLen, |k| self._Triggers[cm._OutTriggers[k]]._Future);

                ( cm._Callback)( &inVals, &mut outVals);

                USeg::New( U32::_0, outLen).Traverse( |k| {
                    self._Triggers[cm._OutTriggers[k]]._Future = outVals[k];
                });
            }
        });

        // Phase 3: Clock Tick ( Advance contiguous slice of AoS cells)
        USeg::New( U32::_0, self._Triggers.Size()).Traverse( |i| {
            self._Triggers[i].Advance();
        });

        self._CycleCount += 1;
        return self._CycleCount;
    }

    #[inline]
    pub const fn	CycleCount( &self) -> usize
    {
        return self._CycleCount;
    }

    #[inline]
    pub fn	Triggers( &self) -> &Buff< TriggerState>
    {
        return &self._Triggers;
    }

    #[inline]
    pub fn	TriggersMut( &mut self) -> &mut Buff< TriggerState>
    {
        return &mut self._Triggers;
    }

    #[inline]
    pub fn	GetTrigger( &self, id: TriggerId) -> Reg
    {
        return self._Triggers[id]._Current;
    }

    #[inline]
    pub fn	GetPastTrigger( &self, id: TriggerId) -> Reg
    {
        return self._Triggers[id]._Past;
    }

    #[inline]
    pub fn	GetFutureTrigger( &self, id: TriggerId) -> Reg
    {
        return self._Triggers[id]._Future;
    }

    #[inline]
    pub fn	SetTrigger( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers[id]._Future = val;
    }

    #[inline]
    pub fn	InitTrigger( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers[id].Init( val);
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
            self.InitTrigger( trigId, val);
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
        return self._Triggers[id].IsPosedge();
    }

    #[inline]
    pub fn	IsNegedge( &self, id: TriggerId) -> bool
    {
        return self._Triggers[id].IsNegedge();
    }

    #[inline]
    pub fn	IsEdge( &self, id: TriggerId) -> bool
    {
        return self._Triggers[id].IsEdge();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
