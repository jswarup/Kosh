//-- module.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::sync::Arc;
use	crate::{
    rube::{ port::PortId, regval::RegVal },
    silo::{ Buff, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ModuleId( pub U32);

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KernelOp
{
    Nand,
    And,
    Or,
    Not,
    Xor,
    Nor,
    Xnor,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub enum KernelKind
{
    Nand,
    And,
    Or,
    Not,
    Xor,
    Nor,
    Xnor,
    Custom( Arc< dyn Fn( &[RegVal], &mut [RegVal]) + Send + Sync>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for KernelKind
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::Nand => write!( f, "KernelKind::Nand"),
            Self::And => write!( f, "KernelKind::And"),
            Self::Or => write!( f, "KernelKind::Or"),
            Self::Not => write!( f, "KernelKind::Not"),
            Self::Xor => write!( f, "KernelKind::Xor"),
            Self::Nor => write!( f, "KernelKind::Nor"),
            Self::Xnor => write!( f, "KernelKind::Xnor"),
            Self::Custom( _) => write!( f, "KernelKind::Custom(...)"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct ModuleDescriptor
{
    pub _Id: ModuleId,
    pub _Name: String,
    pub _InPorts: Buff< PortId>,
    pub _OutPorts: Buff< PortId>,
    pub _Kernel: KernelKind,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ModuleDescriptor
{
    pub fn	New( id: ModuleId, name: &str, inPorts: Buff< PortId>, outPorts: Buff< PortId>, kernel: KernelKind) -> Self
    {
        return Self {
            _Id: id,
            _Name: name.to_string(),
            _InPorts: inPorts,
            _OutPorts: outPorts,
            _Kernel: kernel,
        };
    }

    #[inline]
    pub fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    #[inline]
    pub fn	InPorts( &self) -> &Buff< PortId>
    {
        return &self._InPorts;
    }

    #[inline]
    pub fn	OutPorts( &self) -> &Buff< PortId>
    {
        return &self._OutPorts;
    }

    #[inline]
    pub fn	Kernel( &self) -> &KernelKind
    {
        return &self._Kernel;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
