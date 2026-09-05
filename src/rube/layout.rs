//-- layout.rs -----------------------------------------------------------------------------------------------------------------------

use	std::{
    cell::RefCell,
    fmt,
    sync::Arc,
};
use	crate::{

    rube::{
        coro_kernel::{ CoroInstance, CoroWarp },
        module::{ BehavioralWarp, CustomModule, CustomWarp, FastModule, FastWarp, IModule, KernelKind, Module, ModuleId },
        netlist::{ INetlist, Netlist },
        port::{ IPort, PortDesc, PortDir, PortId, PortType },
        reg::Reg,
        trigger::{ TriggerId, TriggerWad },
    },
    silo::{ Arr, Buff, IAccess, IArr, Stash, U32, USeg },
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
    InvalidHierarchyConnection {
        _Src:       PortId,
        _Dst:       PortId,
        _SrcOwner:  ModuleId,
        _DstOwner:  ModuleId,
        _Reason:    &'static str,
    },
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
            LayoutError::InvalidHierarchyConnection { _Src, _Dst, _SrcOwner, _DstOwner, _Reason } => {
                write!( f, "Invalid hierarchy connection between port {:?} of module {:?} and port {:?} of module {:?}: {}",
                    _Src, _SrcOwner, _Dst, _DstOwner, _Reason)
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
    pub _Netlist:         Netlist,
    pub _ModuleChildren:  Stash< Stash< ModuleId>>,
    pub _SubModules:      Stash< ModuleId>,
    pub _Descendents:     Stash< ModuleId>,
    pub _PortToTrigger:   Buff< TriggerId>,
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
            _Netlist:         Netlist::New(),
            _ModuleChildren:  Stash::New(),
            _SubModules:      Stash::New(),
            _Descendents:     Stash::New(),
            _PortToTrigger:   Buff::New(),
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
        self._Netlist.Grow( count);
        return USeg::New( start, count);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddModule< 'a, I, O>(
        &mut self,
        name: &str,
        parent: Option< ModuleId>,
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
            parent,
            name,
            inSeg,
            outSeg,
            kernel,
        );
        self._Modules.Push( module);
        self._ModuleChildren.Push( Stash::New());
        if let Some( p) = parent {
            assert!( p.0 < self._Modules.Size(), "Parent ModuleId out of bounds");
            assert!( modId.0 < self._Modules.Size(), "Child ModuleId out of bounds");
            self._ModuleChildren[p.0].Push( modId);
        }

        return modId;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AddCoroModule< 'a, I, O, F>(
        &mut self,
        name: &str,
        parent: Option< ModuleId>,
        inPorts: I,
        outPorts: O,
        factory: F,
    ) -> ModuleId
    where
        I: Into< Arr< 'a, PortDesc>>,
        O: Into< Arr< 'a, PortDesc>>,
        F: Fn() -> CoroInstance + Send + Sync + 'static,
    {
        return self.AddModule( name, parent, inPorts, outPorts, KernelKind::Coro( Arc::new( factory)) );
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

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	Connect( &mut self, src: impl IPort, dst: impl IPort) -> &mut Self
    {
        return self.ConnectPorts( src.Id(), dst.Id());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ConnectPorts( &mut self, src: PortId, dst: PortId) -> &mut Self
    {
        let  	srcIdx = src.Index();
        let  	dstIdx = dst.Index();
        assert!( srcIdx < self._Ports.Size(), "Source port out of bounds");
        assert!( dstIdx < self._Ports.Size(), "Destination port out of bounds");

        let  	srcOwner = self._Ports[srcIdx]._Owner;
        let  	dstOwner = self._Ports[dstIdx]._Owner;
        let  	srcParent = self._Modules[srcOwner.0]._Parent;
        let  	dstParent = self._Modules[dstOwner.0]._Parent;

        // 4 valid hierarchical scope cases
        let  	( driver, sink) = if srcParent == dstParent {
            // Sibling-to-Sibling (inside shared parent)
            assert!( src.IsOut(), "In sibling connection, source must be an output port");
            assert!( dst.IsIn(), "In sibling connection, destination must be an input port");
            ( src, dst)
        } else if Some( srcOwner) == dstParent {
            // Pass-Down: parent input driving child input
            assert!( src.IsIn(), "In pass-down connection, parent port must be an input");
            assert!( dst.IsIn(), "In pass-down connection, child port must be an input");
            ( src, dst)
        } else if Some( dstOwner) == srcParent {
            // Pass-Up: child output driving parent output
            assert!( src.IsOut(), "In pass-up connection, child port must be an output");
            assert!( dst.IsOut(), "In pass-up connection, parent port must be an output");
            ( src, dst)
        } else if srcOwner == dstOwner {
            // Feedthrough: parent input connected directly to parent output
            assert!( src.IsIn(), "In feedthrough connection, source must be an input");
            assert!( dst.IsOut(), "In feedthrough connection, destination must be an output");
            ( src, dst)
        } else {
            panic!(
                "Cannot connect port {:?} of module {:?} to port {:?} of module {:?}: port is not visible beyond its immediate parent",
                src, srcOwner, dst, dstOwner
            );
        };

        let  	srcType = self._Ports[srcIdx]._Type;
        let  	dstType = self._Ports[dstIdx]._Type;
        assert_eq!( srcType, dstType, "Type mismatch connecting {:?} ({:?}) to {:?} ({:?})", src, srcType, dst, dstType);

        self._Netlist.Connect( driver, sink).expect( "Connection validation failed");

        return self;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub fn	Seal( &mut self, module: &impl IModule)
    {
        self.SealModule( module.Id());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SealModule( &mut self, moduleId: ModuleId)
    {
        let  	modIdx = moduleId.0;
        assert!( modIdx < self._Modules.Size(), "ModuleId out of bounds");
        assert!( !self._Modules[modIdx]._IsSealed, "Module {:?} is already sealed", moduleId);

        // 1. Gather all root IDs for boundary ports of this module and sort for binary search
        let  	mut boundaryRoots = Stash::New();
        self._Modules[modIdx]._InPorts.Traverse( |idx| {
            boundaryRoots.Push( self._Netlist.FindRoot( PortId::In( idx)));
        });
        self._Modules[modIdx]._OutPorts.Traverse( |idx| {
            boundaryRoots.Push( self._Netlist.FindRoot( PortId::Out( idx)));
        });
        let  	arr = boundaryRoots.Arr();
        let  	lessFn = move |i, j| arr.At( i) < arr.At( j);
        let  	swapFn = move |i, j| arr.Swap( i, j);
        arr.USeg().QSort( lessFn, swapFn);

        // 2. Traverse all direct children of this module
        let  	childCount = self._ModuleChildren[modIdx].Size();
        USeg::New( 0, childCount).Traverse( |cIdx| {
            let  	childId = self._ModuleChildren[modIdx][cIdx];
            let  	child = &self._Modules[childId.0];
            assert!( child._IsSealed, "Child module {:?} must be sealed before parent {:?}", childId, moduleId);

            child._InPorts.Traverse( |idx| {
                let  	portId = PortId::In( idx);
                let  	root = self._Netlist.FindRoot( portId);
                let  	isBoundary = arr.BinarySearch( &root).is_ok();
                if !isBoundary && !self._Netlist.HasTrigger( portId) {
                    let  	portType = self._Ports[idx]._Type;
                    self._Netlist.AssignTrigger( root, portType);
                }
            });
            child._OutPorts.Traverse( |idx| {
                let  	portId = PortId::Out( idx);
                let  	root = self._Netlist.FindRoot( portId);
                let  	isBoundary = arr.BinarySearch( &root).is_ok();
                if !isBoundary && !self._Netlist.HasTrigger( portId) {
                    let  	portType = self._Ports[idx]._Type;
                    self._Netlist.AssignTrigger( root, portType);
                }
            });
        });

        // 3. If top-level module (parent is None), seal its own boundary ports too
        if self._Modules[modIdx]._Parent.is_none() {
            self._Modules[modIdx]._InPorts.Traverse( |idx| {
                let  	portId = PortId::In( idx);
                let  	root = self._Netlist.FindRoot( portId);
                if !self._Netlist.HasTrigger( portId) {
                    let  	portType = self._Ports[idx]._Type;
                    self._Netlist.AssignTrigger( root, portType);
                }
            });
            self._Modules[modIdx]._OutPorts.Traverse( |idx| {
                let  	portId = PortId::Out( idx);
                let  	root = self._Netlist.FindRoot( portId);
                if !self._Netlist.HasTrigger( portId) {
                    let  	portType = self._Ports[idx]._Type;
                    self._Netlist.AssignTrigger( root, portType);
                }
            });
        }

        self._Modules[modIdx]._IsSealed = true;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

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
    pub fn	Netlist( &self) -> &Netlist
    {
        return &self._Netlist;
    }

    #[inline]
    pub fn	NetlistMut( &mut self) -> &mut Netlist
    {
        return &mut self._Netlist;
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
        let  	count = roots.Size();
        USeg::New( U32::_0, count).Traverse( |i| {
            let  	isLast = i + U32::_1 == count;
            self.DumpHierarchyNode( roots[i], "", true, isLast, ostr);
        });
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
        ostr.push_str( &format!( "{prefix}{marker}{} [{} ({})]\n", m._Name, portsStr, kindStr));

        let  	childPrefix = if isRoot {
            String::new()
        } else if isLast {
            format!( "{prefix}    ")
        } else {
            format!( "{prefix}│   ")
        };

        let  	children = self.SubModules( modId);
        let  	childCount = U32( children.len() as u32);
        USeg::New( U32::_0, childCount).Traverse( |i| {
            let  	childIsLast = i + U32::_1 == childCount;
            self.DumpHierarchyNode( children[i.AsUsize()], &childPrefix, false, childIsLast, ostr);
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Freeze( &mut self) -> Result< (), LayoutError>
    {
        // Seal any modules that have not yet been sealed (from leaves to root)
        let  	modCount = self._Modules.Size();
        USeg::New( U32::_0, modCount).Traverse( |step| {
            let  	modId = ModuleId( modCount - U32::_1 - step);
            if !self._Modules[modId.0]._IsSealed {
                self.SealModule( modId);
            }
        });

        // Precompute and store the port to trigger mapping
        self._PortToTrigger = self._Netlist.BuildPortToTrigger();

        // Sort modules for their KernelKind & vtable
        self.SortModules();

        return Ok( ());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	PortToTrigger( &self) -> Buff< TriggerId>
    {
        if !self._PortToTrigger.is_empty() {
            return self._PortToTrigger.clone();
        }
        return self._Netlist.BuildPortToTriggerConst();
    }

    #[inline]
    pub fn	PortTriggersOf( &self, seg: USeg, portToTrigger: &Buff< TriggerId>) -> Buff< TriggerId>
    {
        let  	mut stash = Stash::WithCapacity( seg.Size());
        seg.Traverse( |idx| {
            stash.Push( portToTrigger[idx]);
        });
        return stash.IntoBuff();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	BuildTriggers( &self, portToTrigger: &Buff< TriggerId>) -> TriggerWad
    {
        let  	groupCount = self._Netlist.TriggerCount();
        let  	mut pastVals = Stash::WithCapacity( groupCount);
        let  	mut currentVals = Stash::WithCapacity( groupCount);
        let  	mut futureVals = Stash::WithCapacity( groupCount);

        USeg::New( U32::_0, groupCount).Traverse( |grpIdx| {
            let  	portType = self._Netlist.TriggerType( grpIdx);
            let  	defaultVal = Reg::DefaultTyped( portType);

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

    pub fn	CompileWarps( &self, portToTrigger: &Buff< TriggerId>) -> ( Buff< FastWarp>, Buff< CustomWarp>, Buff< BehavioralWarp>, Buff< CoroWarp>)
    {
        let  	mut fastWarps = Stash::New();
        let  	mut customWarps = Stash::New();
        let  	mut behavioralWarps = Stash::New();
        let  	mut coroWarps = Stash::New();

        let  	modules = self._Modules.Slice();
        let  	mut i = 0;

        while i < modules.len() {
            let  	m = &modules[i];
            match m._Kernel {
                KernelKind::None => {
                    i += 1;
                    continue;
                }
                KernelKind::Coro( _) => {
                    let  	vtablePtr = m._Kernel.ClassKey().1;
                    let  	startIdx = i;
                    let  	mut instances = Stash::New();
                    let  	mut inTriggersList = Stash::New();
                    let  	mut outTriggersList = Stash::New();

                    while i < modules.len() && modules[i]._Kernel.ClassKey() == ( 4, vtablePtr) {
                        let  	curMod = &modules[i];
                        if let KernelKind::Coro( factory) = &curMod._Kernel {
                            instances.Push( RefCell::new( ( factory)() ));
                        }

                        inTriggersList.Push( self.PortTriggersOf( curMod._InPorts, portToTrigger));
                        outTriggersList.Push( self.PortTriggersOf( curMod._OutPorts, portToTrigger));
                        i += 1;
                    }

                    let  	count = ( i - startIdx) as u32;
                    coroWarps.Push( CoroWarp::New(
                        U32( startIdx as u32),
                        U32( count),
                        instances.IntoBuff(),
                        inTriggersList.IntoBuff(),
                        outTriggersList.IntoBuff(),
                    ));
                }
                KernelKind::Behavioral( _) => {
                    let  	vtablePtr = m._Kernel.ClassKey().1;
                    let  	startIdx = i;
                    let  	mut instances = Stash::New();
                    let  	mut inTriggersList = Stash::New();
                    let  	mut outTriggersList = Stash::New();

                    while i < modules.len() && modules[i]._Kernel.ClassKey() == ( 2, vtablePtr) {
                        let  	curMod = &modules[i];
                        if let KernelKind::Behavioral( cb) = &curMod._Kernel {
                            instances.Push( std::sync::Arc::clone( cb));
                        }

                        inTriggersList.Push( self.PortTriggersOf( curMod._InPorts, portToTrigger));
                        outTriggersList.Push( self.PortTriggersOf( curMod._OutPorts, portToTrigger));
                        i += 1;
                    }

                    let  	count = ( i - startIdx) as u32;
                    behavioralWarps.Push( BehavioralWarp::New(
                        U32( startIdx as u32),
                        U32( count),
                        instances.IntoBuff(),
                        inTriggersList.IntoBuff(),
                        outTriggersList.IntoBuff(),
                    ));
                }
                KernelKind::Custom( kernelName) => {
                    let  	startIdx = i;
                    let  	mut inTriggersList = Stash::New();
                    let  	mut outTriggersList = Stash::New();

                    while i < modules.len() {
                        let  	curMod = &modules[i];
                        if let KernelKind::Custom( curName) = curMod._Kernel {
                            if curName == kernelName {
                                inTriggersList.Push( self.PortTriggersOf( curMod._InPorts, portToTrigger));
                                outTriggersList.Push( self.PortTriggersOf( curMod._OutPorts, portToTrigger));
                                i += 1;
                                continue;
                            }
                        }
                        break;
                    }

                    let  	count = ( i - startIdx) as u32;
                    customWarps.Push( CustomWarp::New(
                        kernelName,
                        U32( startIdx as u32),
                        U32( count),
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

        return ( fastWarps.IntoBuff(), customWarps.IntoBuff(), behavioralWarps.IntoBuff(), coroWarps.IntoBuff());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	CompileModules( &self, portToTrigger: &Buff< TriggerId>) -> ( Buff< FastModule>, Buff< CustomModule>)
    {
        let  	modCount = self._Modules.Size();
        let  	mut fastModules = Stash::WithCapacity( modCount);
        let  	mut customModules = Stash::New();

        self._Modules.Arr().Traverse( |module| {
            let  	inTriggers = self.PortTriggersOf( module._InPorts, portToTrigger);
            let  	outTriggers = self.PortTriggersOf( module._OutPorts, portToTrigger);

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
            } else if let KernelKind::Behavioral( _cb) = &module._Kernel { } else if let KernelKind::Coro( _f) = &module._Kernel { } else if let KernelKind::Custom( kernelName) = &module._Kernel {
                customModules.Push( CustomModule::New(
                    module._Id,
                    inTriggers,
                    outTriggers,
                    kernelName,
                ));
            }
        });

        return ( fastModules.IntoBuff(), customModules.IntoBuff());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxSource!( Layout, _Modules, _Ports, _Netlist, _ModuleChildren, _SubModules, _Descendents, _PortToTrigger);
