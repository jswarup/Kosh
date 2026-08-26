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
        let  	portCount = layout._Ports.Size().AsUsize();
        let  	portCountU32 = layout._Ports.Size();

        // Step 1: Disjoint Set Union ( DSU / Union-Find) to merge connected nets with path compression
        let  	mut parent = Buff::Create( portCountU32, |i| i.AsUsize());

        fn	find( p: &mut [usize], i: usize) -> usize
        {
            if p[i] == i {
                return i;
            }
            let  	root = find( p, p[i]);
            p[i] = root;
            return root;
        }

        fn	union( p: &mut [usize], a: usize, b: usize)
        {
            let  	rootA = find( p, a);
            let  	rootB = find( p, b);
            if rootA != rootB {
                p[rootB] = rootA;
            }
        }

        for &( srcOut, dstIn) in layout._Connections.Slice() {
            union( &mut parent, usize::from( srcOut.0), usize::from( dstIn.0));
        }

        // Step 2: Map canonical net roots to AoS TriggerState & TriggerMeta via direct indexed array
        let  	mut rootToTrigger = Buff::< Option< TriggerId>>::Create( portCountU32, |_| None);
        let  	mut triggers = Stash::WithCapacity( portCountU32);
        let  	mut meta = Stash::WithCapacity( portCountU32);
        let  	mut portToTrigger = Stash::WithCapacity( portCountU32);

        for portIdx in 0..portCount {
            let  	root = find( &mut parent, portIdx);
            let  	trigId = if let Some( existingId) = rootToTrigger[root] {
                existingId
            } else {
                let  	rootPort = &layout._Ports.Slice()[root];
                let  	newIdx = triggers.Size().AsUsize();
                let  	newId = U32( newIdx as u32);
                let  	defaultVal = Reg::DefaultTyped( rootPort._PortType);

                triggers.Push( TriggerState::New( defaultVal));
                meta.Push( TriggerMeta::New( rootPort.Name(), rootPort._PortType));
                rootToTrigger[root] = Some( newId);
                newId
            };
            portToTrigger.Push( trigId);
        }

        // Step 3: Categorize Fast Modules vs Custom Modules
        let  	modCount = layout._Modules.Size().AsUsize();
        let  	mut fastModules = Stash::WithCapacity( layout._Modules.Size());
        let  	mut customModules = Stash::New();

        for modIdx in 0..modCount {
            let  	module = &layout._Modules.Slice()[modIdx];

            let  	mut inTriggers = Stash::WithCapacity( U32( module._InPorts.len() as u32));
            for i in 0..module._InPorts.len() {
                let  	portId = module._InPorts[i];
                let  	trigId = portToTrigger.Slice()[usize::from( portId.0)];
                inTriggers.Push( trigId);
            }

            let  	mut outTriggers = Stash::WithCapacity( U32( module._OutPorts.len() as u32));
            for i in 0..module._OutPorts.len() {
                let  	portId = module._OutPorts[i];
                let  	trigId = portToTrigger.Slice()[usize::from( portId.0)];
                outTriggers.Push( trigId);
            }

            if let Some( op) = module._Kernel.ToFastOp() {
                let  	in1 = inTriggers.Slice()[0];
                let  	in2 = if inTriggers.Size().0 > 1 { inTriggers.Slice()[1] } else { in1 };
                fastModules.Push( FastModule::New( in1, in2, outTriggers.Slice()[0], op));
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
