//-- port.rs -------------------------------------------------------------------------------------------------------------------------

use	crate::silo::U32;

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
    pub fn	U8( name: impl Into< String>) -> Self
    {
        return Self::New( name, PortType::U8Val);
    }

    #[inline]
    pub fn	U16( name: impl Into< String>) -> Self
    {
        return Self::New( name, PortType::U16Val);
    }

    #[inline]
    pub fn	U32( name: impl Into< String>) -> Self
    {
        return Self::New( name, PortType::U32Val);
    }

    #[inline]
    pub fn	U64( name: impl Into< String>) -> Self
    {
        return Self::New( name, PortType::U64Val);
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    #[inline]
    pub fn	PortType( &self) -> PortType
    {
        return self._Type;
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
    pub fn	Id( &self) -> PortId
    {
        return self._Id;
    }

    #[inline]
    pub fn	ModuleId( &self) -> U32
    {
        return self._ModuleId;
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    #[inline]
    pub fn	Dir( &self) -> PortDir
    {
        return self._Dir;
    }

    #[inline]
    pub fn	PortType( &self) -> PortType
    {
        return self._PortType;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
