//-- layout.rs -----------------------------------------------------------------------------------------------------------------------

use	std::{ fmt, sync::Arc };
use	crate::{
    rube::{
        engine::{ CustomModule, FastModule, SimEngine },
        module::{ KernelKind, Module, ModuleId },
        port::{ Port, PortDesc, PortDir, PortId, PortType },
        reg::Reg,
        trigger::{ TriggerId, TriggerMeta, TriggerState },
    },
    silo::{ Arr, Buff, EdgeConnect, IAccess, IEdgeConnect, Stash, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub enum LayoutError
{
    DuplicateInputDriver { _DstIn: PortId, _ExistingSrc: PortId, _AttemptedSrc: PortId },
    InvalidPortDirection { _Port: PortId, _Expected: PortDir, _Actual: PortDir },
    PortNotFound( PortId),
    ModuleNotFound( ModuleId),
    UnconnectedInput { _ModuleId: ModuleId, _PortId: PortId },
    TypeMismatch { _Src: PortId, _SrcType: PortType, _Dst: PortId, _DstType: PortType },
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for LayoutError
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::DuplicateInputDriver { _DstIn: dst, _ExistingSrc: src1, _AttemptedSrc: src2 } => {
                write!( f, "Input port {:?} already driven by {:?}, cannot also connect to {:?}", dst, src1, src2)
            }
            Self::InvalidPortDirection { _Port: p, _Expected: exp, _Actual: act } => {
                write!( f, "Port {:?} direction mismatch: expected {:?}, got {:?}", p, exp, act)
            }
            Self::PortNotFound( p) => write!( f, "Port {:?} not found", p),
            Self::ModuleNotFound( m) => write!( f, "Module {:?} not found", m),
            Self::UnconnectedInput { _ModuleId: m, _PortId: p } => {
                write!( f, "Module {:?} has unconnected input port {:?}", m, p)
            }
            Self::TypeMismatch { _Src: src, _SrcType: st, _Dst: dst, _DstType: dt } => {
                write!( f, "Type mismatch connecting {:?} ({:?}) to {:?} ({:?})", src, st, dst, dt)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::error::Error for LayoutError {}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Layout
{
    pub _Modules: Stash< Module>,
    pub _Ports: Stash< Port>,
    pub _Connections: EdgeConnect,
    pub _InDrivers: Stash< Option< PortId>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Layout
{
    fn	default() -> Self
    {
        return Self::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Layout
{
    #[inline]
    pub fn	New() -> Self
    {
        return Self {
            _Modules: Stash::New(),
            _Ports: Stash::New(),
            _Connections: EdgeConnect::New(),
            _InDrivers: Stash::New(),
        };
    }

    fn	AddPorts< 'a, T: 'a, P, F>(
        &mut self,
        modId: ModuleId,
        moduleName: &str,
        ports: P,
        dir: PortDir,
        extract: F,
    ) -> Buff< PortId>
    where
        P: Into< Arr< 'a, T>>,
        F: Fn( &'a T) -> ( &str, PortType),
    {
        let  	arr: Arr< 'a, T> = ports.into();
        let  	mut portIds = Stash::WithCapacity( arr.Size());
        for item in arr {
            let  	( portName, portType) = extract( item);
            let  	rawIdx = self._Ports.Size();
            let  	portId = if dir == PortDir::Out {
                PortId::Out( rawIdx)
            } else {
                PortId::In( rawIdx)
            };
            let  	fullName = format!( "{moduleName}.{portName}");
            let  	port = Port::New( portId, modId.0, &fullName, dir, portType);
            self._Ports.Push( port);
            self._InDrivers.Push( None);
            portIds.Push( portId);
        }
        return portIds.IntoBuff();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddModule< 'a, I, O>(
        &mut self,
        name: &str,
        inPorts: I,
        outPorts: O,
        kernel: KernelKind,
    ) -> ModuleId
    where
        I: Into< Arr< 'a, PortDesc>>,
        O: Into< Arr< 'a, PortDesc>>,
    {
        let  	modId = ModuleId( self._Modules.Size());
        let  	inPortIds = self.AddPorts( modId, name, inPorts, PortDir::In, |d| ( &d._Name, d._Type));
        let  	outPortIds = self.AddPorts( modId, name, outPorts, PortDir::Out, |d| ( &d._Name, d._Type));
        let  	module = Module::New(
            modId,
            name,
            inPortIds,
            outPortIds,
            kernel,
        );
        self._Modules.Push( module);
        return modId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	AddModuleSimple< 'a, I, O>(
        &mut self,
        name: &str,
        inPorts: I,
        outPorts: O,
        kernel: KernelKind,
    ) -> ModuleId
    where
        I: Into< Arr< 'a, &'a str>>,
        O: Into< Arr< 'a, &'a str>>,
    {
        let  	modId = ModuleId( self._Modules.Size());
        let  	inPortIds = self.AddPorts( modId, name, inPorts, PortDir::In, |&name| ( name, PortType::Bool));
        let  	outPortIds = self.AddPorts( modId, name, outPorts, PortDir::Out, |&name| ( name, PortType::Bool));
        let  	module = Module::New(
            modId,
            name,
            inPortIds,
            outPortIds,
            kernel,
        );
        self._Modules.Push( module);
        return modId;
    }

    #[inline]
    pub fn	InPort< K: Into< U32>>( &self, moduleId: ModuleId, portIdx: K) -> Option< PortId>
    {
        let  	idx = moduleId.0;
        if idx >= self._Modules.Size() {
            return None;
        }
        let  	module = &self._Modules[idx];
        let  	pIdx = portIdx.into();
        if pIdx >= module._InPorts.Size() {
            return None;
        }
        return Some( module._InPorts[pIdx]);
    }

    #[inline]
    pub fn	OutPort< K: Into< U32>>( &self, moduleId: ModuleId, portIdx: K) -> Option< PortId>
    {
        let  	idx = moduleId.0;
        if idx >= self._Modules.Size() {
            return None;
        }
        let  	module = &self._Modules[idx];
        let  	pIdx = portIdx.into();
        if pIdx >= module._OutPorts.Size() {
            return None;
        }
        return Some( module._OutPorts[pIdx]);
    }

    pub fn	Connect( &mut self, srcOut: PortId, dstIn: PortId) -> Result< (), LayoutError>
    {
        if !srcOut.IsOut() {
            return Err( LayoutError::InvalidPortDirection {
                _Port: srcOut,
                _Expected: PortDir::Out,
                _Actual: PortDir::In,
            });
        }
        if !dstIn.IsIn() {
            return Err( LayoutError::InvalidPortDirection {
                _Port: dstIn,
                _Expected: PortDir::In,
                _Actual: PortDir::Out,
            });
        }

        let  	srcIdx = srcOut.Index();
        let  	dstIdx = dstIn.Index();
        let  	portCount = self._Ports.Size();

        if srcIdx >= portCount {
            return Err( LayoutError::PortNotFound( srcOut));
        }
        if dstIdx >= portCount {
            return Err( LayoutError::PortNotFound( dstIn));
        }

        let  	srcPort = &self._Ports[srcIdx];
        let  	dstPort = &self._Ports[dstIdx];

        // Check Type Matching rule
        if srcPort._PortType != dstPort._PortType {
            return Err( LayoutError::TypeMismatch {
                _Src: srcOut,
                _SrcType: srcPort._PortType,
                _Dst: dstIn,
                _DstType: dstPort._PortType,
            });
        }

        // Check 1-to-1 input assignment rule
        if let Some( existingSrc) = self._InDrivers[dstIdx] {
            return Err( LayoutError::DuplicateInputDriver {
                _DstIn: dstIn,
                _ExistingSrc: existingSrc,
                _AttemptedSrc: srcOut,
            });
        }

        self._InDrivers[dstIdx] = Some( srcOut);
        self._Connections.RegisterEdge( srcIdx, dstIdx, false);
        return Ok( ());
    }

    #[inline]
    pub fn	Modules( &self) -> &[Module]
    {
        return self._Modules.Slice();
    }

    #[inline]
    pub fn	Ports( &self) -> &[Port]
    {
        return self._Ports.Slice();
    }

    #[inline]
    pub fn	Connections( &self) -> &EdgeConnect
    {
        return &self._Connections;
    }

    #[inline]
    pub fn	DumpDot( &self, ostr: &mut String)
    {
        self._Connections.DumpDot( ostr);
    }

    pub fn	Compile( &self) -> Result< SimEngine, LayoutError>
    {
        let  	portCountU32 = self._Ports.Size();

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

        self._Connections.TraverseAll( |_, srcOut, dstIn, _| {
            union( &mut parent, srcOut, dstIn);
        });

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
                let  	rootPort = &self._Ports[root];
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
        let  	modCount = self._Modules.Size();
        let  	mut fastModules = Stash::WithCapacity( modCount);
        let  	mut customModules = Stash::New();

        for modIdx in 0..modCount.0 {
            let  	module = &self._Modules[U32( modIdx)];

            let  	mut inTriggers = Stash::WithCapacity( module._InPorts.Size());
            for i in 0..module._InPorts.Size().0 {
                let  	portId = module._InPorts[U32( i)];
                let  	trigId = portToTrigger[portId.Index()];
                inTriggers.Push( trigId);
            }

            let  	mut outTriggers = Stash::WithCapacity( module._OutPorts.Size());
            for i in 0..module._OutPorts.Size().0 {
                let  	portId = module._OutPorts[U32( i)];
                let  	trigId = portToTrigger[portId.Index()];
                outTriggers.Push( trigId);
            }

            if let Some( op) = module._Kernel.ToFastOp() {
                let  	in1 = inTriggers[U32( 0)];
                let  	in2 = if inTriggers.Size() > U32( 1) { inTriggers[U32( 1)] } else { in1 };
                let  	outTrig = outTriggers[U32( 0)];
                let  	outPortId = module._OutPorts[U32( 0)];
                let  	outPortType = self._Ports[outPortId.Index()]._PortType;
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
