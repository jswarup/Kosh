//-- signal.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::rube::{
    port::PortType,
    regval::RegVal,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Hot temporal state cell for a single signal in AoS layout.
/// Exactly 48 bytes ( 3 x 16-byte RegVal), fitting inside a single 64-byte L1 cache line.
/// Zero pointers, zero heap allocations, Copy-able.
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct SignalState
{
    pub _Past: RegVal,
    pub _Current: RegVal,
    pub _Future: RegVal,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SignalState
{
    #[inline]
    pub fn	New( initVal: RegVal) -> Self
    {
        return Self {
            _Past: initVal,
            _Current: initVal,
            _Future: initVal,
        };
    }

    #[inline]
    pub fn	Past( &self) -> RegVal
    {
        return self._Past;
    }

    #[inline]
    pub fn	Current( &self) -> RegVal
    {
        return self._Current;
    }

    #[inline]
    pub fn	Future( &self) -> RegVal
    {
        return self._Future;
    }

    #[inline]
    pub fn	SetFuture( &mut self, val: RegVal)
    {
        self._Future = val;
    }

    #[inline]
    pub fn	Init( &mut self, val: RegVal)
    {
        self._Past = val;
        self._Current = val;
        self._Future = val;
    }

    #[inline]
    pub fn	Advance( &mut self) -> ( RegVal, RegVal)
    {
        let  	past = self._Current;
        let  	current = self._Future;
        self._Past = past;
        self._Current = current;
        return ( past, current);
    }

    #[inline]
    pub fn	IsEdge( &self) -> bool
    {
        return self._Past != self._Current;
    }

    #[inline]
    pub fn	IsPosedge( &self) -> bool
    {
        return self._Past.IsFalse() && self._Current.IsTrue();
    }

    #[inline]
    pub fn	IsNegedge( &self) -> bool
    {
        return self._Past.IsTrue() && self._Current.IsFalse();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Cold metadata stored separately from hot simulation arrays.
#[derive( Clone, Debug)]
pub struct SignalMeta
{
    pub _Name: String,
    pub _Type: PortType,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SignalMeta
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
    pub fn	PortType( &self) -> PortType
    {
        return self._Type;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
