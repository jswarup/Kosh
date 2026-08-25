//-- trigger.rs -------------------------------------------------------------------------------------------------------------------
use	std::ops::BitOr;
use	crate::{
    rube::reg::Reg,
    silo::{ Buff, U32, U8 },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type TriggerId = U32;

//--------------------------------------------------------------------------------------------------------------------------------- 

/// Decouples cold metadata ( `names`) from hot simulation states ( `past`, `current`, `future`),
/// ensuring cache-friendly contiguous memory access during delta-cycle propagation.
#[derive( Clone, Debug, Default)]
pub struct TriggerWad
{
    pub _Names: Buff< String>,
    pub _Past: Buff< Reg>,
    pub _Current: Buff< Reg>,
    pub _Future: Buff< Reg>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TriggerWad
{
    pub fn	New() -> Self
    {
        Self::default()
    }

    pub fn	new() -> Self
    {
        Self::default()
    }

    /// Add a signal with initial state
    pub fn	add( &mut self, name: impl Into< String>, initial: Reg) -> TriggerId
    {
        let  	nameStr = name.into();
        let  	id = U32( self._Current.len() as u32);
        let  	newSize = id + U32( 1);
        self._Names.Resize( newSize, |_| nameStr.clone());
        self._Past.Resize( newSize, |_| initial);
        self._Current.Resize( newSize, |_| initial);
        self._Future.Resize( newSize, |_| initial);
        return id;
    }

    #[inline]
    pub fn	len( &self) -> usize
    {
        return self._Current.len();
    }

    #[inline]
    pub fn	is_empty( &self) -> bool
    {
        return self._Current.is_empty();
    }

    #[inline]
    pub fn	get( &self, id: TriggerId) -> Reg
    {
        return self._Current[id.0 as usize];
    }

    #[inline]
    pub fn	get_past( &self, id: TriggerId) -> Reg
    {
        return self._Past[id.0 as usize];
    }

    #[inline]
    pub fn	get_future( &self, id: TriggerId) -> Reg
    {
        return self._Future[id.0 as usize];
    }

    #[inline]
    pub fn	name( &self, id: TriggerId) -> &str
    {
        return &self._Names[id.0 as usize];
    }

    #[inline]
    pub fn	is_armed( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        return self._Current[idx] != self._Future[idx];
    }

    #[inline]
    pub fn	init_value( &mut self, id: TriggerId, val: Reg)
    {
        let  	idx = id.0 as usize;
        self._Past[idx] = val;
        self._Current[idx] = val;
        self._Future[idx] = val;
    }

    #[inline]
    pub fn	set_future_value( &mut self, id: TriggerId, val: Reg) -> bool
    {
        let  	idx = id.0 as usize;
        self._Future[idx] = val;
        return self._Current[idx] != val;
    }

    #[inline]
    pub fn	advance( &mut self, id: TriggerId) -> ( Reg, Reg)
    {
        let  	idx = id.0 as usize;
        self._Past[idx] = self._Current[idx];
        self._Current[idx] = self._Future[idx];
        return ( self._Past[idx], self._Current[idx]);
    }

    // --- Logic Gate Methods ---

    #[inline]
    pub fn	and( &self, _In1: TriggerId, _In2: TriggerId) -> Reg
    {
        return self._Current[_In1.0 as usize] & self._Current[_In2.0 as usize];
    }

    #[inline]
    pub fn	or( &self, _In1: TriggerId, _In2: TriggerId) -> Reg
    {
        return self._Current[_In1.0 as usize] | self._Current[_In2.0 as usize];
    }

    #[inline]
    pub fn	xor( &self, _In1: TriggerId, _In2: TriggerId) -> Reg
    {
        return self._Current[_In1.0 as usize] ^ self._Current[_In2.0 as usize];
    }

    #[inline]
    pub fn	not( &self, _In1: TriggerId) -> Reg
    {
        return !self._Current[_In1.0 as usize];
    }

    #[inline]
    pub fn	nand( &self, _In1: TriggerId, _In2: TriggerId) -> Reg
    {
        return !( self.and( _In1, _In2));
    }

    #[inline]
    pub fn	nor( &self, _In1: TriggerId, _In2: TriggerId) -> Reg
    {
        return !( self.or( _In1, _In2));
    }

    #[inline]
    pub fn	xnor( &self, _In1: TriggerId, _In2: TriggerId) -> Reg
    {
        return !( self.xor( _In1, _In2));
    }

    // --- Edge Detection Methods ---

    #[inline]
    pub fn	is_edge( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        return self._Current[idx] != self._Past[idx];
    }

    #[inline]
    pub fn	is_posedge( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        let  	cur = self._Current[idx];
        let  	past = self._Past[idx];
        return cur != past && ( cur.get_bool() && !past.get_bool() && cur.is_valid());
    }

    #[inline]
    pub fn	is_negedge( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        let  	cur = self._Current[idx];
        let  	past = self._Past[idx];
        return cur != past && ( !cur.get_bool() && past.get_bool() && past.is_valid());
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
    pub const fn	is_none( &self) -> bool
    {
        return self.0.0 == 0;
    }

    #[inline]
    pub const fn	contains( &self, other: Self) -> bool
    {
        return ( self.0.0 & other.0.0) == other.0.0;
    }

    #[inline]
    pub fn	matches( &self, signals: &TriggerWad, id: TriggerId) -> bool
    {
        if self.is_none() || !signals.is_edge( id) {
            return false;
        }
        if self.contains( Self::POS_EDGE) && signals.is_posedge( id) {
            return true;
        }
        if self.contains( Self::NEG_EDGE) && signals.is_negedge( id) {
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
