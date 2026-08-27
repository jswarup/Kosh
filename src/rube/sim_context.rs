//-- sim_context.rs -----------------------------------------------------------------------------------------------------------------
use	std::sync::Arc;
use	crate::{
    rube::{
        reg::Reg,
        trigger::{ TriggerId, TriggerSense, TriggerWad },
    },
    silo::{ Buff, Stash, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type ActionId = usize;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub enum ActionKind
{
    Nand { _In1: TriggerId, _In2: TriggerId, _Out: TriggerId },
    And { _In1: TriggerId, _In2: TriggerId, _Out: TriggerId },
    Or { _In1: TriggerId, _In2: TriggerId, _Out: TriggerId },
    Not { _In: TriggerId, _Out: TriggerId },
    Xor { _In1: TriggerId, _In2: TriggerId, _Out: TriggerId },
    Nor { _In1: TriggerId, _In2: TriggerId, _Out: TriggerId },
    Xnor { _In1: TriggerId, _In2: TriggerId, _Out: TriggerId },
    Custom( Arc< dyn Fn( &mut SimContext) + Send + Sync>),
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SimError
{
    Oscillation,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug)]
pub struct TriggerTarget
{
    pub _Sense: TriggerSense,
    pub _ActionId: ActionId,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Sensitivity
{
    pub _TriggerId: TriggerId,
    pub _Sense: TriggerSense,
    pub _ActionId: ActionId,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct SimContext
{
    pub _Triggers: TriggerWad,
    _Actions: Stash< ActionKind>,
    _TriggerSensitivities: Buff< Stash< TriggerTarget>>,
    _ArmedMask: Stash< u64>,
    _ArmedQueue: Stash< TriggerId>,
    _PendingMask: Stash< u64>,
    _PendingQueue: Stash< ActionId>,
    _CurrArmed: Stash< TriggerId>,
    _CurrPending: Stash< ActionId>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for SimContext
{
    fn	default() -> Self
    {
        return Self::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimContext
{
    pub fn	New() -> Self
    {
        return Self {
            _Triggers: TriggerWad::New(),
            _Actions: Stash::New(),
            _TriggerSensitivities: Buff::New(),
            _ArmedMask: Stash::New(),
            _ArmedQueue: Stash::New(),
            _PendingMask: Stash::New(),
            _PendingQueue: Stash::New(),
            _CurrArmed: Stash::New(),
            _CurrPending: Stash::New(),
        };
    }

    /// Add a new trigger to the simulation context
    #[inline]
    pub fn	AddTrigger( &mut self, name: &str, initial: Reg) -> TriggerId
    {
        let  	id = self._Triggers.Add( name, initial);
        let  	idx = usize::from( id);
        if idx >= self._TriggerSensitivities.len() {
            let  	newSize = U32( ( idx + 1) as u32);
            self._TriggerSensitivities.Resize( newSize, |_| Stash::New());
        }
        let  	word = idx / 64;
        while word >= self._ArmedMask.Size().AsUsize() {
            self._ArmedMask.Push( 0);
        }
        return id;
    }

    /// Query the current value of a trigger
    #[inline]
    pub fn	GetValue( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.Get( id);
    }

    /// Query the future value of a trigger
    #[inline]
    pub fn	GetFutureValue( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.GetFuture( id);
    }

    /// Initialize a trigger value directly without scheduling events
    #[inline]
    pub fn	InitValue( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers.InitValue( id, val);
        self._DisarmTrigger( id);
    }

    /// Set a new future value on a trigger, arming it if changed
    #[inline]
    pub fn	SetValue( &mut self, id: TriggerId, val: Reg)
    {
        if self._Triggers.SetFutureValue( id, val) {
            self._ArmTrigger( id);
        }
    }

    #[inline]
    fn	_ArmTrigger( &mut self, id: TriggerId)
    {
        let  	idx = usize::from( id);
        let  	word = idx / 64;
        let  	bit = 1u64 << ( idx % 64);
        while word >= self._ArmedMask.Size().AsUsize() {
            self._ArmedMask.Push( 0);
        }
        if ( self._ArmedMask.Slice()[word] & bit) == 0 {
            self._ArmedMask.SliceMut()[word] |= bit;
            self._ArmedQueue.Push( id);
        }
    }

    #[inline]
    fn	_DisarmTrigger( &mut self, id: TriggerId)
    {
        let  	idx = usize::from( id);
        let  	word = idx / 64;
        let  	bit = 1u64 << ( idx % 64);
        if word < self._ArmedMask.Size().AsUsize() {
            self._ArmedMask.SliceMut()[word] &= !bit;
        }
    }

    #[inline]
    fn	_QueueAction( &mut self, actId: ActionId)
    {
        let  	word = actId / 64;
        let  	bit = 1u64 << ( actId % 64);
        while word >= self._PendingMask.Size().AsUsize() {
            self._PendingMask.Push( 0);
        }
        if ( self._PendingMask.Slice()[word] & bit) == 0 {
            self._PendingMask.SliceMut()[word] |= bit;
            self._PendingQueue.Push( actId);
        }
    }

    /// Register a gate or action along with its input sensitivities
    pub fn	AddAction( &mut self, action: ActionKind, sensitivities: &[( TriggerId, TriggerSense)]) -> ActionId
    {
        let  	actId = self._Actions.Size().AsUsize();
        self._Actions.Push( action);
        for &( triggerId, sense) in sensitivities {
            let  	idx = usize::from( triggerId);
            if idx >= self._TriggerSensitivities.len() {
                let  	newSize = U32( ( idx + 1) as u32);
                self._TriggerSensitivities.Resize( newSize, |_| Stash::New());
            }
            self._TriggerSensitivities[idx].Push( TriggerTarget {
                _Sense: sense,
                _ActionId: actId,
            });
        }
        let  	word = actId / 64;
        while word >= self._PendingMask.Size().AsUsize() {
            self._PendingMask.Push( 0);
        }
        return actId;
    }

    /// Perform delta-cycle propagation until all triggers reach steady state
    pub fn	Drive( &mut self) -> Result< usize, SimError>
    {
        let  	mut deltaCycles = 0;
        const MAX_DELTA_CYCLES: usize = 10_000;

        loop {
            if self._ArmedQueue.Size().0 == 0 && self._PendingQueue.Size().0 == 0 {
                break;
            }

            deltaCycles += 1;
            if deltaCycles > MAX_DELTA_CYCLES {
                return Err( SimError::Oscillation);
            }

            // Step 1: Advance armed triggers and collect triggered actions
            std::mem::swap( &mut self._ArmedQueue, &mut self._CurrArmed);

            let  	numArmed = self._CurrArmed.Size().AsUsize();
            for i in 0..numArmed {
                let  	triggerId = self._CurrArmed.Slice()[i];
                let  	idx = usize::from( triggerId);
                let  	word = idx / 64;
                let  	bit = 1u64 << ( idx % 64);
                if word < self._ArmedMask.Size().AsUsize() && ( self._ArmedMask.Slice()[word] & bit) != 0 {
                    self._ArmedMask.SliceMut()[word] &= !bit;

                    if self._Triggers.IsArmed( triggerId) {
                        self._Triggers.Advance( triggerId);

                        if idx < self._TriggerSensitivities.len() {
                            let  	numTargets = self._TriggerSensitivities[idx].Size().AsUsize();
                            for t in 0..numTargets {
                                let  	target = self._TriggerSensitivities[idx].Slice()[t];
                                if target._Sense.Matches( &self._Triggers, triggerId) {
                                    self._QueueAction( target._ActionId);
                                }
                            }
                        }
                    }
                }
            }
            self._CurrArmed.Clear();

            // Step 2: Fire all pending actions
            std::mem::swap( &mut self._PendingQueue, &mut self._CurrPending);

            let  	numPending = self._CurrPending.Size().AsUsize();
            for i in 0..numPending {
                let  	actId = self._CurrPending.Slice()[i];
                let  	word = actId / 64;
                let  	bit = 1u64 << ( actId % 64);
                if word < self._PendingMask.Size().AsUsize() {
                    self._PendingMask.SliceMut()[word] &= !bit;
                }
                self._FireAction( actId);
            }
            self._CurrPending.Clear();
        }

        return Ok( deltaCycles);
    }

    /// Internal evaluation of an action
    fn	_FireAction( &mut self, actId: ActionId)
    {
        match self._Actions.Slice()[actId] {
            ActionKind::Nand { _In1: in1, _In2: in2, _Out: out } => self.SetValue( out, self._Triggers.Nand( in1, in2)),
            ActionKind::And { _In1: in1, _In2: in2, _Out: out } => self.SetValue( out, self._Triggers.And( in1, in2)),
            ActionKind::Or { _In1: in1, _In2: in2, _Out: out } => self.SetValue( out, self._Triggers.Or( in1, in2)),
            ActionKind::Not { _In: inTrig, _Out: out } => self.SetValue( out, self._Triggers.Not( inTrig)),
            ActionKind::Xor { _In1: in1, _In2: in2, _Out: out } => self.SetValue( out, self._Triggers.Xor( in1, in2)),
            ActionKind::Nor { _In1: in1, _In2: in2, _Out: out } => self.SetValue( out, self._Triggers.Nor( in1, in2)),
            ActionKind::Xnor { _In1: in1, _In2: in2, _Out: out } => self.SetValue( out, self._Triggers.Xnor( in1, in2)),
            ActionKind::Custom( ref callback) => {
                let  	cb = Arc::clone( callback);
                cb( self);
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
