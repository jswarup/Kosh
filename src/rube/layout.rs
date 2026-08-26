//-- layout.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    rube::{
        compiler::NetCompiler,
        engine::SimEngine,
        module::{ KernelKind, ModuleDescriptor, ModuleId },
        port::{ Port, PortDesc, PortDir, PortId, PortType },
    },
    silo::{ Buff, U32 },
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

#[derive( Clone, Debug)]
pub struct Layout
{
    pub _Modules: Vec< ModuleDescriptor>,
    pub _Ports: Vec< Port>,
    pub _Connections: Vec<( PortId, PortId)>,
    pub _InDrivers: Vec< Option< PortId>>,
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
    pub fn	New() -> Self
    {
        return Self {
            _Modules: Vec::new(),
            _Ports: Vec::new(),
            _Connections: Vec::new(),
            _InDrivers: Vec::new(),
        };
    }

    pub fn	AddModule( &mut self, name: &str, inPorts: &[PortDesc], outPorts: &[PortDesc], kernel: KernelKind) -> ModuleId
    {
        let  	modIdx = self._Modules.len();
        let  	modId = ModuleId( U32( modIdx as u32));

        let  	mut inPortIds = Vec::new();
        for inDesc in inPorts {
            let  	portIdx = self._Ports.len();
            let  	portId = PortId( U32( portIdx as u32));
            let  	fullName = format!( "{name}.{}", inDesc._Name);
            let  	port = Port::New( portId, modId.0, &fullName, PortDir::In, inDesc._Type);
            self._Ports.push( port);
            self._InDrivers.push( None);
            inPortIds.push( portId);
        }

        let  	mut outPortIds = Vec::new();
        for outDesc in outPorts {
            let  	portIdx = self._Ports.len();
            let  	portId = PortId( U32( portIdx as u32));
            let  	fullName = format!( "{name}.{}", outDesc._Name);
            let  	port = Port::New( portId, modId.0, &fullName, PortDir::Out, outDesc._Type);
            self._Ports.push( port);
            self._InDrivers.push( None);
            outPortIds.push( portId);
        }

        let  	module = ModuleDescriptor::New(
            modId,
            name,
            Buff::from( inPortIds.as_slice()),
            Buff::from( outPortIds.as_slice()),
            kernel,
        );
        self._Modules.push( module);
        return modId;
    }

    pub fn	AddModuleSimple( &mut self, name: &str, inPorts: &[&str], outPorts: &[&str], kernel: KernelKind) -> ModuleId
    {
        let  	inDescs: Vec< PortDesc> = inPorts.iter().map( |&n| PortDesc::Bool( n)).collect();
        let  	outDescs: Vec< PortDesc> = outPorts.iter().map( |&n| PortDesc::Bool( n)).collect();
        return self.AddModule( name, &inDescs, &outDescs, kernel);
    }

    pub fn	InPort( &self, moduleId: ModuleId, portIdx: usize) -> Option< PortId>
    {
        let  	idx = usize::from( moduleId.0);
        if idx >= self._Modules.len() {
            return None;
        }
        let  	module = &self._Modules[idx];
        if portIdx >= module._InPorts.len() {
            return None;
        }
        return Some( module._InPorts[portIdx]);
    }

    pub fn	OutPort( &self, moduleId: ModuleId, portIdx: usize) -> Option< PortId>
    {
        let  	idx = usize::from( moduleId.0);
        if idx >= self._Modules.len() {
            return None;
        }
        let  	module = &self._Modules[idx];
        if portIdx >= module._OutPorts.len() {
            return None;
        }
        return Some( module._OutPorts[portIdx]);
    }

    pub fn	Connect( &mut self, srcOut: PortId, dstIn: PortId) -> Result< (), LayoutError>
    {
        let  	srcIdx = usize::from( srcOut.0);
        let  	dstIdx = usize::from( dstIn.0);

        if srcIdx >= self._Ports.len() {
            return Err( LayoutError::PortNotFound( srcOut));
        }
        if dstIdx >= self._Ports.len() {
            return Err( LayoutError::PortNotFound( dstIn));
        }

        let  	srcPort = &self._Ports[srcIdx];
        if srcPort._Dir != PortDir::Out {
            return Err( LayoutError::InvalidPortDirection {
                _Port: srcOut,
                _Expected: PortDir::Out,
                _Actual: srcPort._Dir,
            });
        }

        let  	dstPort = &self._Ports[dstIdx];
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
        if let Some( existingSrc) = self._InDrivers[dstIdx] {
            return Err( LayoutError::DuplicateInputDriver {
                _DstIn: dstIn,
                _ExistingSrc: existingSrc,
                _AttemptedSrc: srcOut,
            });
        }

        self._InDrivers[dstIdx] = Some( srcOut);
        self._Connections.push(( srcOut, dstIn));
        return Ok( ());
    }

    #[inline]
    pub fn	Modules( &self) -> &[ModuleDescriptor]
    {
        return &self._Modules;
    }

    #[inline]
    pub fn	Ports( &self) -> &[Port]
    {
        return &self._Ports;
    }

    #[inline]
    pub fn	Connections( &self) -> &[( PortId, PortId)]
    {
        return &self._Connections;
    }

    pub fn	Compile( &self) -> Result< SimEngine, LayoutError>
    {
        return NetCompiler::New().Compile( self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
