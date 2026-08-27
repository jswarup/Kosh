//-- trigger.rs -------------------------------------------------------------------------------------------------------------------
use	std::ops::BitOr;
use	crate::{
    rube::{
        module::KernelOp,
        port::PortType,
        reg::Reg,
    },
    silo::{ Buff, U32, U8 },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type TriggerId = U32;

//---------------------------------------------------------------------------------------------------------------------------------

/// Hot temporal state cell for a single trigger in AoS layout.
/// Exactly 48 bytes ( 3 x 16-byte Reg), fitting inside a single 64-byte L1 cache line.
/// Zero pointers, zero heap allocations, Copy-able.
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct TriggerState
{
    pub _Past: Reg,
    pub _Current: Reg,
    pub _Future: Reg,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TriggerState
{
    #[inline]
    pub const fn	New( initVal: Reg) -> Self
    {
        return Self {
            _Past: initVal,
            _Current: initVal,
            _Future: initVal,
        };
    }

    #[inline]
    pub const fn	Past( &self) -> Reg
    {
        return self._Past;
    }

    #[inline]
    pub const fn	Current( &self) -> Reg
    {
        return self._Current;
    }

    #[inline]
    pub const fn	Future( &self) -> Reg
    {
        return self._Future;
    }

    #[inline]
    pub fn	SetFuture( &mut self, val: Reg)
    {
        self._Future = val;
    }

    #[inline]
    pub fn	Init( &mut self, val: Reg)
    {
        self._Past = val;
        self._Current = val;
        self._Future = val;
    }

    #[inline]
    pub fn	Advance( &mut self) -> ( Reg, Reg)
    {
        let  	past = self._Current;
        let  	current = self._Future;
        self._Past = past;
        self._Current = current;
        return ( past, current);
    }

    #[inline]
    pub const fn	IsEdge( &self) -> bool
    {
        return self._Past._Val != self._Current._Val || self._Past._X != self._Current._X;
    }

    #[inline]
    pub const fn	IsPosedge( &self) -> bool
    {
        return self._Past.IsFalse() && self._Current.IsTrue();
    }

    #[inline]
    pub const fn	IsNegedge( &self) -> bool
    {
        return self._Past.IsTrue() && self._Current.IsFalse();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Cold metadata stored separately from hot simulation arrays.
#[derive( Clone, Debug)]
pub struct TriggerMeta
{
    pub _Name: String,
    pub _Type: PortType,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TriggerMeta
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
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    #[inline]
    pub const fn	PortType( &self) -> PortType
    {
        return self._Type;
    }
}



//---------------------------------------------------------------------------------------------------------------------------------
