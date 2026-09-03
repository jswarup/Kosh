//-- module.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::sync::Arc;
use	crate::{
    rube::{
        reg::Reg,
        trigger::TriggerId,
    },
    silo::{ Buff, U32, USeg },
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
    Custom( Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for KernelKind
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            Self::None => write!( f, "KernelKind::None"),
            Self::Nand => write!( f, "KernelKind::Nand"),
            Self::And => write!( f, "KernelKind::And"),
            Self::Or => write!( f, "KernelKind::Or"),
            Self::Not => write!( f, "KernelKind::Not"),
            Self::Xor => write!( f, "KernelKind::Xor"),
            Self::Nor => write!( f, "KernelKind::Nor"),
            Self::Xnor => write!( f, "KernelKind::Xnor"),
            Self::Add => write!( f, "KernelKind::Add"),
            Self::Sub => write!( f, "KernelKind::Sub"),
            Self::Shl => write!( f, "KernelKind::Shl"),
            Self::Shr => write!( f, "KernelKind::Shr"),
            Self::Custom( _) => write!( f, "KernelKind::Custom(...)"),
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
            Self::None => None,
            Self::Nand => Some( KernelOp::Nand),
            Self::And => Some( KernelOp::And),
            Self::Or => Some( KernelOp::Or),
            Self::Not => Some( KernelOp::Not),
            Self::Xor => Some( KernelOp::Xor),
            Self::Nor => Some( KernelOp::Nor),
            Self::Xnor => Some( KernelOp::Xnor),
            Self::Add => Some( KernelOp::Add),
            Self::Sub => Some( KernelOp::Sub),
            Self::Shl => Some( KernelOp::Shl),
            Self::Shr => Some( KernelOp::Shr),
            Self::Custom( _) => None,
        };
    }

    #[inline]
    pub fn	ClassKey( &self) -> ( u8, usize)
    {
        return match self {
            Self::None => ( 2, 0),
            Self::Nand => ( 0, KernelOp::Nand as usize),
            Self::And => ( 0, KernelOp::And as usize),
            Self::Or => ( 0, KernelOp::Or as usize),
            Self::Not => ( 0, KernelOp::Not as usize),
            Self::Xor => ( 0, KernelOp::Xor as usize),
            Self::Nor => ( 0, KernelOp::Nor as usize),
            Self::Xnor => ( 0, KernelOp::Xnor as usize),
            Self::Add => ( 0, KernelOp::Add as usize),
            Self::Sub => ( 0, KernelOp::Sub as usize),
            Self::Shl => ( 0, KernelOp::Shl as usize),
            Self::Shr => ( 0, KernelOp::Shr as usize),
            Self::Custom( callback) => {
                let  	rawDyn: *const ( dyn Fn( &[Reg], &mut [Reg]) + Send + Sync) = Arc::as_ptr( callback);
                let  	( _dataPtr, vtablePtr): ( *const (), *const ()) = unsafe {
                    std::mem::transmute( rawDyn)
                };
                ( 1, vtablePtr as usize)
            }
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Module
{
    pub _Id:          ModuleId,
    pub _Name:        String,
    pub _InPorts:     USeg,
    pub _OutPorts:    USeg,
    pub _Kernel:      KernelKind,
    pub _SubModules:  Buff< ModuleId>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Module
{
    pub fn	NewContainer( id: ModuleId, name: &str) -> Self
    {
        return Self {
            _Id:          id,
            _Name:        name.to_string(),
            _InPorts:     USeg::New( U32::_0, U32::_0),
            _OutPorts:    USeg::New( U32::_0, U32::_0),
            _Kernel:      KernelKind::None,
            _SubModules:  Buff::New(),
        };
    }

    pub fn	New( id: ModuleId, name: &str, inPorts: USeg, outPorts: USeg, kernel: KernelKind) -> Self
    {
        return Self {
            _Id:          id,
            _Name:        name.to_string(),
            _InPorts:     inPorts,
            _OutPorts:    outPorts,
            _Kernel:      kernel,
            _SubModules:  Buff::New(),
        };
    }

    #[inline]
    pub fn	SubModules( &self) -> &[ModuleId]
    {
        return &self._SubModules;
    }

    #[inline]
    pub fn	IsContainer( &self) -> bool
    {
        return self._Kernel.IsNone();
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
    pub _Op:       KernelOp,
    pub _Mask:     u64,
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
            _Op:       op,
            _Mask:     mask,
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
    pub _Callback:    Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CustomModule
{
    pub fn	New(
        moduleId: ModuleId,
        inTriggers: Buff< TriggerId>,
        outTriggers: Buff< TriggerId>,
        callback: Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>,
    ) -> Self
    {
        return Self {
            _ModuleId:    moduleId,
            _InTriggers:  inTriggers,
            _OutTriggers: outTriggers,
            _Callback:    callback,
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
    pub _VtablePtr:   usize,
    pub _ModStart:    U32,
    pub _Count:       U32,
    pub _Instances:   Buff< Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>>,
    pub _InTriggers:  Buff< Buff< TriggerId>>,
    pub _OutTriggers: Buff< Buff< TriggerId>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CustomWarp
{
    #[inline]
    pub fn	New(
        vtablePtr: usize,
        modStart: U32,
        count: U32,
        instances: Buff< Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>>,
        inTriggers: Buff< Buff< TriggerId>>,
        outTriggers: Buff< Buff< TriggerId>>,
    ) -> Self
    {
        return Self {
            _VtablePtr:   vtablePtr,
            _ModStart:    modStart,
            _Count:       count,
            _Instances:   instances,
            _InTriggers:  inTriggers,
            _OutTriggers: outTriggers,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

