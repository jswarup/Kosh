//-- port.rs -------------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::{ FieldExp, IFluxExportSource },
    rube::module::ModuleId,
    silo::{ U32, U64 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
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

pub trait IPort: Copy
{
    fn	Id( &self) -> PortId;

    #[inline]
    fn	Index( &self) -> U32
    {
        return self.Id().Index();
    }

    #[inline]
    fn	Dir( &self) -> PortDir
    {
        return self.Id().Dir();
    }

    #[inline]
    fn	IsOut( &self) -> bool
    {
        return self.Id().IsOut();
    }

    #[inline]
    fn	IsIn( &self) -> bool
    {
        return self.Id().IsIn();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IPort for PortId
{
    #[inline]
    fn	Id( &self) -> PortId
    {
        return *self;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum PortType
{
    #[default]
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

#[derive( Clone, PartialEq, Eq, Hash, Debug)]
pub struct PortDesc
{
    pub _Name:   String,
    pub _Type:   PortType,
    pub _Owner:  ModuleId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PortDesc
{
    #[inline]
    pub fn	New( name: impl Into< String>, portType: PortType) -> Self
    {
        return Self {
            _Name:   name.into(),
            _Type:   portType,
            _Owner:  ModuleId( U32::_X),
        };
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
    pub fn	Owner( &self) -> ModuleId
    {
        return self._Owner;
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

impl IFluxExportSource for PortId
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        *field = FieldExp::U64( U64( self.0.0 as u64));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for PortDir
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        let  	s = match self {
            Self::In => "In",
            Self::Out => "Out",
        };
        *field = FieldExp::Str( s);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for PortType
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        match self {
            Self::Bool => *field = FieldExp::Str( "Bool"),
            Self::U8Val => *field = FieldExp::Str( "U8Val"),
            Self::U16Val => *field = FieldExp::Str( "U16Val"),
            Self::U32Val => *field = FieldExp::Str( "U32Val"),
            Self::U64Val => *field = FieldExp::Str( "U64Val"),
            Self::Custom( bits) => *field = FieldExp::String( format!( "Custom:{}", bits.0)),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxSource!( PortDesc, _Name, _Type, _Owner);

impl crate::flux::IFluxImportSink for PortId {
    fn FromFieldImp( &mut self, field: crate::flux::FieldImp) -> bool {
        self.0.FromFieldImp( field)
    }
}
impl crate::flux::IFluxImportSource for PortId {
    fn FetchFieldImp< 'a>( &'a mut self, field: &mut crate::flux::FieldImp< 'a>) {
        self.0.FetchFieldImp( field);
    }
}

impl Default for PortDir {
    fn default() -> Self { Self::In }
}
impl crate::flux::IFluxImportSink for PortDir {
    fn FromFieldImp( &mut self, field: crate::flux::FieldImp) -> bool {
        if let crate::flux::FieldImp::Str( s) = field {
            *self = match *s {
                "In" => Self::In,
                "Out" => Self::Out,
                _ => return false,
            };
            return true;
        }
        false
    }
}
impl crate::flux::IFluxImportSource for PortDir {
    fn FetchFieldImp< 'a>( &'a mut self, field: &mut crate::flux::FieldImp< 'a>) {
        *field = crate::flux::FieldImp::FluxSink( self);
    }
}

impl crate::flux::IFluxImportSink for PortType
{
    fn	FromFieldImp( &mut self, field: crate::flux::FieldImp) -> bool
    {
        if let crate::flux::FieldImp::Str( s) = field {
            *self = match *s {
                "Bool" => Self::Bool,
                "U8Val" => Self::U8Val,
                "U16Val" => Self::U16Val,
                "U32Val" => Self::U32Val,
                "U64Val" => Self::U64Val,
                "Custom" => Self::Custom( crate::silo::U32( 0)),
                custom if custom.starts_with( "Custom:") => {
                    let  	bits = custom[7..].parse::< u32>().unwrap_or( 0);
                    Self::Custom( crate::silo::U32( bits))
                }
                _ => return false,
            };
            return true;
        }
        return false;
    }
}
impl crate::flux::IFluxImportSource for PortType {
    fn FetchFieldImp< 'a>( &'a mut self, field: &mut crate::flux::FieldImp< 'a>) {
        *field = crate::flux::FieldImp::FluxSink( self);
    }
}

impl Default for PortDesc {
    fn default() -> Self {
        Self {
            _Name: String::new(),
            _Type: PortType::Bool,
            _Owner: crate::rube::module::ModuleId::default(),
        }
    }
}
