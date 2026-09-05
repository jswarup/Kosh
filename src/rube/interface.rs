use	crate::rube::port::PortDir;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataType
{
    Logic(usize),
    Bool,
    Integer,
    Real,
}

impl DataType
{
    pub fn	Width( &self) -> usize
    {
        match self {
            DataType::Logic( w) => *w,
            DataType::Bool      => 1,
            DataType::Integer   => 32,
            DataType::Real      => 64,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BusType
{
    Single,
    Bus(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polarity
{
    Positive,
    Negative,
}

impl Default for Polarity
{
    fn	default() -> Self
    {
        Polarity::Positive
    }
}

#[derive(Clone, Debug, Default)]
pub struct PortAttributes
{
    pub _IsClock:       bool,
    pub _IsReset:       bool,
    pub _IsValid:       bool,
    pub _IsReady:       bool,
    pub _ClockPolarity: Polarity,
    pub _ResetPolarity: Polarity,
    pub _Tags:          &'static [&'static str],
}

#[derive(Clone, Debug)]
pub struct PortInterface
{
    pub _Name:          &'static str,
    pub _Width:         usize,
    pub _DataType:      DataType,
    pub _Direction:     PortDir,
    pub _BusType:       BusType,
    pub _Attributes:    PortAttributes,
    pub _Documentation: Option<&'static str>,
}

impl PortInterface
{
    pub const fn Input( name: &'static str, width: usize, doc: Option<&'static str>) -> Self
    {
        Self {
            _Name:          name,
            _Width:         width,
            _DataType:      DataType::Logic( width),
            _Direction:     PortDir::In,
            _BusType:       BusType::Bus( width),
            _Attributes:    PortAttributes {
                _IsClock:       false,
                _IsReset:       false,
                _IsValid:       false,
                _IsReady:       false,
                _ClockPolarity: Polarity::Positive,
                _ResetPolarity: Polarity::Positive,
                _Tags:          &[],
            },
            _Documentation: doc,
        }
    }

    pub const fn Output( name: &'static str, width: usize, doc: Option<&'static str>) -> Self
    {
        Self {
            _Name:          name,
            _Width:         width,
            _DataType:      DataType::Logic( width),
            _Direction:     PortDir::Out,
            _BusType:       BusType::Bus( width),
            _Attributes:    PortAttributes {
                _IsClock:       false,
                _IsReset:       false,
                _IsValid:       false,
                _IsReady:       false,
                _ClockPolarity: Polarity::Positive,
                _ResetPolarity: Polarity::Positive,
                _Tags:          &[],
            },
            _Documentation: doc,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterType
{
    Integer,
    String,
    Real,
    Boolean,
}

#[derive(Clone, Debug)]
pub struct ParameterInterface
{
    pub _Name:          &'static str,
    pub _ParamType:     ParameterType,
    pub _Default:       Option<&'static str>,
    pub _Documentation: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct ModuleInterface
{
    pub _Name:          &'static str,
    pub _Version:       &'static str,
    pub _Description:   &'static str,
    pub _Vendor:        Option<&'static str>,
    pub _InPorts:       &'static [PortInterface],
    pub _OutPorts:      &'static [PortInterface],
    pub _Parameters:    &'static [ParameterInterface],
}

#[derive(Clone, Debug)]
pub enum InterfaceError
{
    DuplicatePortName(&'static str),
    WidthMismatch(&'static str),
}

impl ModuleInterface
{
    pub fn	ValidatePorts( &self) -> Result< (), InterfaceError>
    {
        let mut	names = std::collections::HashSet::new();
        
        for port in self._InPorts.iter().chain( self._OutPorts.iter()) {
            if !names.insert( port._Name) {
                return Err( InterfaceError::DuplicatePortName( port._Name));
            }
        }
        
        Ok( ())
    }

    pub fn	ToSystemVerilog( &self) -> String
    {
        let mut	sv = format!( "module {} (\n", self._Name);
        
        for port in self._InPorts {
            match port._BusType {
                BusType::Bus( w) if w > 1 => {
                    sv.push_str( &format!( "    input [{}:0] {},\n", w - 1, port._Name));
                }
                _ => {
                    sv.push_str( &format!( "    input {},\n", port._Name));
                }
            }
        }
        
        for ( i, port) in self._OutPorts.iter().enumerate() {
            let  	suffix = if i == self._OutPorts.len() - 1 { "\n" } else { ",\n" };
            match port._BusType {
                BusType::Bus( w) if w > 1 => {
                    sv.push_str( &format!( "    output [{}:0] {}{}", w - 1, port._Name, suffix));
                }
                _ => {
                    sv.push_str( &format!( "    output {}{}", port._Name, suffix));
                }
            }
        }
        
        sv.push_str( ");\n");
        sv.push_str( &format!( "    // {}\n", self._Description));
        sv.push_str( "endmodule\n");
        sv
    }
}

pub trait IModuleInterface
{
    fn	Interface() -> &'static ModuleInterface;

    fn	Name( &self) -> &str
    {
        Self::Interface()._Name
    }
}

/// Declarative macro for defining a module interface with minimal boilerplate.
#[macro_export]
macro_rules! DefineModuleInterface {
    (
        $target_type:ty,
        $name:expr,
        $version:expr,
        $desc:expr,
        inports: [ $( ($in_name:expr, $in_width:expr) ),* $(,)? ],
        outports: [ $( ($out_name:expr, $out_width:expr) ),* $(,)? ]
    ) => {
        impl $crate::rube::interface::IModuleInterface for $target_type
        {
            fn	Interface() -> &'static $crate::rube::interface::ModuleInterface
            {
                static INTERFACE: $crate::rube::interface::ModuleInterface = $crate::rube::interface::ModuleInterface {
                    _Name:          $name,
                    _Version:       $version,
                    _Description:   $desc,
                    _Vendor:        Some( "OrioleDesigns"),
                    _InPorts:       &[
                        $( $crate::rube::interface::PortInterface::Input( $in_name, $in_width, None) ),*
                    ],
                    _OutPorts:      &[
                        $( $crate::rube::interface::PortInterface::Output( $out_name, $out_width, None) ),*
                    ],
                    _Parameters:    &[],
                };
                &INTERFACE
            }
        }
    };
}
