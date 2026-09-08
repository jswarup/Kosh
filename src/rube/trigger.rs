//-- trigger.rs -------------------------------------------------------------------------------------------------------------------
use	crate::{
    rube::reg::Reg,
    silo::{ arr::IArr, Buff, U8, U16, U32, U64, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type TriggerId = U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ITriggerVal: Copy + Default + PartialEq + Send + Sync + 'static
{
    fn	ToU64( self) -> U64;
    fn	FromU64( val: U64) -> Self;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for U64
{
    #[inline]
    fn	ToU64( self) -> U64 { self }
    #[inline]
    fn	FromU64( val: U64) -> Self { val }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for U32
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self.0 as u64) }
    #[inline]
    fn	FromU64( val: U64) -> Self { U32( val.0 as u32) }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for U16
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self.0 as u64) }
    #[inline]
    fn	FromU64( val: U64) -> Self { U16( val.0 as u16) }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for U8
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self.0 as u64) }
    #[inline]
    fn	FromU64( val: U64) -> Self { U8( val.0 as u8) }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for u64
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self) }
    #[inline]
    fn	FromU64( val: U64) -> Self { val.0 }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for u32
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self as u64) }
    #[inline]
    fn	FromU64( val: U64) -> Self { val.0 as u32 }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for u16
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self as u64) }
    #[inline]
    fn	FromU64( val: U64) -> Self { val.0 as u16 }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for u8
{
    #[inline]
    fn	ToU64( self) -> U64 { U64( self as u64) }
    #[inline]
    fn	FromU64( val: U64) -> Self { val.0 as u8 }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for bool
{
    #[inline]
    fn	ToU64( self) -> U64 { if self { U64::_1 } else { U64::_0 } }
    #[inline]
    fn	FromU64( val: U64) -> Self { ( val.0 & 1) != 0 }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerVal for Reg
{
    #[inline]
    fn	ToU64( self) -> U64 { self._Val }
    #[inline]
    fn	FromU64( val: U64) -> Self { Reg { _Val: val, _X: false, _I: false } }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub const PAST_X:    u8 = 1 << 0;
pub const PAST_I:    u8 = 1 << 1;
pub const PAST_MASK: u8 = 0b0000_0011;

pub const CURR_X:    u8 = 1 << 2;
pub const CURR_I:    u8 = 1 << 3;
pub const CURR_MASK: u8 = 0b0000_1100;

pub const FUTR_X:    u8 = 1 << 4;
pub const FUTR_I:    u8 = 1 << 5;
pub const FUTR_MASK: u8 = 0b0011_0000;

//---------------------------------------------------------------------------------------------------------------------------------

/// Hot temporal state cell for triggers in 4-Buff SoA layout.
#[derive( Clone, Debug)]
pub struct TriggerWad< T: ITriggerVal = U64>
{
    pub _PastVals:        Buff< T>,
    pub _CurrentVals:     Buff< T>,
    pub _FutureVals:      Buff< T>,
    pub _Flags:           Buff< U8>,
    pub _SubscriberSpans: Buff< USeg>,
    pub _Subscribers:     Buff< TriggerSubscriber>,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type TriggerSubscriber = U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ITriggerWad
{
    fn	Size( &self) -> U32;
    fn	Advance( &mut self, idx: TriggerId) -> ( Reg, Reg);
    fn	AdvanceAll( &mut self);
    fn	Init( &mut self, idx: TriggerId, val: Reg);
    fn	IsEdge( &self, idx: TriggerId) -> bool;
    fn	IsPosedge( &self, idx: TriggerId) -> bool;
    fn	IsNegedge( &self, idx: TriggerId) -> bool;
    fn	Past( &self, idx: TriggerId) -> Reg;
    fn	Current( &self, idx: TriggerId) -> Reg;
    fn	Future( &self, idx: TriggerId) -> Reg;
    fn	SetFuture( &mut self, idx: TriggerId, val: Reg);
    fn	SetImmediate( &mut self, idx: TriggerId, val: Reg);
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: ITriggerVal> TriggerWad< T>
{
    #[inline]
    pub fn	New(
        pastVals: Buff< T>,
        currentVals: Buff< T>,
        futureVals: Buff< T>,
        flags: Buff< U8>,
        subscriberSpans: Buff< USeg>,
        subscribers: Buff< TriggerSubscriber>,
    ) -> Self
    {
        return Self {
            _PastVals:        pastVals,
            _CurrentVals:     currentVals,
            _FutureVals:      futureVals,
            _Flags:           flags,
            _SubscriberSpans: subscriberSpans,
            _Subscribers:     subscribers,
        };
    }

    #[inline]
    pub fn	PastVal( &self, idx: TriggerId) -> T
    {
        return self._PastVals[idx];
    }

    #[inline]
    pub fn	CurrentVal( &self, idx: TriggerId) -> T
    {
        return self._CurrentVals[idx];
    }

    #[inline]
    pub fn	FutureVal( &self, idx: TriggerId) -> T
    {
        return self._FutureVals[idx];
    }

    #[inline]
    pub fn	SetFutureVal( &mut self, idx: TriggerId, val: T)
    {
        self._FutureVals[idx] = val;
        self._Flags[idx] = U8( self._Flags[idx].0 & !FUTR_MASK );
    }

    #[inline]
    pub fn	SetImmediateVal( &mut self, idx: TriggerId, val: T)
    {
        self._CurrentVals[idx] = val;
        self._FutureVals[idx] = val;
        self._Flags[idx] = U8( self._Flags[idx].0 & !( CURR_MASK | FUTR_MASK ) );
    }

    #[inline]
    pub fn	Flags( &self, idx: TriggerId) -> U8
    {
        return self._Flags[idx];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: ITriggerVal> ITriggerWad for TriggerWad< T>
{
    #[inline]
    fn	Size( &self) -> U32
    {
        return self._PastVals.Size();
    }

    #[inline]
    fn	Advance( &mut self, idx: TriggerId) -> ( Reg, Reg)
    {
        let  	past = self.Past( idx);
        let  	current = self.Current( idx);
        self._PastVals[idx] = self._CurrentVals[idx];
        self._CurrentVals[idx] = self._FutureVals[idx];
        let  	f = self._Flags[idx].0;
        self._Flags[idx] = U8( ( ( f >> 2) & 0b0000_1111 ) | ( f & 0b0011_0000 ) );
        return ( past, current);
    }

    #[inline]
    fn	AdvanceAll( &mut self)
    {
        self._PastVals.Arr().CopyFrom( &self._CurrentVals.Arr());
        self._CurrentVals.Arr().CopyFrom( &self._FutureVals.Arr());
        USeg::New( U32::_0, self.Size()).Traverse( |i| {
            let  	f = self._Flags[i].0;
            self._Flags[i] = U8( ( ( f >> 2) & 0b0000_1111 ) | ( f & 0b0011_0000 ) );
        });
    }

    #[inline]
    fn	Init( &mut self, idx: TriggerId, val: Reg)
    {
        let  	v = T::FromU64( val._Val);
        self._PastVals[idx] = v;
        self._CurrentVals[idx] = v;
        self._FutureVals[idx] = v;
        let  	mut f: u8 = 0;
        if val.IsX() {
            f |= PAST_X | CURR_X | FUTR_X;
        }
        if val.IsI() {
            f |= PAST_I | CURR_I | FUTR_I;
        }
        self._Flags[idx] = U8( f);
    }

    #[inline]
    fn	IsEdge( &self, idx: TriggerId) -> bool
    {
        let  	pastVal = self._PastVals[idx];
        let  	currVal = self._CurrentVals[idx];
        let  	f = self._Flags[idx].0;
        let  	pastFlags = f & PAST_MASK;
        let  	currFlags = ( f >> 2) & PAST_MASK;
        return pastVal != currVal || pastFlags != currFlags;
    }

    #[inline]
    fn	IsPosedge( &self, idx: TriggerId) -> bool
    {
        let  	f = self._Flags[idx].0;
        if ( f & ( PAST_MASK | CURR_MASK ) ) != 0 {
            return false;
        }
        return ( self._PastVals[idx].ToU64().0 & 1) == 0 && ( self._CurrentVals[idx].ToU64().0 & 1) != 0;
    }

    #[inline]
    fn	IsNegedge( &self, idx: TriggerId) -> bool
    {
        let  	f = self._Flags[idx].0;
        if ( f & ( PAST_MASK | CURR_MASK ) ) != 0 {
            return false;
        }
        return ( self._PastVals[idx].ToU64().0 & 1) != 0 && ( self._CurrentVals[idx].ToU64().0 & 1) == 0;
    }

    #[inline]
    fn	Past( &self, idx: TriggerId) -> Reg
    {
        let  	val = self._PastVals[idx].ToU64();
        let  	f = self._Flags[idx].0;
        return Reg {
            _Val: val,
            _X:   ( f & PAST_X) != 0,
            _I:   ( f & PAST_I) != 0,
        };
    }

    #[inline]
    fn	Current( &self, idx: TriggerId) -> Reg
    {
        let  	val = self._CurrentVals[idx].ToU64();
        let  	f = self._Flags[idx].0;
        return Reg {
            _Val: val,
            _X:   ( f & CURR_X) != 0,
            _I:   ( f & CURR_I) != 0,
        };
    }

    #[inline]
    fn	Future( &self, idx: TriggerId) -> Reg
    {
        let  	val = self._FutureVals[idx].ToU64();
        let  	f = self._Flags[idx].0;
        return Reg {
            _Val: val,
            _X:   ( f & FUTR_X) != 0,
            _I:   ( f & FUTR_I) != 0,
        };
    }

    #[inline]
    fn	SetFuture( &mut self, idx: TriggerId, val: Reg)
    {
        self._FutureVals[idx] = T::FromU64( val._Val);
        let  	mut f = self._Flags[idx].0 & !FUTR_MASK;
        if val.IsX() {
            f |= FUTR_X;
        }
        if val.IsI() {
            f |= FUTR_I;
        }
        self._Flags[idx] = U8( f);
    }

    #[inline]
    fn	SetImmediate( &mut self, idx: TriggerId, val: Reg)
    {
        let  	v = T::FromU64( val._Val);
        self._CurrentVals[idx] = v;
        self._FutureVals[idx] = v;
        let  	mut f = self._Flags[idx].0 & !( CURR_MASK | FUTR_MASK );
        if val.IsX() {
            f |= CURR_X | FUTR_X;
        }
        if val.IsI() {
            f |= CURR_I | FUTR_I;
        }
        self._Flags[idx] = U8( f);
    }
}

