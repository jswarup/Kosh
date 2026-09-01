//-- port.rs -------------------------------------------------------------------------------------------------------------------------

use	crate::silo::U32;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PortId( pub U32);

impl PortId
{
    pub const DIR_BIT: u32 = 31;

    #[inline]
    pub const fn	In( index: U32) -> Self
    {
        return Self( index.SetBit( Self::DIR_BIT, false));
    }

    #[inline]
    pub const fn	Out( index: U32) -> Self
    {
        return Self( index.SetBit( Self::DIR_BIT, true));
    }

    #[inline]
    pub const fn	IsOut( self) -> bool
    {
        return self.0.GetBit( Self::DIR_BIT);
    }

    #[inline]
    pub const fn	IsIn( self) -> bool
    {
        return !self.IsOut();
    }

    #[inline]
    pub const fn	Index( self) -> U32
    {
        return self.0.Grab( 0, Self::DIR_BIT);
    }

    #[inline]
    pub const fn	Dir( self) -> PortDir
    {
        if self.IsOut() { PortDir::Out } else { PortDir::In }
    }
}

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
    pub const fn	TypeSize( &self) -> U32
    {
        match self {
            PortType::Bool => U32( 1),
            PortType::U8Val => U32( 1),
            PortType::U16Val => U32( 1),
            PortType::U32Val => U32( 1),
            PortType::U64Val => U32( 2),
            PortType::Custom( bits) => U32( ( bits.0 + 31) / 32),
        }
    }

    pub const fn	Bits( &self) -> u32
    {
        match self {
            PortType::Bool => 1,
            PortType::U8Val => 8,
            PortType::U16Val => 16,
            PortType::U32Val => 32,
            PortType::U64Val => 64,
            PortType::Custom( bits) => bits.0,
        }
    }

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

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PortSensitivity
{
    Up,
    Down,
    Any,
    None,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, PartialEq, Eq, Hash, Debug)]
pub struct PortDesc
{
    pub _Name: String,
    pub _Type: PortType,
    pub _Sensitivity: PortSensitivity,
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
            _Sensitivity: PortSensitivity::Any,
        };
    }

    #[inline]
    pub fn	Sensitive( mut self, sensitivity: PortSensitivity) -> Self
    {
        self._Sensitivity = sensitivity;
        return self;
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

