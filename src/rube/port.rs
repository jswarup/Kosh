//-- port.rs -------------------------------------------------------------------------------------------------------------------------

use	crate::{
    rube::trigger::TriggerId,
    silo::{ EdgeBroadcast, EdgeConnect, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PortId( pub U32);

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PortDir
{
    In,
    Out,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PortType
{
    Bool,
    U8Val,
    U16Val,
    U32Val,
    U64Val,
    Custom( U32),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PortType
{
    #[inline]
    pub const fn	Mask( &self) -> u64
    {
        return match self {
            Self::Bool => 1,
            Self::U8Val => 0xFF,
            Self::U16Val => 0xFFFF,
            Self::U32Val => 0xFFFF_FFFF,
            Self::U64Val => 0xFFFF_FFFF_FFFF_FFFF,
            Self::Custom( bits) => {
                let  	b = bits.0;
                if b >= 64 {
                    0xFFFF_FFFF_FFFF_FFFF
                } else if b == 0 {
                    0
                } else {
                    ( 1u64 << b) - 1
                }
            }
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, PartialEq, Eq, Hash, Debug)]
pub struct PortDesc
{
    pub _Name: String,
    pub _Type: PortType,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PortDesc
{
    #[inline]
    pub fn	New( name: impl Into< String>, portType: PortType) -> Self
    {
        return Self {
            _Name: name.into(),
            _Type: portType,
        };
    }

    #[inline]
    pub fn	Bool( name: impl Into< String>) -> Self
    {
        return Self::New( name, PortType::Bool);
    }

    #[inline]
    pub fn	U32( name: impl Into< String>) -> Self
    {
        return Self::New( name, PortType::U32Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> From< &'a str> for PortDesc
{
    #[inline]
    fn	from( name: &'a str) -> Self
    {
        return Self::Bool( name);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Port
{
    pub _Id: PortId,
    pub _ModuleId: U32,
    pub _Name: String,
    pub _Dir: PortDir,
    pub _PortType: PortType,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Port
{
    #[inline]
    pub fn	New( id: PortId, moduleId: U32, name: &str, dir: PortDir, portType: PortType) -> Self
    {
        return Self {
            _Id: id,
            _ModuleId: moduleId,
            _Name: name.to_string(),
            _Dir: dir,
            _PortType: portType,
        };
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub struct TopologyPort
{
    pub _Name: String,
    pub _Trigger: TriggerId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TopologyPort
{
    #[inline]
    pub fn	New( name: impl Into< String>, trigger: TriggerId) -> Self
    {
        return Self {
            _Name: name.into(),
            _Trigger: trigger,
        };
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    #[inline]
    pub fn	Trigger( &self) -> TriggerId
    {
        return self._Trigger;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct PortLayout
{
    pub _PortConn: EdgeConnect,
    pub _PortCast: EdgeBroadcast,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PortLayout
{
    #[inline]
    pub fn	New( mxVert: U32) -> Self
    {
        return Self {
            _PortConn: EdgeConnect::New(),
            _PortCast: EdgeBroadcast::New( mxVert),
        };
    }

    #[inline]
    pub fn	PortConn( &self) -> &EdgeConnect
    {
        return &self._PortConn;
    }

    #[inline]
    pub fn	PortCast( &self) -> &EdgeBroadcast
    {
        return &self._PortCast;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
