//-- engine.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        layout::Layout,
        module::{ KernelKind, KernelOp, ModuleId },
        port::{ PortId, PortSensitivity },
        reg::Reg,
        trigger::{ ITriggerWad, TriggerId, TriggerSubscriber, TriggerWad },
    },
    silo::{ Buff, EdgeBroadcast, IAccess, IEdgeBroadcast, IEdgeConnect, Stash, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Compact 24-byte Copy struct for standard 2-in/1-out ( and 1-in/1-out) gates and bus arithmetic.
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FastModule
{
    pub _ModuleId: ModuleId,
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
    pub const fn	New( modId: ModuleId, in1: TriggerId, in2: TriggerId, out: TriggerId, op: KernelOp, mask: u64) -> Self
    {
        return Self {
            _ModuleId: modId,
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
        let  	portCountU32 = layout._Ports.Size();

        // Step 3: Traverse all connected ports using EdgeBroadcast to partition nets into unique trigger groups
        let  	mut broadcast = EdgeBroadcast::New( portCountU32);
        layout._Modules.Arr().Traverse( |module| {
            module._InPorts.Arr().Traverse( |&portId| {
                broadcast.DoBroadcast( portId.0, |elemId, _, _, nextStack| {
                    layout._Connections.NodeTraverse( elemId, |nextElem| {
                        nextStack.Push( nextElem);
                    });
                });
            });
            module._OutPorts.Arr().Traverse( |&portId| {
                broadcast.DoBroadcast( portId.0, |elemId, _, _, nextStack| {
                    layout._Connections.NodeTraverse( elemId, |nextElem| {
                        nextStack.Push( nextElem);
                    });
                });
            });
        });

        let  	groupCount = broadcast.SzGroup();
        let  	mut pastVals = Stash::WithCapacity( groupCount);
        let  	mut currentVals = Stash::WithCapacity( groupCount);
        let  	mut futureVals = Stash::WithCapacity( groupCount);

        USeg::New( U32::_0, groupCount).Traverse( |grpIdx| {
            let  	firstPortId = PortId( broadcast.FirstId( grpIdx));
            let  	rootPort = &layout._Ports[firstPortId.Index()];
            let  	defaultVal = Reg::DefaultTyped( rootPort._Type);

            pastVals.Push( defaultVal);
            currentVals.Push( defaultVal);
            futureVals.Push( defaultVal);
        });

        let  	portToTrigger = broadcast.SnitchNodeGroupIds();

        // Build adjacency list for TriggerWad subscribers
        let  	mut subscribersLists = Buff::Create( groupCount, |_| { Stash::New()});

        layout._Modules.Arr().Traverse( |module| {
            let  	inLen = module._InPorts.Size();
            USeg::New( U32::_0, inLen).Traverse( |i| {
                let  	portId = module._InPorts[i];
                let  	trigId = portToTrigger[portId.Index()];
                let  	sens = module._InSensitivities[i];
                if sens != PortSensitivity::None {
                    subscribersLists[ trigId].Push( TriggerSubscriber {
                        _ModIndex: module._Id.0,
                        _Sensitivity: sens,
                    });
                }
            });
        });

        let  	mut subscriberSpans = Stash::WithCapacity( groupCount);
        let  	mut subscribers = Stash::New();

        subscribersLists.Arr().Traverse( | list|  {
            let  	start = subscribers.Size();
            list.Arr().Traverse( |  sub| {
                subscribers.Push( *sub);
            });
            let  	sz = subscribers.Size() - start;
            subscriberSpans.Push( USeg::New( start, sz));
        });

        let  	triggers = TriggerWad::New(
            pastVals.IntoBuff(),
            currentVals.IntoBuff(),
            futureVals.IntoBuff(),
            subscriberSpans.IntoBuff(),
            subscribers.IntoBuff(),
        );

        // Step 3: Categorize Fast Modules vs Custom Modules
        let  	modCount = layout._Modules.Size();
        let  	mut fastModules = Stash::WithCapacity( modCount);
        let  	mut customModules = Stash::New();

        layout._Modules.Arr().Traverse( |module| {
            let  	mut inTriggers = Stash::WithCapacity( module._InPorts.Size());
            module._InPorts.Arr().Traverse( |portId| {
                inTriggers.Push( portToTrigger[portId.Index()]);
            });

            let  	mut outTriggers = Stash::WithCapacity( module._OutPorts.Size());
            module._OutPorts.Arr().Traverse( |portId| {
                outTriggers.Push( portToTrigger[portId.Index()]);
            });

            if let Some( op) = module._Kernel.ToFastOp() {
                let  	in1 = inTriggers[U32( 0)];
                let  	in2 = if inTriggers.Size() > U32( 1) { inTriggers[U32( 1)] } else { in1 };
                let  	outTrig = outTriggers[U32( 0)];
                let  	outPortId = module._OutPorts[U32( 0)];
                let  	outPortType = layout._Ports[outPortId.Index()]._Type;
                fastModules.Push( FastModule::New( module._Id, in1, in2, outTrig, op, outPortType.Mask()));
            } else if let KernelKind::Custom( ref callback) = module._Kernel {
                customModules.Push( CustomModule::New(
                    module._Id,
                    inTriggers.IntoBuff(),
                    outTriggers.IntoBuff(),
                    Arc::clone( callback),
                ));
            }
        });

        let  	moduleReady = Buff::Create( modCount, |_| false);

        return  Self {
            _Triggers: triggers,
            _FastModules:fastModules.IntoBuff(),
            _CustomModules: customModules.IntoBuff(),
            _PortToTrigger: portToTrigger,
            _ModuleReady: moduleReady,
            _CycleCount: 0
        };
    }

    /// Executes a single synchronous discrete-event simulation cycle with ZERO heap allocations:
    /// 1. Phase 1: Pure evaluation reading immutable Present values ( T) from AoS cells and writing directly to Future slots ( T+1).
    /// 2. Phase 2: Custom module evaluations.
    /// 3. Phase 3: Clock tick latching ( Past <- Present, Present <- Future).
    #[inline]
    pub fn	Drive( &mut self) -> usize
    {
        let  	mut readyCount = 0;

        if self._CycleCount == 0 {
            // First cycle: evaluate all modules to initialize combinational logic
            let  	sz = self._ModuleReady.Size();
            USeg::New( U32::_0, sz).Traverse( |i| {
                self._ModuleReady[i.AsUsize()] = true;
            });
            readyCount = sz.0;
        } else {
            // Reset readiness
            let  	sz = self._ModuleReady.Size();
            USeg::New( U32::_0, sz).Traverse( |i| {
                self._ModuleReady[i.AsUsize()] = false;
            });

            // Find triggers that changed and mark sensitive modules
            USeg::New( U32::_0, self._Triggers.Size()).Traverse( |tIdx| {
                let  	trigId = tIdx;
                if self._Triggers.IsEdge( trigId) {
                    let  	spans = self._Triggers._SubscriberSpans[tIdx];
                    USeg::New( spans.First(), spans.Size()).Traverse( |sIdx| {
                        let  	sub = self._Triggers._Subscribers[sIdx];
                        if self._Triggers.IsSensitive( trigId, sub._Sensitivity) {
                            let  	mIdx = sub._ModIndex.AsUsize();
                            if !self._ModuleReady[mIdx] {
                                self._ModuleReady[mIdx] = true;
                                readyCount += 1;
                            }
                        }
                    });
                }
            });
        }

        if readyCount > 0 {
            // Phase 1: Evaluate Fast Gates ( zero-heap-allocation, direct streaming write to _Future)
            self._FastModules.Arr().Traverse( |fm| {
                if self._ModuleReady[fm._ModuleId.0.AsUsize()] {
                    let  	in1 = self._Triggers._CurrentVals[fm._In1];
                    let  	in2 = self._Triggers._CurrentVals[fm._In2];
                    self._Triggers._FutureVals[fm._Out] = fm._Op.Eval( in1, in2, fm._Mask);
                }
            });

            // Phase 2: Evaluate Custom Modules
            self._CustomModules.Arr().Traverse( |cm| {
                if self._ModuleReady[cm._ModuleId.0.AsUsize()] {
                    let  	inLen = cm._InTriggers.Size();
                    let  	outLen = cm._OutTriggers.Size();

                    // Stack-allocated buffers for modules with up to 16 inputs/outputs
                    if inLen.0 <= 16 && outLen.0 <= 16 {
                        let  	mut inBuf = [Reg::default(); 16];
                        let  	mut outBuf = [Reg::default(); 16];
                        USeg::New( U32::_0, inLen).Traverse( |k| {
                            inBuf[k.AsUsize()] = self._Triggers._CurrentVals[cm._InTriggers[k]];
                        });
                        USeg::New( U32::_0, outLen).Traverse( |k| {
                            outBuf[k.AsUsize()] = self._Triggers._FutureVals[cm._OutTriggers[k]];
                        });

                        ( cm._Callback)( &inBuf[..inLen.AsUsize()], &mut outBuf[..outLen.AsUsize()]);

                        USeg::New( U32::_0, outLen).Traverse( |k| {
                            self._Triggers._FutureVals[cm._OutTriggers[k]] = outBuf[k.AsUsize()];
                        });
                    } else {
                        let  	inVals = Buff::Create( inLen, |k| self._Triggers._CurrentVals[cm._InTriggers[k]]);
                        let  	mut outVals = Buff::Create( outLen, |k| self._Triggers._FutureVals[cm._OutTriggers[k]]);

                        ( cm._Callback)( &inVals, &mut outVals);

                        USeg::New( U32::_0, outLen).Traverse( |k| {
                            self._Triggers._FutureVals[cm._OutTriggers[k]] = outVals[k];
                        });
                    }
                }
            });
        }

        // Phase 3: Clock Drive ( Advance contiguous slice of AoS cells)
        self._Triggers.AdvanceAll();

        self._CycleCount += 1;
        return self._CycleCount;
    }

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
