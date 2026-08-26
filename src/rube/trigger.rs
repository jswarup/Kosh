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
/// using an SoA ( Structure of Arrays) layout for optimal cache density and delta propagation.
#[derive( Clone, Debug)]
pub struct TriggerWad< Val>
{
    pub _Names: Buff< String>,
    pub _PastVal: Buff< Val>,
    pub _PastX: Buff< bool>,
    pub _CurrentVal: Buff< Val>,
    pub _CurrentX: Buff< bool>,
    pub _FutureVal: Buff< Val>,
    pub _FutureX: Buff< bool>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val> Default for TriggerWad< Val>
{
    fn	default() -> Self
    {
        Self {
            _Names: Buff::New(),
            _PastVal: Buff::New(),
            _PastX: Buff::New(),
            _CurrentVal: Buff::New(),
            _CurrentX: Buff::New(),
            _FutureVal: Buff::New(),
            _FutureX: Buff::New(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ITriggerWad< Val>
{
    fn	Add( &mut self, name: &str, initial: Reg< Val>) -> TriggerId;
    fn	Len( &self) -> usize;
    fn	IsEmpty( &self) -> bool;
    fn	Get( &self, id: TriggerId) -> Reg< Val>;
    fn	GetPast( &self, id: TriggerId) -> Reg< Val>;
    fn	GetFuture( &self, id: TriggerId) -> Reg< Val>;
    fn	Name( &self, id: TriggerId) -> &str;
    fn	IsArmed( &self, id: TriggerId) -> bool;
    fn	InitValue( &mut self, id: TriggerId, val: Reg< Val>);
    fn	SetFutureValue( &mut self, id: TriggerId, val: Reg< Val>) -> bool;
    fn	Advance( &mut self, id: TriggerId) -> ( Reg< Val>, Reg< Val>);
    fn	IsEdge( &self, id: TriggerId) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ITriggerWadBool
{
    fn	And( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>;
    fn	Or( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>;
    fn	Xor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>;
    fn	Not( &self, in1: TriggerId) -> Reg< bool>;
    fn	Nand( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>;
    fn	Nor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>;
    fn	Xnor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>;
    fn	IsPosedge( &self, id: TriggerId) -> bool;
    fn	IsNegedge( &self, id: TriggerId) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val> TriggerWad< Val>
{
    pub fn	New() -> Self
    {
        Self::default()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val: Clone + Copy + PartialEq> TriggerWad< Val>
{
    /// Add a signal with initial state
    pub fn	Add( &mut self, name: &str, initial: Reg< Val>) -> TriggerId
    {
        let  	nameStr = name.to_string();
        let  	id = U32( self._CurrentVal.len() as u32);
        let  	newSize = id + U32( 1);
        self._Names.Resize( newSize, |_| nameStr.clone());
        self._PastVal.Resize( newSize, |_| initial._Val);
        self._PastX.Resize( newSize, |_| initial._X);
        self._CurrentVal.Resize( newSize, |_| initial._Val);
        self._CurrentX.Resize( newSize, |_| initial._X);
        self._FutureVal.Resize( newSize, |_| initial._Val);
        self._FutureX.Resize( newSize, |_| initial._X);
        return id;
    }

    #[inline]
    pub fn	Len( &self) -> usize
    {
        return self._CurrentVal.len();
    }

    #[inline]
    pub fn	IsEmpty( &self) -> bool
    {
        return self._CurrentVal.is_empty();
    }

    #[inline]
    pub fn	Get( &self, id: TriggerId) -> Reg< Val>
    {
        let  	idx = id.0 as usize;
        return Reg::New( self._CurrentVal[idx], self._CurrentX[idx]);
    }

    #[inline]
    pub fn	GetPast( &self, id: TriggerId) -> Reg< Val>
    {
        let  	idx = id.0 as usize;
        return Reg::New( self._PastVal[idx], self._PastX[idx]);
    }

    #[inline]
    pub fn	GetFuture( &self, id: TriggerId) -> Reg< Val>
    {
        let  	idx = id.0 as usize;
        return Reg::New( self._FutureVal[idx], self._FutureX[idx]);
    }

    #[inline]
    pub fn	Name( &self, id: TriggerId) -> &str
    {
        return &self._Names[id.0 as usize];
    }

    #[inline]
    pub fn	IsArmed( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        return self._CurrentVal[idx] != self._FutureVal[idx] || self._CurrentX[idx] != self._FutureX[idx];
    }

    #[inline]
    pub fn	InitValue( &mut self, id: TriggerId, val: Reg< Val>)
    {
        let  	idx = id.0 as usize;
        self._PastVal[idx] = val._Val;
        self._PastX[idx] = val._X;
        self._CurrentVal[idx] = val._Val;
        self._CurrentX[idx] = val._X;
        self._FutureVal[idx] = val._Val;
        self._FutureX[idx] = val._X;
    }

    #[inline]
    pub fn	SetFutureValue( &mut self, id: TriggerId, val: Reg< Val>) -> bool
    {
        let  	idx = id.0 as usize;
        self._FutureVal[idx] = val._Val;
        self._FutureX[idx] = val._X;
        return self._CurrentVal[idx] != val._Val || self._CurrentX[idx] != val._X;
    }

    #[inline]
    pub fn	Advance( &mut self, id: TriggerId) -> ( Reg< Val>, Reg< Val>)
    {
        let  	idx = id.0 as usize;
        self._PastVal[idx] = self._CurrentVal[idx];
        self._PastX[idx] = self._CurrentX[idx];
        self._CurrentVal[idx] = self._FutureVal[idx];
        self._CurrentX[idx] = self._FutureX[idx];
        return (
            Reg::New( self._PastVal[idx], self._PastX[idx]),
            Reg::New( self._CurrentVal[idx], self._CurrentX[idx]),
        );
    }

    #[inline]
    pub fn	IsEdge( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        return self._CurrentVal[idx] != self._PastVal[idx] || self._CurrentX[idx] != self._PastX[idx];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< Val: Clone + Copy + PartialEq> ITriggerWad< Val> for TriggerWad< Val>
{
    fn	Add( &mut self, name: &str, initial: Reg< Val>) -> TriggerId
    {
        return self.Add( name, initial);
    }

    fn	Len( &self) -> usize
    {
        return self.Len();
    }

    fn	IsEmpty( &self) -> bool
    {
        return self.IsEmpty();
    }

    fn	Get( &self, id: TriggerId) -> Reg< Val>
    {
        return self.Get( id);
    }

    fn	GetPast( &self, id: TriggerId) -> Reg< Val>
    {
        return self.GetPast( id);
    }

    fn	GetFuture( &self, id: TriggerId) -> Reg< Val>
    {
        return self.GetFuture( id);
    }

    fn	Name( &self, id: TriggerId) -> &str
    {
        return self.Name( id);
    }

    fn	IsArmed( &self, id: TriggerId) -> bool
    {
        return self.IsArmed( id);
    }

    fn	InitValue( &mut self, id: TriggerId, val: Reg< Val>)
    {
        self.InitValue( id, val);
    }

    fn	SetFutureValue( &mut self, id: TriggerId, val: Reg< Val>) -> bool
    {
        return self.SetFutureValue( id, val);
    }

    fn	Advance( &mut self, id: TriggerId) -> ( Reg< Val>, Reg< Val>)
    {
        return self.Advance( id);
    }

    fn	IsEdge( &self, id: TriggerId) -> bool
    {
        return self.IsEdge( id);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TriggerWad< bool>
{
    // --- Logic Gate Methods ---

    #[inline]
    pub fn	And( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Get( in1) & self.Get( in2);
    }

    #[inline]
    pub fn	Or( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Get( in1) | self.Get( in2);
    }

    #[inline]
    pub fn	Xor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Get( in1) ^ self.Get( in2);
    }

    #[inline]
    pub fn	Not( &self, in1: TriggerId) -> Reg< bool>
    {
        return !self.Get( in1);
    }

    #[inline]
    pub fn	Nand( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return !( self.And( in1, in2));
    }

    #[inline]
    pub fn	Nor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return !( self.Or( in1, in2));
    }

    #[inline]
    pub fn	Xnor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return !( self.Xor( in1, in2));
    }

    // --- Edge Detection Methods ---

    #[inline]
    pub fn	IsPosedge( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        let  	curVal = self._CurrentVal[idx];
        let  	curX = self._CurrentX[idx];
        let  	pastVal = self._PastVal[idx];
        let  	pastX = self._PastX[idx];
        return ( curVal != pastVal || curX != pastX) && ( curVal && !pastVal && !curX);
    }

    #[inline]
    pub fn	IsNegedge( &self, id: TriggerId) -> bool
    {
        let  	idx = id.0 as usize;
        let  	curVal = self._CurrentVal[idx];
        let  	curX = self._CurrentX[idx];
        let  	pastVal = self._PastVal[idx];
        let  	pastX = self._PastX[idx];
        return ( curVal != pastVal || curX != pastX) && ( !curVal && pastVal && !pastX);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerWadBool for TriggerWad< bool>
{
    #[inline]
    fn	And( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.And( in1, in2);
    }

    #[inline]
    fn	Or( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Or( in1, in2);
    }

    #[inline]
    fn	Xor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Xor( in1, in2);
    }

    #[inline]
    fn	Not( &self, in1: TriggerId) -> Reg< bool>
    {
        return self.Not( in1);
    }

    #[inline]
    fn	Nand( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Nand( in1, in2);
    }

    #[inline]
    fn	Nor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Nor( in1, in2);
    }

    #[inline]
    fn	Xnor( &self, in1: TriggerId, in2: TriggerId) -> Reg< bool>
    {
        return self.Xnor( in1, in2);
    }

    #[inline]
    fn	IsPosedge( &self, id: TriggerId) -> bool
    {
        return self.IsPosedge( id);
    }

    #[inline]
    fn	IsNegedge( &self, id: TriggerId) -> bool
    {
        return self.IsNegedge( id);
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
    pub fn	Matches( &self, signals: &TriggerWad< bool>, id: TriggerId) -> bool
    {
        if self.IsNone() || !signals.IsEdge( id) {
            return false;
        }
        if self.Contains( Self::POS_EDGE) && signals.IsPosedge( id) {
            return true;
        }
        if self.Contains( Self::NEG_EDGE) && signals.IsNegedge( id) {
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
