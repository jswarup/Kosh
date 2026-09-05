//-- module.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::sync::Arc;
use	crate::{
    flux::{ FieldExp, IFluxExportSource, FieldImp, IFluxImportSource },
    rube::{
        coro_kernel::{ CoroInstance, CoroKernelFactory },
        reg::Reg,
        registry::KernelRegistry,
        trigger::TriggerId,
    },
    silo::{ Buff, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ModuleId( pub U32);

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IModule
{
    fn	Id( &self) -> ModuleId;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for ModuleId
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return *self;
    }
}

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
    Add,
    Sub,
    Shl,
    Shr,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl KernelOp
{
    #[inline]
    pub fn	Eval( self, in1: Reg, in2: Reg, mask: u64) -> Reg
    {
        let  	res = match self {
            Self::Nand => !( in1 & in2),
            Self::And => in1 & in2,
            Self::Or => in1 | in2,
            Self::Not => !in1,
            Self::Xor => in1 ^ in2,
            Self::Nor => !( in1 | in2),
            Self::Xnor => !( in1 ^ in2),
            Self::Add => {
                if in1.IsX() || in2.IsX() {
                    Reg::Unknown( mask)
                } else {
                    Reg::Known( in1.Val().wrapping_add( in2.Val()))
                }
            }
            Self::Sub => {
                if in1.IsX() || in2.IsX() {
                    Reg::Unknown( mask)
                } else {
                    Reg::Known( in1.Val().wrapping_sub( in2.Val()))
                }
            }
            Self::Shl => {
                if in1.IsX() || in2.IsX() {
                    Reg::Unknown( mask)
                } else {
                    Reg::Known( in1.Val().wrapping_shl( in2.Val() as u32))
                }
            }
            Self::Shr => {
                if in1.IsX() || in2.IsX() {
                    Reg::Unknown( mask)
                } else {
                    Reg::Known( in1.Val().wrapping_shr( in2.Val() as u32))
                }
            }
        };
        return res.Masked( mask);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub enum KernelKind
{
    None,
    Fast( KernelOp),
    Behavioral( Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>),
    Custom( &'static str),
    Coro( CoroKernelFactory),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for KernelKind
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::None => write!( f, "KernelKind::None"),
            Self::Fast( op) => write!( f, "KernelKind::Fast({:?})", op),
            Self::Behavioral( _) => write!( f, "Behavioral( ..)"),
            Self::Custom( n) => write!( f, "Custom( {})", n),
            Self::Coro( _) => write!( f, "Coro( ..)"),
        }
    }
}

impl KernelKind
{
    #[inline]
    pub const fn	IsNone( &self) -> bool
    {
        return matches!( self, Self::None);
    }

    #[inline]
    pub const fn	ToFastOp( &self) -> Option< KernelOp>
    {
        return match self {
            Self::Fast( op) => Some( *op),
            _ => None,
        };
    }

    #[inline]
    pub fn	ClassKey( &self) -> ( u8, usize)
    {
        return match self {
            Self::None => ( 2, 0),
            Self::Fast( op) => ( 0, *op as usize),
            KernelKind::Behavioral( callback) => {
                let  	rawDyn: *const ( dyn Fn( &[Reg], &mut [Reg]) + Send + Sync) = Arc::as_ptr( callback);
                let  	vtablePtr = unsafe { std::mem::transmute::< _, ( usize, usize)>( rawDyn).1 };
                return ( 2, vtablePtr);
            }
            KernelKind::Custom( name) => {
                return ( 3, name.as_ptr() as usize);
            }
            KernelKind::Coro( factory) => {
                let  	rawDyn: *const ( dyn Fn() -> CoroInstance + Send + Sync) = Arc::as_ptr( factory);
                let  	vtablePtr = unsafe { std::mem::transmute::< _, ( usize, usize)>( rawDyn).1 };
                return ( 4, vtablePtr);
            }
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Module
{
    pub _Id:           ModuleId,
    pub _Parent:       Option< ModuleId>,
    pub _Name:         String,
    pub _InPorts:      USeg,
    pub _OutPorts:     USeg,
    pub _SubModules:   USeg,
    pub _Descendents:  USeg,
    pub _Kernel:       KernelKind,
    pub _IsSealed:     bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Module
{
    pub fn	NewContainer( id: ModuleId, parent: Option< ModuleId>, name: &str) -> Self
    {
        return Self {
            _Id:           id,
            _Parent:       parent,
            _Name:         name.to_string(),
            _InPorts:      USeg::New( U32::_0, U32::_0),
            _OutPorts:     USeg::New( U32::_0, U32::_0),
            _SubModules:   USeg::New( U32::_0, U32::_0),
            _Descendents:  USeg::New( U32::_0, U32::_0),
            _Kernel:       KernelKind::None,
            _IsSealed:     false,
        };
    }

    pub fn	New( id: ModuleId, parent: Option< ModuleId>, name: &str, inPorts: USeg, outPorts: USeg, kernel: KernelKind) -> Self
    {
        return Self {
            _Id:           id,
            _Parent:       parent,
            _Name:         name.to_string(),
            _InPorts:      inPorts,
            _OutPorts:     outPorts,
            _SubModules:   USeg::New( U32::_0, U32::_0),
            _Descendents:  USeg::New( U32::_0, U32::_0),
            _Kernel:       kernel,
            _IsSealed:     false,
        };
    }

    #[inline]
    pub fn	IsContainer( &self) -> bool
    {
        return self._Kernel.IsNone();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for Module
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Compact 24-byte Copy struct for standard 2-in/1-out ( and 1-in/1-out) gates and bus arithmetic.
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FastModule
{
    pub _ModuleId: ModuleId,
    pub _In1:      TriggerId,
    pub _In2:      TriggerId,
    pub _Out:      TriggerId,
    pub _Mask:     u64,
    pub _Op:       KernelOp,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FastModule
{
    #[inline]
    pub const fn	New( modId: ModuleId, in1: TriggerId, in2: TriggerId, out: TriggerId, op: KernelOp, mask: u64) -> Self
    {
        return Self {
            _ModuleId: modId,
            _In1:      in1,
            _In2:      in2,
            _Out:      out,
            _Mask:     mask,
            _Op:       op,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct CustomModule
{
    pub _ModuleId:    ModuleId,
    pub _InTriggers:  Buff< TriggerId>,
    pub _OutTriggers: Buff< TriggerId>,
    pub _KernelName:  &'static str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CustomModule
{
    pub fn	New(
        moduleId: ModuleId,
        inTriggers: Buff< TriggerId>,
        outTriggers: Buff< TriggerId>,
        kernelName: &'static str,
    ) -> Self
    {
        return Self {
            _ModuleId:    moduleId,
            _InTriggers:  inTriggers,
            _OutTriggers: outTriggers,
            _KernelName:  kernelName,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Structure-of-Arrays (SoA) SIMT Warp for homogeneous FastModule blocks.
#[derive( Clone, Debug)]
pub struct FastWarp
{
    pub _Op:       KernelOp,
    pub _ModStart: U32,
    pub _Count:    U32,
    pub _Mask:     u64,
    pub _In1:      Buff< TriggerId>,
    pub _In2:      Buff< TriggerId>,
    pub _Out:      Buff< TriggerId>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FastWarp
{
    #[inline]
    pub fn	New(
        op: KernelOp,
        modStart: U32,
        count: U32,
        mask: u64,
        in1: Buff< TriggerId>,
        in2: Buff< TriggerId>,
        out: Buff< TriggerId>,
    ) -> Self
    {
        return Self {
            _Op:       op,
            _ModStart: modStart,
            _Count:    count,
            _Mask:     mask,
            _In1:      in1,
            _In2:      in2,
            _Out:      out,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Structure-of-Arrays (SoA) SIMT Warp for homogeneous CustomModule closures sharing a vtable.
#[derive( Clone)]
pub struct CustomWarp
{
    pub _KernelName:  &'static str,
    pub _ModStart:    U32,
    pub _Count:       U32,
    pub _InTriggers:  Buff< Buff< TriggerId>>,
    pub _OutTriggers: Buff< Buff< TriggerId>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CustomWarp
{
    #[inline]
    pub fn	New(
        kernelName: &'static str,
        modStart: U32,
        count: U32,
        inTriggers: Buff< Buff< TriggerId>>,
        outTriggers: Buff< Buff< TriggerId>>,
    ) -> Self
    {
        return Self {
            _KernelName: kernelName,
            _ModStart: modStart,
            _Count: count,
            _InTriggers: inTriggers,
            _OutTriggers: outTriggers,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for ModuleId {
    fn FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>) {
        self.0.FetchFieldExp( field);
    }
}
impl crate::flux::IFluxImportSink for ModuleId {
    fn FromFieldImp( &mut self, field: FieldImp) -> bool {
        self.0.FromFieldImp( field)
    }
}
impl IFluxImportSource for ModuleId {
    fn FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>) {
        self.0.FetchFieldImp( field);
    }
}

impl Default for KernelOp {
    fn default() -> Self { Self::Nand }
}
impl IFluxExportSource for KernelOp {
    fn FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>) {
        let s = match self {
            Self::Nand => "Nand", Self::And => "And", Self::Or => "Or", Self::Not => "Not",
            Self::Xor => "Xor", Self::Nor => "Nor", Self::Xnor => "Xnor", Self::Add => "Add",
            Self::Sub => "Sub", Self::Shl => "Shl", Self::Shr => "Shr",
        };
        *field = FieldExp::Str( s);
    }
}
impl crate::flux::IFluxImportSink for KernelOp {
    fn FromFieldImp( &mut self, field: FieldImp) -> bool {
        if let FieldImp::Str( s) = field {
            *self = match *s {
                "Nand" => Self::Nand, "And" => Self::And, "Or" => Self::Or, "Not" => Self::Not,
                "Xor" => Self::Xor, "Nor" => Self::Nor, "Xnor" => Self::Xnor, "Add" => Self::Add,
                "Sub" => Self::Sub, "Shl" => Self::Shl, "Shr" => Self::Shr,
                _ => return false,
            };
            return true;
        }
        false
    }
}
impl IFluxImportSource for KernelOp {
    fn FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>) {
        *field = FieldImp::FluxSink( self);
    }
}

impl Default for KernelKind
{
    fn	default() -> Self
    {
        Self::None
    }
}

impl IFluxExportSource for KernelKind
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        match self {
            Self::None => *field = FieldExp::Str( "None"),
            Self::Fast( op) => op.FetchFieldExp( field),
            Self::Custom( name) => *field = FieldExp::Str( name),
            Self::Behavioral( _) => *field = FieldExp::Str( "Behavioral"),
            Self::Coro( _) => *field = FieldExp::Str( "Coro"),
        }
    }
}

impl crate::flux::IFluxImportSink for KernelKind
{
    fn	FromFieldImp( &mut self, field: FieldImp) -> bool
    {
        if let FieldImp::Str( s) = field {
            if *s == "None" {
                *self = Self::None;
                return true;
            }
            if *s == "Behavioral" {
                return false;
            }
            if *s == "Coro" {
                return false;
            }
            let  	sCopy: String = s.to_string();
            let  	mut op = KernelOp::default();
            let  	mut tmp: &str = &sCopy;
            if crate::flux::IFluxImportSink::FromFieldImp( &mut op, FieldImp::Str( &mut tmp)) {
                *self = Self::Fast( op);
                return true;
            }
            *self = Self::Custom( KernelRegistry::FindOrInternStaticName( &sCopy));
            return true;
        }
        return false;
    }
}

impl IFluxImportSource for KernelKind
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        *field = FieldImp::FluxSink( self);
    }
}

crate::ImplFluxSource!( Module, _Id, _Parent, _Name, _InPorts, _OutPorts, _SubModules, _Descendents, _Kernel, _IsSealed);

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct BehavioralWarp
{
    pub _ModStart:    U32,
    pub _Count:       U32,
    pub _Instances:   Buff< Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>>,
    pub _InTriggers:  Buff< Buff< TriggerId>>,
    pub _OutTriggers: Buff< Buff< TriggerId>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BehavioralWarp
{
    pub fn	New(
        modStart: U32,
        count: U32,
        instances: Buff< Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>>,
        inTriggers: Buff< Buff< TriggerId>>,
        outTriggers: Buff< Buff< TriggerId>>,
    ) -> Self
    {
        return Self {
            _ModStart:    modStart,
            _Count:       count,
            _Instances:   instances,
            _InTriggers:  inTriggers,
            _OutTriggers: outTriggers,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Module
{
    fn	default() -> Self
    {
        return Self {
            _Id:          ModuleId::default(),
            _Parent:      None,
            _Name:        String::new(),
            _InPorts:     USeg::New( U32::_0, U32::_0),
            _OutPorts:    USeg::New( U32::_0, U32::_0),
            _SubModules:  USeg::New( U32::_0, U32::_0),
            _Descendents: USeg::New( U32::_0, U32::_0),
            _Kernel:      KernelKind::default(),
            _IsSealed:    false,
        };
    }
}

