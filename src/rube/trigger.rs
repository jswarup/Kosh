//-- trigger.rs -------------------------------------------------------------------------------------------------------------------
use	std::ops::BitOr;
use	crate::{
    rube::{
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

/// Unified AoS trigger and metadata container for event-driven simulation.
/// Holds hot temporal trigger states ( 48 bytes per cell) in `_Triggers` and cold metadata in `_Meta`.
#[derive( Clone, Debug)]
pub struct TriggerWad
{
    pub _Triggers: Buff< TriggerState>,
    pub _Meta: Buff< TriggerMeta>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for TriggerWad
{
    fn	default() -> Self
    {
        return Self {
            _Triggers: Buff::New(),
            _Meta: Buff::New(),
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TriggerWad
{
    pub fn	New() -> Self
    {
        return Self::default();
    }

    #[inline]
    pub fn	Add( &mut self, name: &str, initial: Reg) -> TriggerId
    {
        return self.AddTyped( name, PortType::Bool, initial);
    }

    #[inline]
    pub fn	AddTyped( &mut self, name: &str, portType: PortType, initial: Reg) -> TriggerId
    {
        let  	nameStr = name.to_string();
        let  	id = U32( self._Triggers.len() as u32);
        let  	newSize = id + U32( 1);
        self._Triggers.Resize( newSize, |_| TriggerState::New( initial));
        self._Meta.Resize( newSize, |_| TriggerMeta::New( nameStr.clone(), portType));
        return id;
    }

    #[inline]
    pub fn	Len( &self) -> usize
    {
        return self._Triggers.len();
    }

    #[inline]
    pub fn	IsEmpty( &self) -> bool
    {
        return self._Triggers.is_empty();
    }

    #[inline]
    pub fn	Get( &self, id: TriggerId) -> Reg
    {
        return self._Triggers[usize::from( id)].Current();
    }

    #[inline]
    pub fn	GetFuture( &self, id: TriggerId) -> Reg
    {
        return self._Triggers[usize::from( id)].Future();
    }

    #[inline]
    pub fn	Name( &self, id: TriggerId) -> &str
    {
        return self._Meta[usize::from( id)].Name();
    }

    #[inline]
    pub fn	PortType( &self, id: TriggerId) -> PortType
    {
        return self._Meta[usize::from( id)].PortType();
    }

    #[inline]
    pub fn	IsArmed( &self, id: TriggerId) -> bool
    {
        let  	trig = &self._Triggers[usize::from( id)];
        return trig._Current != trig._Future;
    }

    #[inline]
    pub fn	InitValue( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers[usize::from( id)].Init( val);
    }

    #[inline]
    pub fn	SetFutureValue( &mut self, id: TriggerId, val: Reg) -> bool
    {
        let  	idx = usize::from( id);
        let  	changed = self._Triggers[idx]._Current != val;
        self._Triggers[idx].SetFuture( val);
        return changed;
    }

    #[inline]
    pub fn	Advance( &mut self, id: TriggerId) -> ( Reg, Reg)
    {
        return self._Triggers[usize::from( id)].Advance();
    }

    #[inline]
    pub fn	IsEdge( &self, id: TriggerId) -> bool
    {
        return self._Triggers[usize::from( id)].IsEdge();
    }

    #[inline]
    pub fn	IsPosedge( &self, id: TriggerId) -> bool
    {
        return self._Triggers[usize::from( id)].IsPosedge();
    }

    #[inline]
    pub fn	IsNegedge( &self, id: TriggerId) -> bool
    {
        return self._Triggers[usize::from( id)].IsNegedge();
    }

    #[inline]
    pub fn	And( &self, in1: TriggerId, in2: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( self.Get( in1) & self.Get( in2)).Masked( mask);
    }

    #[inline]
    pub fn	Or( &self, in1: TriggerId, in2: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( self.Get( in1) | self.Get( in2)).Masked( mask);
    }

    #[inline]
    pub fn	Xor( &self, in1: TriggerId, in2: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( self.Get( in1) ^ self.Get( in2)).Masked( mask);
    }

    #[inline]
    pub fn	Not( &self, in1: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( !self.Get( in1)).Masked( mask);
    }

    #[inline]
    pub fn	Nand( &self, in1: TriggerId, in2: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( !( self.Get( in1) & self.Get( in2))).Masked( mask);
    }

    #[inline]
    pub fn	Nor( &self, in1: TriggerId, in2: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( !( self.Get( in1) | self.Get( in2))).Masked( mask);
    }

    #[inline]
    pub fn	Xnor( &self, in1: TriggerId, in2: TriggerId) -> Reg
    {
        let  	mask = self.PortType( in1).Mask();
        return ( !( self.Get( in1) ^ self.Get( in2))).Masked( mask);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TriggerSense( pub U8);

impl TriggerSense
{
    pub const NONE: Self = Self( U8( 0));
    pub const POS_EDGE: Self = Self( U8( 1));
    pub const NEG_EDGE: Self = Self( U8( 2));
    pub const EDGE: Self = Self( U8( 1 | 2));

    #[inline]
    pub const fn	IsNone( &self) -> bool
    {
        return self.0.0 == 0;
    }

    #[inline]
    pub const fn	Contains( &self, other: Self) -> bool
    {
        return ( self.0.0 & other.0.0) == other.0.0;
    }

    #[inline]
    pub fn	Matches( &self, wad: &TriggerWad, id: TriggerId) -> bool
    {
        if self.IsNone() || !wad.IsEdge( id) {
            return false;
        }
        if self.Contains( Self::POS_EDGE) && wad.IsPosedge( id) {
            return true;
        }
        if self.Contains( Self::NEG_EDGE) && wad.IsNegedge( id) {
            return true;
        }
        return self.0 == Self::EDGE.0;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BitOr for TriggerSense
{
    type Output = Self;

    #[inline]
    fn	bitor( self, rhs: Self) -> Self::Output
    {
        return Self( self.0 | rhs.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
