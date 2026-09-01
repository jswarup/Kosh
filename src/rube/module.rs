//-- module.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::sync::Arc;
use	crate::{
    rube::{
        port::PortId,
        reg::Reg,
        trigger::TriggerId,
    },
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
    pub const fn	ToFastOp( &self) -> Option< KernelOp>
    {
        return match self {
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
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Module
{
    pub _Id: ModuleId,
    pub _Name: String,
    pub _InPorts: Buff< PortId>,
    pub _OutPorts: Buff< PortId>,
    pub _Kernel: KernelKind,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Module
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
