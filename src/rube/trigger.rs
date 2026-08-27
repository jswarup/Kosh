//-- trigger.rs -------------------------------------------------------------------------------------------------------------------
use	crate::{
    rube::reg::Reg,
    silo::U32,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type TriggerId = U32;

//---------------------------------------------------------------------------------------------------------------------------------

/// Hot temporal state cell for a single trigger in SoA layout.
#[derive( Clone, Debug)]
pub struct TriggerWad
{
    pub _PastVals: crate::silo::Buff< Reg>,
    pub _CurrentVals: crate::silo::Buff< Reg>,
    pub _FutureVals: crate::silo::Buff< Reg>,
    pub _SubscriberSpans: crate::silo::Buff< crate::silo::USeg>,
    pub _Subscribers: crate::silo::Buff< TriggerSubscriber>,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug)]
pub struct TriggerSubscriber
{
    pub _ModIndex: crate::silo::U32,
    pub _Sensitivity: crate::rube::port::PortSensitivity,
}

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
    fn	IsSensitive( &self, idx: TriggerId, sens: crate::rube::port::PortSensitivity) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TriggerWad
{
    #[inline]
    pub fn	New(
        pastVals: crate::silo::Buff< Reg>,
        currentVals: crate::silo::Buff< Reg>,
        futureVals: crate::silo::Buff< Reg>,
        subscriberSpans: crate::silo::Buff< crate::silo::USeg>,
        subscribers: crate::silo::Buff< TriggerSubscriber>,
    ) -> Self
    {
        return Self {
            _PastVals: pastVals,
            _CurrentVals: currentVals,
            _FutureVals: futureVals,
            _SubscriberSpans: subscriberSpans,
            _Subscribers: subscribers,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ITriggerWad for TriggerWad
{
    #[inline]
    fn	Size( &self) -> U32
    {
        return self._PastVals.Size();
    }

    #[inline]
    fn	Advance( &mut self, idx: TriggerId) -> ( Reg, Reg)
    {
        let  	past = self._CurrentVals[idx];
        let  	current = self._FutureVals[idx];
        self._PastVals[idx] = past;
        self._CurrentVals[idx] = current;
        return ( past, current);
    }

    #[inline]
    fn	AdvanceAll( &mut self)
    {
        crate::silo::USeg::New( U32::_0, self.Size()).Traverse( |i| {
            let  	past = self._CurrentVals[i];
            let  	current = self._FutureVals[i];
            self._PastVals[i] = past;
            self._CurrentVals[i] = current;
        });
    }

    #[inline]
    fn	Init( &mut self, idx: TriggerId, val: Reg)
    {
        self._PastVals[idx] = val;
        self._CurrentVals[idx] = val;
        self._FutureVals[idx] = val;
    }

    #[inline]
    fn	IsEdge( &self, idx: TriggerId) -> bool
    {
        let  	past = self._PastVals[idx];
        let  	current = self._CurrentVals[idx];
        return past._Val != current._Val || past._X != current._X;
    }

    #[inline]
    fn	IsPosedge( &self, idx: TriggerId) -> bool
    {
        return self._PastVals[idx].IsFalse() && self._CurrentVals[idx].IsTrue();
    }

    #[inline]
    fn	IsNegedge( &self, idx: TriggerId) -> bool
    {
        return self._PastVals[idx].IsTrue() && self._CurrentVals[idx].IsFalse();
    }

    #[inline]
    fn	Past( &self, idx: TriggerId) -> Reg
    {
        return self._PastVals[idx];
    }

    #[inline]
    fn	Current( &self, idx: TriggerId) -> Reg
    {
        return self._CurrentVals[idx];
    }

    #[inline]
    fn	Future( &self, idx: TriggerId) -> Reg
    {
        return self._FutureVals[idx];
    }

    #[inline]
    fn	SetFuture( &mut self, idx: TriggerId, val: Reg)
    {
        self._FutureVals[idx] = val;
    }

    #[inline]
    fn	SetImmediate( &mut self, idx: TriggerId, val: Reg)
    {
        self._CurrentVals[idx] = val;
        self._FutureVals[idx] = val;
    }

    #[inline]
    fn	IsSensitive( &self, idx: TriggerId, sens: crate::rube::port::PortSensitivity) -> bool
    {
        match sens {
            crate::rube::port::PortSensitivity::Up => self.IsPosedge( idx),
            crate::rube::port::PortSensitivity::Down => self.IsNegedge( idx),
            crate::rube::port::PortSensitivity::Any => self.IsEdge( idx),
            crate::rube::port::PortSensitivity::None => false,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
