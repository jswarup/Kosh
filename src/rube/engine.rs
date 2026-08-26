//-- engine.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        module::{ KernelOp, ModuleId },
        port::PortId,
        reg::Reg,
        regval::RegVal,
        signal::{ SignalMeta, SignalState },
        trigger::TriggerId,
    },
    silo::{ Buff, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Compact 16-byte Copy struct for standard 2-in/1-out ( and 1-in/1-out) gates.
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FastModule
{
    pub _In1: TriggerId,
    pub _In2: TriggerId,
    pub _Out: TriggerId,
    pub _Op: KernelOp,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FastModule
{
    #[inline]
    pub fn	New( in1: TriggerId, in2: TriggerId, out: TriggerId, op: KernelOp) -> Self
    {
        return Self {
            _In1: in1,
            _In2: in2,
            _Out: out,
            _Op: op,
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
    pub _Callback: Arc< dyn Fn( &[RegVal], &mut [RegVal]) + Send + Sync>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CustomModule
{
    pub fn	New(
        moduleId: ModuleId,
        inTriggers: Buff< TriggerId>,
        outTriggers: Buff< TriggerId>,
        callback: Arc< dyn Fn( &[RegVal], &mut [RegVal]) + Send + Sync>,
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
/// Stores hot signal states in `_Signals` ( 48 bytes per cell, cache-aligned).
/// Stores cold metadata separately in `_Meta`.
/// Evaluates fast gates via 16-byte `FastModule` descriptors without heap overhead.
#[derive( Clone)]
pub struct SimEngine
{
    pub _Signals: Buff< SignalState>,
    pub _Meta: Buff< SignalMeta>,
    pub _FastModules: Buff< FastModule>,
    pub _CustomModules: Buff< CustomModule>,
    pub _PortToTrigger: Buff< TriggerId>,
    pub _CycleCount: usize,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimEngine
{
    pub fn	New(
        signals: Buff< SignalState>,
        meta: Buff< SignalMeta>,
        fastModules: Buff< FastModule>,
        customModules: Buff< CustomModule>,
        portToTrigger: Buff< TriggerId>,
    ) -> Self
    {
        return Self {
            _Signals: signals,
            _Meta: meta,
            _FastModules: fastModules,
            _CustomModules: customModules,
            _PortToTrigger: portToTrigger,
            _CycleCount: 0,
        };
    }

    /// Executes a single synchronous discrete-event simulation cycle with zero heap allocations:
    /// 1. Phase 1: Pure evaluation reading immutable Present values ( T) from AoS cells.
    /// 2. Phase 2: Commit output stages to the Future slots ( T+1).
    /// 3. Phase 3: Clock tick latching ( Past <- Present, Present <- Future).
    pub fn	Tick( &mut self) -> usize
    {
        let  	mut stagedOutputs: Vec<( TriggerId, RegVal)> = Vec::with_capacity( self._FastModules.len() + self._CustomModules.len());

        // Phase 1: Evaluate Fast Gates
        for i in 0..self._FastModules.len() {
            let  	fm = self._FastModules[i];
            let  	in1 = self._Signals[usize::from( fm._In1)]._Current;
            let  	in2 = self._Signals[usize::from( fm._In2)]._Current;

            let  	res = match fm._Op {
                KernelOp::Nand => !( in1 & in2),
                KernelOp::And => in1 & in2,
                KernelOp::Or => in1 | in2,
                KernelOp::Not => !in1,
                KernelOp::Xor => in1 ^ in2,
                KernelOp::Nor => !( in1 | in2),
                KernelOp::Xnor => !( in1 ^ in2),
            };
            stagedOutputs.push(( fm._Out, res));
        }

        // Phase 1b: Evaluate Custom Modules
        for i in 0..self._CustomModules.len() {
            let  	cm = &self._CustomModules[i];
            let  	inVals: Vec< RegVal> = ( 0..cm._InTriggers.len())
                .map( |k| self._Signals[usize::from( cm._InTriggers[k])]._Current)
                .collect();
            let  	mut outVals: Vec< RegVal> = ( 0..cm._OutTriggers.len())
                .map( |k| self._Signals[usize::from( cm._OutTriggers[k])]._Future)
                .collect();

            ( cm._Callback)( &inVals, &mut outVals);

            for k in 0..cm._OutTriggers.len() {
                stagedOutputs.push(( cm._OutTriggers[k], outVals[k]));
            }
        }

        // Phase 2: Commit ( Staged Future committed to AoS Signal Future slots)
        for ( trigId, val) in stagedOutputs {
            self._Signals[usize::from( trigId)]._Future = val;
        }

        // Phase 3: Clock Tick ( Advance each AoS Signal cell in contiguous memory)
        for i in 0..self._Signals.len() {
            self._Signals[i].Advance();
        }

        self._CycleCount += 1;
        return self._CycleCount;
    }

    #[inline]
    pub fn	CycleCount( &self) -> usize
    {
        return self._CycleCount;
    }

    #[inline]
    pub fn	Signals( &self) -> &Buff< SignalState>
    {
        return &self._Signals;
    }

    #[inline]
    pub fn	SignalsMut( &mut self) -> &mut Buff< SignalState>
    {
        return &mut self._Signals;
    }

    #[inline]
    pub fn	GetSignal( &self, id: TriggerId) -> RegVal
    {
        return self._Signals[usize::from( id)]._Current;
    }

    #[inline]
    pub fn	GetPastSignal( &self, id: TriggerId) -> RegVal
    {
        return self._Signals[usize::from( id)]._Past;
    }

    #[inline]
    pub fn	GetFutureSignal( &self, id: TriggerId) -> RegVal
    {
        return self._Signals[usize::from( id)]._Future;
    }

    #[inline]
    pub fn	SetSignal( &mut self, id: TriggerId, val: RegVal)
    {
        self._Signals[usize::from( id)]._Future = val;
    }

    #[inline]
    pub fn	InitSignal( &mut self, id: TriggerId, val: RegVal)
    {
        self._Signals[usize::from( id)].Init( val);
    }

    #[inline]
    pub fn	GetPortTrigger( &self, portId: PortId) -> Option< TriggerId>
    {
        let  	idx = usize::from( portId.0);
        if idx >= self._PortToTrigger.len() {
            return None;
        }
        return Some( self._PortToTrigger[idx]);
    }

    #[inline]
    pub fn	GetPortValue( &self, portId: PortId) -> Option< RegVal>
    {
        let  	trigId = self.GetPortTrigger( portId)?;
        return Some( self.GetSignal( trigId));
    }

    #[inline]
    pub fn	SetPortValue( &mut self, portId: PortId, val: RegVal) -> bool
    {
        if let Some( trigId) = self.GetPortTrigger( portId) {
            self.InitSignal( trigId, val);
            return true;
        }
        return false;
    }

    #[inline]
    pub fn	StagePortValue( &mut self, portId: PortId, val: RegVal) -> bool
    {
        if let Some( trigId) = self.GetPortTrigger( portId) {
            self.SetSignal( trigId, val);
            return true;
        }
        return false;
    }

    #[inline]
    pub fn	GetPortBool( &self, portId: PortId) -> Option< Reg< bool>>
    {
        return self.GetPortValue( portId).map( |v| v.AsBool());
    }

    #[inline]
    pub fn	SetPortBool( &mut self, portId: PortId, val: Reg< bool>) -> bool
    {
        return self.SetPortValue( portId, RegVal::FromRegBool( val));
    }

    #[inline]
    pub fn	GetPortU32( &self, portId: PortId) -> Option< Reg< U32>>
    {
        return self.GetPortValue( portId).map( |v| v.AsU32());
    }

    #[inline]
    pub fn	SetPortU32( &mut self, portId: PortId, val: Reg< U32>) -> bool
    {
        return self.SetPortValue( portId, RegVal::FromRegU32( val));
    }

    #[inline]
    pub fn	IsPosedge( &self, id: TriggerId) -> bool
    {
        return self._Signals[usize::from( id)].IsPosedge();
    }

    #[inline]
    pub fn	IsNegedge( &self, id: TriggerId) -> bool
    {
        return self._Signals[usize::from( id)].IsNegedge();
    }

    #[inline]
    pub fn	IsEdge( &self, id: TriggerId) -> bool
    {
        return self._Signals[usize::from( id)].IsEdge();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
