//-- layout.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    rube::{
        module::{ KernelKind, Module, ModuleId },
        port::{ PortDesc, PortDir, PortId, PortSensitivity, PortType },
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
    pub _Modules: Stash< Module>,
    pub _Ports: Stash< PortDesc>,
    pub _PortOwners: Stash< ModuleId>,
    pub _Connections: EdgeConnect,
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
            _Modules: Stash::New(),
            _Ports: Stash::New(),
            _PortOwners: Stash::New(),
            _Connections: EdgeConnect::New(),
        }
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
        F: Fn( &T) -> PortDesc,
    {
        let  	arr: Arr< 'a, T> = ports.into();
        let  	mut portIds = Stash::WithCapacity( arr.Size());
        arr.Traverse( |item| {
            let  	mut desc = extract( item);
            let  	rawIdx = self._Ports.Size();
            let  	portId = if dir == PortDir::Out {
                PortId::Out( rawIdx)
            } else {
                PortId::In( rawIdx)
            };
            desc._Name = format!( "{moduleName}.{}", desc._Name);
            self._Ports.Push( desc);
            self._PortOwners.Push( modId);
            portIds.Push( portId);
        });
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
        let  	inArr: Arr< 'a, PortDesc> = inPorts.into();
        let  	inSensitivities = Buff::Create( inArr.Size(), |i| inArr[i]._Sensitivity);
        let  	inPortIds = self.AddPorts( modId, name, inArr, PortDir::In, |d| d.clone());
        let  	outPortIds = self.AddPorts( modId, name, outPorts, PortDir::Out, |d| d.clone());
        let  	module = Module::New(
            modId,
            name,
            inPortIds,
            inSensitivities,
            outPortIds,
            kernel,
        );
        self._Modules.Push( module);
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
        let  	inPortIds = self.AddPorts( modId, name, inPorts, PortDir::In, |&name| PortDesc::Bool( name));
        let  	inSensitivities = Buff::Create( inPortIds.Size(), |_| PortSensitivity::Any);
        let  	outPortIds = self.AddPorts( modId, name, outPorts, PortDir::Out, |&name| PortDesc::Bool( name));
        let  	module = Module::New(
            modId,
            name,
            inPortIds,
            inSensitivities,
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
            module._InPorts.Arr().Traverse( |&inPort| {
                if err.is_some() {
                    return;
                }
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

            module._OutPorts.Arr().Traverse( |&outPort| {
                if err.is_some() {
                    return;
                }
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
    pub fn	PortOwners( &self) -> &[ModuleId]
    {
        return self._PortOwners.Slice();
    }

    #[inline]
    pub fn	PortOwner( &self, portId: PortId) -> Option< ModuleId>
    {
        let  	idx = portId.Index();
        if idx < self._PortOwners.Size() {
            return Some( self._PortOwners[idx]);
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

    pub fn	Freeze( &mut self) -> Result< (), LayoutError>
    {
        // Step 1: Compact connection graph for CSR binary search / segments
        self._Connections.Compact();

        // Step 2: Validate graph connections
        self.Validate()?;

        return Ok( ());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
