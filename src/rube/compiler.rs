//-- compiler.rs ---------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        engine::{ CustomModule, FastModule, SimEngine },
        layout::{ Layout, LayoutError },
        module::KernelKind,
        reg::Reg,
        trigger::{ TriggerId, TriggerMeta, TriggerState },
    },
    silo::{ Buff, Stash, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug, Default)]
pub struct NetCompiler;

//---------------------------------------------------------------------------------------------------------------------------------

impl NetCompiler
{
    #[inline]
    pub const fn	New() -> Self
    {
        return Self;
    }

    pub fn	Compile( &self, layout: &Layout) -> Result< SimEngine, LayoutError>
    {
        let  	portCountU32 = layout._Ports.Size();

        // Step 1: Disjoint Set Union ( DSU / Union-Find) to merge connected nets with path compression
        let  	mut parent = Buff::Create( portCountU32, |i| i);

        fn	find( p: &mut Buff< U32>, i: U32) -> U32
        {
            if p[i] == i {
                return i;
            }
            let  	root = find( p, p[i]);
            p[i] = root;
            return root;
        }

        fn	union( p: &mut Buff< U32>, a: U32, b: U32)
        {
            let  	rootA = find( p, a);
            let  	rootB = find( p, b);
            if rootA != rootB {
                p[rootB] = rootA;
            }
        }

        for &( srcOut, dstIn) in layout._Connections.Slice() {
            union( &mut parent, srcOut.0, dstIn.0);
        }

        // Step 2: Map canonical net roots to AoS TriggerState & TriggerMeta via direct indexed array
        let  	mut rootToTrigger = Buff::< Option< TriggerId>>::Create( portCountU32, |_| None);
        let  	mut triggers = Stash::WithCapacity( portCountU32);
        let  	mut meta = Stash::WithCapacity( portCountU32);
        let  	mut portToTrigger = Stash::WithCapacity( portCountU32);

        for portIdx in 0..portCountU32.0 {
            let  	pId = U32( portIdx);
            let  	root = find( &mut parent, pId);
            let  	trigId = if let Some( existingId) = rootToTrigger[root] {
                existingId
            } else {
                let  	rootPort = &layout._Ports[root];
                let  	newId = triggers.Size();
                let  	defaultVal = Reg::DefaultTyped( rootPort._PortType);

                triggers.Push( TriggerState::New( defaultVal));
                meta.Push( TriggerMeta::New( rootPort.Name(), rootPort._PortType));
                rootToTrigger[root] = Some( newId);
                newId
            };
            portToTrigger.Push( trigId);
        }

        // Step 3: Categorize Fast Modules vs Custom Modules
        let  	modCount = layout._Modules.Size();
        let  	mut fastModules = Stash::WithCapacity( modCount);
        let  	mut customModules = Stash::New();

        for modIdx in 0..modCount.0 {
            let  	module = &layout._Modules[U32( modIdx)];

            let  	mut inTriggers = Stash::WithCapacity( module._InPorts.Size());
            for i in 0..module._InPorts.Size().0 {
                let  	portId = module._InPorts[U32( i)];
                let  	trigId = portToTrigger[portId.0];
                inTriggers.Push( trigId);
            }

            let  	mut outTriggers = Stash::WithCapacity( module._OutPorts.Size());
            for i in 0..module._OutPorts.Size().0 {
                let  	portId = module._OutPorts[U32( i)];
                let  	trigId = portToTrigger[portId.0];
                outTriggers.Push( trigId);
            }

            if let Some( op) = module._Kernel.ToFastOp() {
                let  	in1 = inTriggers[U32( 0)];
                let  	in2 = if inTriggers.Size() > U32( 1) { inTriggers[U32( 1)] } else { in1 };
                let  	outTrig = outTriggers[U32( 0)];
                let  	outPortId = module._OutPorts[U32( 0)];
                let  	outPortType = layout._Ports[outPortId.0]._PortType;
                fastModules.Push( FastModule::New( in1, in2, outTrig, op, outPortType.Mask()));
            } else if let KernelKind::Custom( ref callback) = module._Kernel {
                customModules.Push( CustomModule::New(
                    module._Id,
                    inTriggers.IntoBuff(),
                    outTriggers.IntoBuff(),
                    Arc::clone( callback),
                ));
            }
        }

        return Ok( SimEngine::New(
            triggers.IntoBuff(),
            meta.IntoBuff(),
            fastModules.IntoBuff(),
            customModules.IntoBuff(),
            portToTrigger.IntoBuff(),
        ));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
