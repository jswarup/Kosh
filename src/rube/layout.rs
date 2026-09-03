//-- layout.rs -----------------------------------------------------------------------------------------------------------------------

use	std::{ fmt, sync::Arc };
use	crate::{
    rube::{
        module::{ CustomModule, CustomWarp, FastModule, FastWarp, KernelKind, Module, ModuleId },
        port::{ PortDesc, PortDir, PortId, PortType },
        reg::Reg,
        trigger::{ TriggerId, TriggerWad },
    },
    silo::{ Arr, Buff, EdgeBroadcast, EdgeConnect, IAccess, IArr, IEdgeBroadcast, IEdgeConnect, Stash, U32, USeg },
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
            LayoutError::DuplicateInputDriver { _DstIn, _ExistingSrc, _AttemptedSrc } => {
                write!( f, "Input port {:?} already driven by {:?}, cannot connect from {:?}", _DstIn, _ExistingSrc, _AttemptedSrc)
            }
            LayoutError::InvalidPortDirection { _Port, _Expected, _Actual } => {
                write!( f, "Port {:?} expected {:?} but is {:?}", _Port, _Expected, _Actual)
            }
            LayoutError::PortNotFound( portId) => write!( f, "Port {:?} not found", portId),
            LayoutError::ModuleNotFound( modId) => write!( f, "Module {:?} not found", modId),
            LayoutError::UnconnectedInput { _ModuleId, _PortId } => {
                write!( f, "Module {:?} has unconnected input port {:?}", _ModuleId, _PortId)
            }
            LayoutError::TypeMismatch { _Src, _SrcType, _Dst, _DstType } => {
                write!( f, "Type mismatch connecting {:?} ({:?}) to {:?} ({:?})", _Src, _SrcType, _Dst, _DstType)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::error::Error for LayoutError {}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Layout
{
    pub _Modules:         Stash< Module>,
    pub _Ports:           Stash< PortDesc>,
    pub _Connections:     EdgeConnect,
    pub _ModuleChildren:  Stash< Stash< ModuleId>>,
    pub _SubModules:      Stash< ModuleId>,
    pub _Descendents:     Stash< ModuleId>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Layout
{
    fn	default() -> Self
    {
        Self::New()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Layout
{
    pub fn	New() -> Self
    {
        Self {
            _Modules:         Stash::New(),
            _Ports:           Stash::New(),
            _Connections:     EdgeConnect::New(),
            _ModuleChildren:  Stash::New(),
            _SubModules:      Stash::New(),
            _Descendents:     Stash::New(),
        }
    }

    fn	AddPorts< 'a, T: 'a, P, F>(
        &mut self,
        modId: ModuleId,
        moduleName: &str,
        ports: P,
        extract: F,
    ) -> USeg
    where
        P: Into< Arr< 'a, T>>,
        F: Fn( &T) -> PortDesc,
    {
        let  	arr: Arr< 'a, T> = ports.into();
        let  	start = self._Ports.Size();
        let  	count = arr.Size();
        arr.Traverse( |item| {
            let  	mut desc = extract( item);
            desc._Name = format!( "{moduleName}.{}", desc._Name);
            desc._Owner = modId;
            self._Ports.Push( desc);
        });
        return USeg::New( start, count);
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
        let  	inArr: Arr< 'a, PortDesc> = inPorts.into();
        let  	inSeg = self.AddPorts( modId, name, inArr, |d| d.clone());
        let  	outSeg = self.AddPorts( modId, name, outPorts, |d| d.clone());
        let  	module = Module::New(
            modId,
            name,
            inSeg,
            outSeg,
            kernel,
        );
        self._Modules.Push( module);
        self._ModuleChildren.Push( Stash::New());
        return modId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	AddStdModule< 'a, I, O>(
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
        let  	inSeg = self.AddPorts( modId, name, inPorts, |&name| PortDesc::Bool( name));
        let  	outSeg = self.AddPorts( modId, name, outPorts, |&name| PortDesc::Bool( name));
        let  	module = Module::New(
            modId,
            name,
            inSeg,
            outSeg,
            kernel,
        );
        self._Modules.Push( module);
        self._ModuleChildren.Push( Stash::New());
        return modId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddContainer( &mut self, name: &str) -> ModuleId
    {
        let  	modId = ModuleId( self._Modules.Size());
        let  	module = Module::NewContainer( modId, name);
        self._Modules.Push( module);
        self._ModuleChildren.Push( Stash::New());
        return modId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddContainerWithPorts< 'a, I, O>(
        &mut self,
        name: &str,
        inPorts: I,
        outPorts: O,
    ) -> ModuleId
    where
        I: Into< Arr< 'a, PortDesc>>,
        O: Into< Arr< 'a, PortDesc>>,
    {
        let  	modId = ModuleId( self._Modules.Size());
        let  	inArr: Arr< 'a, PortDesc> = inPorts.into();
        let  	inSeg = self.AddPorts( modId, name, inArr, |d| d.clone());
        let  	outSeg = self.AddPorts( modId, name, outPorts, |d| d.clone());
        let  	mut module = Module::NewContainer( modId, name);
        module._InPorts = inSeg;
        module._OutPorts = outSeg;
        self._Modules.Push( module);
        self._ModuleChildren.Push( Stash::New());
        return modId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddSubModule( &mut self, parent: ModuleId, child: ModuleId)
    {
        assert!( parent.0 < self._Modules.Size(), "Parent ModuleId out of bounds");
        assert!( child.0 < self._Modules.Size(), "Child ModuleId out of bounds");
        self._ModuleChildren[parent.0].Push( child);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddContainerUnder( &mut self, parent: ModuleId, name: &str) -> ModuleId
    {
        let  	child = self.AddContainer( name);
        self.AddSubModule( parent, child);
        return child;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

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
        return Some( PortId::In( module._InPorts.First() + pIdx));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

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
        return Some( PortId::Out( module._OutPorts.First() + pIdx));
    }

    #[inline]
    pub fn	Connect( &mut self, srcOut: PortId, dstIn: PortId) -> &mut Self
    {
        self._Connections.RegisterEdge( srcOut.0, dstIn.0, true);
        return self;
    }

    pub fn	Validate( &self) -> Result< (), LayoutError>
    {
        let  	portCount = self._Ports.Size();
        let  	mut err = None;

        self._Modules.Arr().Traverse( |module| {
            if err.is_some() {
                return;
            }
            module._InPorts.Traverse( |idx| {
                if err.is_some() {
                    return;
                }
                let  	inPort = PortId::In( idx);
                let  	dstIdx = inPort.Index();
                if dstIdx >= portCount {
                    err = Some( LayoutError::PortNotFound( inPort));
                    return;
                }

                let  	edSeg = self._Connections.EdgeSeg( inPort.0);
                let  	driverCount = edSeg.Size();

                if driverCount > U32( 1) {
                    let  	e0 = self._Connections.EdgeAt( edSeg.First());
                    let  	e1 = self._Connections.EdgeAt( edSeg.First() + U32( 1));
                    err = Some( LayoutError::DuplicateInputDriver {
                        _DstIn: inPort,
                        _ExistingSrc: PortId( e0[1]),
                        _AttemptedSrc: PortId( e1[1]),
                    });
                    return;
                }

                if driverCount == U32( 1) {
                    let  	e = self._Connections.EdgeAt( edSeg.First());
                    let  	srcOut = PortId( e[1]);
                    if !srcOut.IsOut() {
                        err = Some( LayoutError::InvalidPortDirection {
                            _Port: srcOut,
                            _Expected: PortDir::Out,
                            _Actual: PortDir::In,
                        });
                        return;
                    }
                    let  	srcIdx = srcOut.Index();
                    if srcIdx >= portCount {
                        err = Some( LayoutError::PortNotFound( srcOut));
                        return;
                    }
                    let  	srcPort = &self._Ports[srcIdx];
                    let  	dstPort = &self._Ports[dstIdx];
                    if srcPort._Type != dstPort._Type {
                        err = Some( LayoutError::TypeMismatch {
                            _Src: srcOut,
                            _SrcType: srcPort._Type,
                            _Dst: inPort,
                            _DstType: dstPort._Type,
                        });
                        return;
                    }
                }
            });

            if err.is_some() {
                return;
            }

            module._OutPorts.Traverse( |idx| {
                if err.is_some() {
                    return;
                }
                let  	outPort = PortId::Out( idx);
                let  	srcIdx = outPort.Index();
                if srcIdx >= portCount {
                    err = Some( LayoutError::PortNotFound( outPort));
                    return;
                }

                let  	edSeg = self._Connections.EdgeSeg( outPort.0);
                edSeg.Traverse( |edIdx| {
                    if err.is_some() {
                        return;
                    }
                    let  	e = self._Connections.EdgeAt( edIdx);
                    let  	dstIn = PortId( e[1]);
                    if !dstIn.IsIn() {
                        err = Some( LayoutError::InvalidPortDirection {
                            _Port: dstIn,
                            _Expected: PortDir::In,
                            _Actual: PortDir::Out,
                        });
                    }
                });
            });
        });

        if let Some( e) = err {
            return Err( e);
        }
        return Ok( ());
    }

    #[inline]
    pub fn	Modules( &self) -> &[Module]
    {
        return self._Modules.Slice();
    }

    #[inline]
    pub fn	Ports( &self) -> &[PortDesc]
    {
        return self._Ports.Slice();
    }

    #[inline]
    pub fn	Port( &self, portId: PortId) -> Option< &PortDesc>
    {
        let  	idx = portId.Index();
        if idx < self._Ports.Size() {
            return Some( &self._Ports[idx]);
        }
        return None;
    }

    #[inline]
    pub fn	PortOwner( &self, portId: PortId) -> Option< ModuleId>
    {
        let  	idx = portId.Index();
        if idx < self._Ports.Size() {
            return Some( self._Ports[idx]._Owner);
        }
        return None;
    }

    #[inline]
    pub fn	Connections( &self) -> &EdgeConnect
    {
        return &self._Connections;
    }

    #[inline]
    pub fn	ConnectionsMut( &mut self) -> &mut EdgeConnect
    {
        return &mut self._Connections;
    }

    #[inline]
    pub fn	DumpDot( &self, ostr: &mut String)
    {
        self._Connections.DumpDot( ostr);
    }

    pub fn	SortModules( &mut self)
    {
        let  	modCount = self._Modules.Size();
        if modCount == U32( 0) {
            return;
        }

        if modCount == U32( 1) {
            self._SubModules.Clear();
            self._Descendents.Clear();
            self._ModuleChildren.Clear();
            self._Modules[U32( 0)]._SubModules = USeg::New( U32::_0, U32::_0);
            self._Modules[U32( 0)]._Descendents = USeg::New( U32::_0, U32::_0);
            return;
        }

        let  	arr = self._Modules.Arr();
        let  	lessFn = move |i, j| {
            if arr.At( i)._Kernel.ClassKey() == arr.At( j)._Kernel.ClassKey() {
                arr.At( i)._Id < arr.At( j)._Id
            } else {
                arr.At( i)._Kernel.ClassKey() < arr.At( j)._Kernel.ClassKey()
            }
        };
        let  	swapFn = move |i, j| arr.Swap( i, j);
        arr.USeg().QSort( lessFn, swapFn);

        let  	mut oldToNew = Stash::WithCapacityVal( modCount, ModuleId( U32::_0));
        USeg::New( U32::_0, modCount).Traverse( |newIdx| {
            let  	oldId = self._Modules[newIdx]._Id;
            oldToNew[oldId.0] = ModuleId( newIdx);
        });

        // Compute total direct sub-modules to pre-reserve contiguous capacity in _SubModules
        let  	mut totalSub = U32( 0);
        USeg::New( U32::_0, modCount).Traverse( |i| {
            totalSub += self._ModuleChildren[i].Size();
        });

        self._SubModules.Clear();
        self._SubModules.Reserve( totalSub);

        // Single pass: re-index modules, update port owners, and flatten mapped submodules
        USeg::New( U32::_0, modCount).Traverse( |newIdx| {
            let  	oldId = self._Modules[newIdx]._Id;
            let  	newModId = ModuleId( newIdx);
            self._Modules[newIdx]._Id = newModId;

            self._Modules[newIdx]._InPorts.Traverse( |idx| {
                self._Ports[idx]._Owner = newModId;
            });
            self._Modules[newIdx]._OutPorts.Traverse( |idx| {
                self._Ports[idx]._Owner = newModId;
            });

            let  	start = self._SubModules.Size();
            let  	oldChildren = &self._ModuleChildren[oldId.0];
            let  	count = oldChildren.Size();
            oldChildren.Arr().Traverse( |&childOldId| {
                self._SubModules.Push( oldToNew[childOldId.0]);
            });
            self._Modules[newIdx]._SubModules = USeg::New( start, count);
        });

        // Drop temporary dynamic child vectors now that _SubModules is contiguous
        self._ModuleChildren.Clear();

        // Flatten transitive recursive descendents into contiguous self._Descendents
        self._Descendents.Clear();
        self._Descendents.Reserve( totalSub);

        // Single reusable visited array and stack across all module traversals
        let  	mut visited = Stash::WithCapacityVal( modCount, false);
        let  	mut stack = Stash::WithCapacity( modCount);

        USeg::New( U32::_0, modCount).Traverse( |newIdx| {
            let  	start = self._Descendents.Size();

            self._Modules[newIdx]._SubModules.Traverse( |childSlot| {
                let  	childId = self._SubModules[childSlot];
                stack.Push( childId);
                visited[childId.0] = true;
            });

            while stack.Size() > U32::_0 {
                let  	curr = stack.Pop().unwrap();
                self._Descendents.Push( curr);
                self._Modules[curr.0]._SubModules.Traverse( |childSlot| {
                    let  	childId = self._SubModules[childSlot];
                    if !visited[childId.0] {
                        visited[childId.0] = true;
                        stack.Push( childId);
                    }
                });
            }

            let  	count = self._Descendents.Size() - start;
            self._Modules[newIdx]._Descendents = USeg::New( start, count);

            // Reset visited flags only for the nodes that were marked (O(deg) instead of O(N))
            USeg::New( start, count).Traverse( |dIdx| {
                visited[self._Descendents[dIdx].0] = false;
            });
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	SubModules( &self, moduleId: ModuleId) -> &[ModuleId]
    {
        let  	idx = moduleId.0;
        if idx < self._Modules.Size() {
            let  	seg = self._Modules[idx]._SubModules;
            if !seg.IsEmpty() && seg.End() <= self._SubModules.Size() {
                return &self._SubModules.Slice()[seg.First().AsUsize()..seg.End().AsUsize()];
            }
            if idx < self._ModuleChildren.Size() {
                return self._ModuleChildren[idx].Slice();
            }
        }
        return &[];
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	Descendents( &self, moduleId: ModuleId) -> &[ModuleId]
    {
        let  	idx = moduleId.0;
        if idx < self._Modules.Size() {
            let  	seg = self._Modules[idx]._Descendents;
            if !seg.IsEmpty() && seg.End() <= self._Descendents.Size() {
                return &self._Descendents.Slice()[seg.First().AsUsize()..seg.End().AsUsize()];
            }
        }
        return &[];
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	IsContainer( &self, moduleId: ModuleId) -> bool
    {
        let  	idx = moduleId.0;
        if idx < self._Modules.Size() {
            return self._Modules[idx].IsContainer()
                || !self._Modules[idx]._SubModules.IsEmpty()
                || ( idx < self._ModuleChildren.Size() && self._ModuleChildren[idx].Size() > U32( 0));
        }
        return false;
    }

    pub fn	RootModules( &self) -> Stash< ModuleId>
    {
        let  	mut hasParent = Stash::WithCapacity( self._Modules.Size());
        USeg::New( U32::_0, self._Modules.Size()).Traverse( |_| {
            hasParent.Push( false);
        });

        USeg::New( U32::_0, self._Modules.Size()).Traverse( |i| {
            self.SubModules( ModuleId( i)).iter().for_each( |&childId| {
                hasParent[childId.0] = true;
            });
        });

        let  	mut roots = Stash::New();
        USeg::New( U32::_0, self._Modules.Size()).Traverse( |i| {
            if !hasParent[i] {
                roots.Push( ModuleId( i));
            }
        });
        return roots;
    }

    pub fn	DumpHierarchy( &self, ostr: &mut String)
    {
        let  	roots = self.RootModules();
        let  	count = roots.Size().AsUsize();
        for ( idx, &rootId) in roots.Slice().iter().enumerate() {
            let  	isLast = idx + 1 == count;
            self.DumpHierarchyNode( rootId, "", true, isLast, ostr);
        }
    }

    fn	DumpHierarchyNode( &self, modId: ModuleId, prefix: &str, isRoot: bool, isLast: bool, ostr: &mut String)
    {
        let  	m = &self._Modules[modId.0];
        let  	marker = if isRoot {
            ""
        } else if isLast {
            "└── "
        } else {
            "├── "
        };

        let  	kindStr = if m._Kernel.IsNone() {
            "Container".to_string()
        } else if let Some( op) = m._Kernel.ToFastOp() {
            format!( "{:?}", op)
        } else {
            "Custom".to_string()
        };

        let  	portsStr = format!( "in:{}, out:{}", m._InPorts.Size(), m._OutPorts.Size());
        ostr.push_str( &format!( "{prefix}{marker}{} [{} ({})]
", m._Name, portsStr, kindStr));

        let  	childPrefix = if isRoot {
            String::new()
        } else if isLast {
            format!( "{prefix}    ")
        } else {
            format!( "{prefix}│   ")
        };

        let  	children = self.SubModules( modId);
        let  	childCount = children.len();
        for ( idx, &childId) in children.iter().enumerate() {
            let  	childIsLast = idx + 1 == childCount;
            self.DumpHierarchyNode( childId, &childPrefix, false, childIsLast, ostr);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Freeze( &mut self) -> Result< (), LayoutError>
    {
        // Step 1: Compact connection graph for CSR binary search / segments
        self._Connections.Compact();

        // Step 2: Sort modules for their KernelKind & vtable
        self.SortModules();

        // Step 3: Validate graph connections
        self.Validate()?;

        return Ok( ());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PartitionNets( &self) -> ( EdgeBroadcast, Buff< TriggerId>)
    {
        let  	portCountU32 = self._Ports.Size();
        let  	mut broadcast = EdgeBroadcast::New( portCountU32);

        self._Modules.Arr().Traverse( |module| {
            module._InPorts.Traverse( |idx| {
                let  	portId = PortId::In( idx);
                broadcast.DoBroadcast( portId.0, |elemId, _, _, nextStack| {
                    self._Connections.NodeTraverse( elemId, |nextElem| {
                        nextStack.Push( nextElem);
                    });
                });
            });
            module._OutPorts.Traverse( |idx| {
                let  	portId = PortId::Out( idx);
                broadcast.DoBroadcast( portId.0, |elemId, _, _, nextStack| {
                    self._Connections.NodeTraverse( elemId, |nextElem| {
                        nextStack.Push( nextElem);
                    });
                });
            });
        });

        let  	portToTrigger = broadcast.SnitchNodeGroupIds();
        return ( broadcast, portToTrigger);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	BuildTriggers( &self, broadcast: &EdgeBroadcast, portToTrigger: &Buff< TriggerId>) -> TriggerWad
    {
        let  	groupCount = broadcast.SzGroup();
        let  	mut pastVals = Stash::WithCapacity( groupCount);
        let  	mut currentVals = Stash::WithCapacity( groupCount);
        let  	mut futureVals = Stash::WithCapacity( groupCount);

        USeg::New( U32::_0, groupCount).Traverse( |grpIdx| {
            let  	firstPortId = PortId( broadcast.FirstId( grpIdx));
            let  	rootPort = &self._Ports[firstPortId.Index()];
            let  	defaultVal = Reg::DefaultTyped( rootPort._Type);

            pastVals.Push( defaultVal);
            currentVals.Push( defaultVal);
            futureVals.Push( defaultVal);
        });

        let  	mut subscribersLists = Buff::Create( groupCount, |_| Stash::New());

        self._Modules.Arr().Traverse( |module| {
            module._InPorts.Traverse( |idx| {
                let  	trigId = portToTrigger[idx];
                subscribersLists[ trigId].Push( module._Id.0);
            });
        });

        let  	mut subscriberSpans = Stash::WithCapacity( groupCount);
        let  	mut subscribers = Stash::New();

        subscribersLists.Arr().Traverse( |list| {
            let  	start = subscribers.Size();
            list.Arr().Traverse( |sub| {
                subscribers.Push( *sub);
            });
            let  	sz = subscribers.Size() - start;
            subscriberSpans.Push( USeg::New( start, sz));
        });

        return TriggerWad::New(
            pastVals.IntoBuff(),
            currentVals.IntoBuff(),
            futureVals.IntoBuff(),
            subscriberSpans.IntoBuff(),
            subscribers.IntoBuff(),
        );
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CompileWarps( &self, portToTrigger: &Buff< TriggerId>) -> ( Buff< FastWarp>, Buff< CustomWarp>)
    {
        let  	mut fastWarps = Stash::New();
        let  	mut customWarps = Stash::New();

        let  	modules = self._Modules.Slice();
        let  	mut i = 0;

        while i < modules.len() {
            let  	m = &modules[i];
            match m._Kernel {
                KernelKind::None => {
                    i += 1;
                    continue;
                }
                KernelKind::Custom( _) => {
                    let  	vtablePtr = m._Kernel.ClassKey().1;
                    let  	startIdx = i;
                    let  	mut instances = Stash::New();
                    let  	mut inTriggersList = Stash::New();
                    let  	mut outTriggersList = Stash::New();

                    while i < modules.len() && modules[i]._Kernel.ClassKey() == ( 1, vtablePtr) {
                        let  	curMod = &modules[i];
                        if let KernelKind::Custom( cb) = &curMod._Kernel {
                            instances.Push( Arc::clone( cb));
                        }

                        let  	mut inTrig = Stash::WithCapacity( curMod._InPorts.Size());
                        curMod._InPorts.Traverse( |idx| {
                            inTrig.Push( portToTrigger[idx]);
                        });
                        inTriggersList.Push( inTrig.IntoBuff());

                        let  	mut outTrig = Stash::WithCapacity( curMod._OutPorts.Size());
                        curMod._OutPorts.Traverse( |idx| {
                            outTrig.Push( portToTrigger[idx]);
                        });
                        outTriggersList.Push( outTrig.IntoBuff());

                        i += 1;
                    }

                    let  	count = ( i - startIdx) as u32;
                    customWarps.Push( CustomWarp::New(
                        vtablePtr,
                        U32( startIdx as u32),
                        U32( count),
                        instances.IntoBuff(),
                        inTriggersList.IntoBuff(),
                        outTriggersList.IntoBuff(),
                    ));
                }
                _ => {
                    let  	op = m._Kernel.ToFastOp().unwrap();
                    let  	outPortIdx0 = m._OutPorts.First();
                    let  	mask = self._Ports[outPortIdx0]._Type.Mask();
                    let  	startIdx = i;

                    let  	mut in1List = Stash::New();
                    let  	mut in2List = Stash::New();
                    let  	mut outList = Stash::New();

                    while i < modules.len() {
                        let  	curMod = &modules[i];
                        if let Some( curOp) = curMod._Kernel.ToFastOp() {
                            let  	curOutPort0 = curMod._OutPorts.First();
                            let  	curMask = self._Ports[curOutPort0]._Type.Mask();
                            if curOp == op && curMask == mask {
                                let  	in1 = portToTrigger[curMod._InPorts.First()];
                                let  	in2 = if curMod._InPorts.Size() > U32( 1) {
                                    portToTrigger[curMod._InPorts.First() + U32( 1)]
                                } else {
                                    in1
                                };
                                let  	outTrig = portToTrigger[curOutPort0];

                                in1List.Push( in1);
                                in2List.Push( in2);
                                outList.Push( outTrig);
                                i += 1;
                                continue;
                            }
                        }
                        break;
                    }

                    let  	count = ( i - startIdx) as u32;
                    fastWarps.Push( FastWarp::New(
                        op,
                        U32( startIdx as u32),
                        U32( count),
                        mask,
                        in1List.IntoBuff(),
                        in2List.IntoBuff(),
                        outList.IntoBuff(),
                    ));
                }
            }
        }

        return ( fastWarps.IntoBuff(), customWarps.IntoBuff());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CompileModules( &self, portToTrigger: &Buff< TriggerId>) -> ( Buff< FastModule>, Buff< CustomModule>)
    {
        let  	modCount = self._Modules.Size();
        let  	mut fastModules = Stash::WithCapacity( modCount);
        let  	mut customModules = Stash::New();

        self._Modules.Arr().Traverse( |module| {
            let  	mut inTriggers = Stash::WithCapacity( module._InPorts.Size());
            module._InPorts.Traverse( |idx| {
                inTriggers.Push( portToTrigger[idx]);
            });

            let  	mut outTriggers = Stash::WithCapacity( module._OutPorts.Size());
            module._OutPorts.Traverse( |idx| {
                outTriggers.Push( portToTrigger[idx]);
            });

            if module._Kernel.IsNone() {
                return;
            }
            if let Some( op) = module._Kernel.ToFastOp() {
                let  	in1 = inTriggers[U32( 0)];
                let  	in2 = if inTriggers.Size() > U32( 1) { inTriggers[U32( 1)] } else { in1 };
                let  	outTrig = outTriggers[U32( 0)];
                let  	outPortIdx = module._OutPorts.First();
                let  	outPortType = self._Ports[outPortIdx]._Type;
                fastModules.Push( FastModule::New( module._Id, in1, in2, outTrig, op, outPortType.Mask()));
            } else if let KernelKind::Custom( callback) = &module._Kernel {
                customModules.Push( CustomModule::New(
                    module._Id,
                    inTriggers.IntoBuff(),
                    outTriggers.IntoBuff(),
                    Arc::clone( callback),
                ));
            }
        });

        return ( fastModules.IntoBuff(), customModules.IntoBuff());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
