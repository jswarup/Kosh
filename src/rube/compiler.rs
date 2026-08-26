//-- compiler.rs ---------------------------------------------------------------------------------------------------------------------

use	std::collections::BTreeMap;
use	std::sync::Arc;
use	crate::{
    rube::{
        engine::{ CustomModule, FastModule, SimEngine },
        layout::{ Layout, LayoutError },
        module::{ KernelKind, KernelOp },
        regval::RegVal,
        signal::{ SignalMeta, SignalState },
        trigger::TriggerId,
    },
    silo::{ Buff, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug, Default)]
pub struct NetCompiler;

//---------------------------------------------------------------------------------------------------------------------------------

impl NetCompiler
{
    pub fn	New() -> Self
    {
        return Self;
    }

    pub fn	Compile( &self, layout: &Layout) -> Result< SimEngine, LayoutError>
    {
        let  	portCount = layout._Ports.len();

        // Step 1: Disjoint Set Union ( DSU / Union-Find) to merge connected nets
        let  	mut parent: Vec< usize> = ( 0..portCount).collect();

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

        for &( srcOut, dstIn) in &layout._Connections {
            union( &mut parent, usize::from( srcOut.0), usize::from( dstIn.0));
        }

        // Step 2: Map canonical net roots to AoS SignalState & SignalMeta
        let  	mut rootToTrigger: BTreeMap< usize, TriggerId> = BTreeMap::new();
        let  	mut signals = Vec::new();
        let  	mut meta = Vec::new();
        let  	mut portToTrigger = Vec::new();

        for portIdx in 0..portCount {
            let  	root = find( &mut parent, portIdx);
            let  	trigId = if let Some( &existingId) = rootToTrigger.get( &root) {
                existingId
            } else {
                let  	rootPort = &layout._Ports[root];
                let  	newIdx = signals.len();
                let  	newId = U32( newIdx as u32);
                let  	defaultVal = RegVal::DefaultTyped( rootPort._PortType);

                signals.push( SignalState::New( defaultVal));
                meta.push( SignalMeta::New( rootPort.Name(), rootPort._PortType));
                rootToTrigger.insert( root, newId);
                newId
            };
            portToTrigger.push( trigId);
        }

        // Step 3: Categorize Fast Modules vs Custom Modules
        let  	mut fastModules = Vec::new();
        let  	mut customModules = Vec::new();

        for modIdx in 0..layout._Modules.len() {
            let  	module = &layout._Modules[modIdx];

            let  	mut inTriggers = Vec::new();
            for i in 0..module._InPorts.len() {
                let  	portId = module._InPorts[i];
                let  	trigId = portToTrigger[usize::from( portId.0)];
                inTriggers.push( trigId);
            }

            let  	mut outTriggers = Vec::new();
            for i in 0..module._OutPorts.len() {
                let  	portId = module._OutPorts[i];
                let  	trigId = portToTrigger[usize::from( portId.0)];
                outTriggers.push( trigId);
            }

            match &module._Kernel {
                KernelKind::Nand => {
                    let  	in1 = inTriggers[0];
                    let  	in2 = if inTriggers.len() > 1 { inTriggers[1] } else { in1 };
                    fastModules.push( FastModule::New( in1, in2, outTriggers[0], KernelOp::Nand));
                }
                KernelKind::And => {
                    let  	in1 = inTriggers[0];
                    let  	in2 = if inTriggers.len() > 1 { inTriggers[1] } else { in1 };
                    fastModules.push( FastModule::New( in1, in2, outTriggers[0], KernelOp::And));
                }
                KernelKind::Or => {
                    let  	in1 = inTriggers[0];
                    let  	in2 = if inTriggers.len() > 1 { inTriggers[1] } else { in1 };
                    fastModules.push( FastModule::New( in1, in2, outTriggers[0], KernelOp::Or));
                }
                KernelKind::Not => {
                    let  	in1 = inTriggers[0];
                    fastModules.push( FastModule::New( in1, in1, outTriggers[0], KernelOp::Not));
                }
                KernelKind::Xor => {
                    let  	in1 = inTriggers[0];
                    let  	in2 = if inTriggers.len() > 1 { inTriggers[1] } else { in1 };
                    fastModules.push( FastModule::New( in1, in2, outTriggers[0], KernelOp::Xor));
                }
                KernelKind::Nor => {
                    let  	in1 = inTriggers[0];
                    let  	in2 = if inTriggers.len() > 1 { inTriggers[1] } else { in1 };
                    fastModules.push( FastModule::New( in1, in2, outTriggers[0], KernelOp::Nor));
                }
                KernelKind::Xnor => {
                    let  	in1 = inTriggers[0];
                    let  	in2 = if inTriggers.len() > 1 { inTriggers[1] } else { in1 };
                    fastModules.push( FastModule::New( in1, in2, outTriggers[0], KernelOp::Xnor));
                }
                KernelKind::Custom( callback) => {
                    customModules.push( CustomModule::New(
                        module._Id,
                        Buff::from( inTriggers.as_slice()),
                        Buff::from( outTriggers.as_slice()),
                        Arc::clone( callback),
                    ));
                }
            }
        }

        return Ok( SimEngine::New(
            Buff::from( signals.as_slice()),
            Buff::from( meta.as_slice()),
            Buff::from( fastModules.as_slice()),
            Buff::from( customModules.as_slice()),
            Buff::from( portToTrigger.as_slice()),
        ));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
