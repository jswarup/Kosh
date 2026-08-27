//-- layout.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    rube::{
        compiler::NetCompiler,
        engine::SimEngine,
        module::{ KernelKind, ModuleDescriptor, ModuleId },
        port::{ Port, PortDesc, PortDir, PortId, PortType },
    },
    silo::{ Arr, Buff, IAccess, Stash, U32 },
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
            Self::DuplicateInputDriver { _DstIn, _ExistingSrc, _AttemptedSrc } => {
                write!( f, "Input port {:?} already driven by {:?}; cannot connect {:?}", _DstIn, _ExistingSrc, _AttemptedSrc)
            }
            Self::InvalidPortDirection { _Port, _Expected, _Actual } => {
                write!( f, "Port {:?} has direction {:?}, expected {:?}", _Port, _Actual, _Expected)
            }
            Self::PortNotFound( id) => write!( f, "Port {:?} not found", id),
            Self::ModuleNotFound( id) => write!( f, "Module {:?} not found", id),
            Self::UnconnectedInput { _ModuleId, _PortId } => {
                write!( f, "Input port {:?} on module {:?} is unconnected", _PortId, _ModuleId)
            }
            Self::TypeMismatch { _Src, _SrcType, _Dst, _DstType } => {
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
    pub _Modules: Stash< ModuleDescriptor>,
    pub _Ports: Stash< Port>,
    pub _Connections: Stash<( PortId, PortId)>,
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
            _Connections: Stash::New(),
            _InDrivers: Stash::New(),
        };
    }

    fn	AddPorts< 'a, T, F>(
        &mut self,
        modId: ModuleId,
        moduleName: &str,
        ports: Arr< 'a, T>,
        dir: PortDir,
        extract: F,
    ) -> Buff< PortId>
    where
        F: Fn( &'a T) -> ( &str, PortType),
    {
        let  	mut portIds = Stash::WithCapacity( ports.Size());
        for item in ports {
            let  	( portName, portType) = extract( item);
            let  	portIdx = self._Ports.Size().AsUsize();
            let  	portId = PortId( U32( portIdx as u32));
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
        let  	inPortIds = self.AddPorts( modId, name, inPorts.into(), PortDir::In, |d| ( &d._Name, d._Type));
        let  	outPortIds = self.AddPorts( modId, name, outPorts.into(), PortDir::Out, |d| ( &d._Name, d._Type));
        let  	module = ModuleDescriptor::New(
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
        let  	inPortIds = self.AddPorts( modId, name, inPorts.into(), PortDir::In, |&name| ( name, PortType::Bool));
        let  	outPortIds = self.AddPorts( modId, name, outPorts.into(), PortDir::Out, |&name| ( name, PortType::Bool));
        let  	module = ModuleDescriptor::New(
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
    pub fn	InPort( &self, moduleId: ModuleId, portIdx: usize) -> Option< PortId>
    {
        let  	idx = usize::from( moduleId.0);
        if idx >= self._Modules.Size().AsUsize() {
            return None;
        }
        let  	module = &self._Modules.Slice()[idx];
        if portIdx >= module._InPorts.len() {
            return None;
        }
        return Some( module._InPorts[portIdx]);
    }

    #[inline]
    pub fn	OutPort( &self, moduleId: ModuleId, portIdx: usize) -> Option< PortId>
    {
        let  	idx = usize::from( moduleId.0);
        if idx >= self._Modules.Size().AsUsize() {
            return None;
        }
        let  	module = &self._Modules.Slice()[idx];
        if portIdx >= module._OutPorts.len() {
            return None;
        }
        return Some( module._OutPorts[portIdx]);
    }

    pub fn	Connect( &mut self, srcOut: PortId, dstIn: PortId) -> Result< (), LayoutError>
    {
        let  	srcIdx = usize::from( srcOut.0);
        let  	dstIdx = usize::from( dstIn.0);
        let  	portCount = self._Ports.Size().AsUsize();

        if srcIdx >= portCount {
            return Err( LayoutError::PortNotFound( srcOut));
        }
        if dstIdx >= portCount {
            return Err( LayoutError::PortNotFound( dstIn));
        }

        let  	srcPort = &self._Ports.Slice()[srcIdx];
        if srcPort._Dir != PortDir::Out {
            return Err( LayoutError::InvalidPortDirection {
                _Port: srcOut,
                _Expected: PortDir::Out,
                _Actual: srcPort._Dir,
            });
        }

        let  	dstPort = &self._Ports.Slice()[dstIdx];
        if dstPort._Dir != PortDir::In {
            return Err( LayoutError::InvalidPortDirection {
                _Port: dstIn,
                _Expected: PortDir::In,
                _Actual: dstPort._Dir,
            });
        }

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
        if let Some( existingSrc) = self._InDrivers.Slice()[dstIdx] {
            return Err( LayoutError::DuplicateInputDriver {
                _DstIn: dstIn,
                _ExistingSrc: existingSrc,
                _AttemptedSrc: srcOut,
            });
        }

        self._InDrivers.SliceMut()[dstIdx] = Some( srcOut);
        self._Connections.Push(( srcOut, dstIn));
        return Ok( ());
    }

    #[inline]
    pub fn	Modules( &self) -> &[ModuleDescriptor]
    {
        return self._Modules.Slice();
    }

    #[inline]
    pub fn	Ports( &self) -> &[Port]
    {
        return self._Ports.Slice();
    }

    #[inline]
    pub fn	Connections( &self) -> &[( PortId, PortId)]
    {
        return self._Connections.Slice();
    }

    #[inline]
    pub fn	Compile( &self) -> Result< SimEngine, LayoutError>
    {
        return NetCompiler::New().Compile( self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
