//-- sim_context.rs -----------------------------------------------------------------------------------------------------------------
use	std::collections::BTreeSet;
use	std::sync::Arc;
use	crate::rube::{
    reg::Reg,
    trigger::{ TriggerId, TriggerSense, TriggerWad },
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
    _Actions: Vec< ActionKind>,
    _Sensitivities: Vec< Sensitivity>,
    _ArmedTriggers: BTreeSet< TriggerId>,
    _PendingActions: BTreeSet< ActionId>,
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
            _Actions: Vec::new(),
            _Sensitivities: Vec::new(),
            _ArmedTriggers: BTreeSet::new(),
            _PendingActions: BTreeSet::new(),
        };
    }

    /// Add a new trigger ( signal) to the simulation context
    #[inline]
    pub fn	AddTrigger( &mut self, name: &str, initial: Reg) -> TriggerId
    {
        return self._Triggers.Add( name, initial);
    }

    /// Query the current value of a trigger
    #[inline]
    pub fn	GetValue( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.Get( id);
    }

    /// Query the future value of a trigger ( if staged)
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
        self._ArmedTriggers.remove( &id);
    }

    /// Set a new future value on a trigger, arming it if changed
    #[inline]
    pub fn	SetValue( &mut self, id: TriggerId, val: Reg)
    {
        if self._Triggers.SetFutureValue( id, val) {
            self._ArmedTriggers.insert( id);
        }
    }

    /// Register a gate or action along with its input sensitivities
    pub fn	AddAction( &mut self, action: ActionKind, sensitivities: &[( TriggerId, TriggerSense)]) -> ActionId
    {
        let  	actId = self._Actions.len();
        self._Actions.push( action);
        for &( triggerId, sense) in sensitivities {
            self._Sensitivities.push( Sensitivity {
                _TriggerId: triggerId,
                _Sense: sense,
                _ActionId: actId,
            });
        }
        return actId;
    }

    /// Perform delta-cycle propagation until all triggers reach steady state
    pub fn	Drive( &mut self) -> usize
    {
        let  	mut deltaCycles = 0;
        const MAX_DELTA_CYCLES: usize = 10_000;

        loop {
            if self._ArmedTriggers.is_empty() && self._PendingActions.is_empty() {
                break;
            }

            deltaCycles += 1;
            if deltaCycles > MAX_DELTA_CYCLES {
                eprintln!( "[SimContext::Drive] Warning: Maximum delta cycles ( {}) reached, possible oscillation/metastability.", MAX_DELTA_CYCLES);
                break;
            }

            // Step 1: Advance armed triggers and collect triggered actions
            let  	armed: Vec< TriggerId> = self._ArmedTriggers.iter().copied().collect();
            self._ArmedTriggers.clear();

            for triggerId in armed {
                if self._Triggers.IsArmed( triggerId) {
                    self._Triggers.Advance( triggerId);

                    for s in &self._Sensitivities {
                        if s._TriggerId == triggerId && s._Sense.Matches( &self._Triggers, triggerId) {
                            self._PendingActions.insert( s._ActionId);
                        }
                    }
                }
            }

            // Step 2: Fire all pending actions
            let  	pending: Vec< ActionId> = self._PendingActions.iter().copied().collect();
            self._PendingActions.clear();

            for actId in pending {
                self._FireAction( actId);
            }
        }

        return deltaCycles;
    }

    /// Internal evaluation of an action
    fn	_FireAction( &mut self, actId: ActionId)
    {
        let  	action = self._Actions[actId].clone();
        match action {
            ActionKind::Nand { _In1: in1, _In2: in2, _Out: out } => {
                let  	res = self._Triggers.Nand( in1, in2);
                self.SetValue( out, res);
            }
            ActionKind::And { _In1: in1, _In2: in2, _Out: out } => {
                let  	res = self._Triggers.And( in1, in2);
                self.SetValue( out, res);
            }
            ActionKind::Or { _In1: in1, _In2: in2, _Out: out } => {
                let  	res = self._Triggers.Or( in1, in2);
                self.SetValue( out, res);
            }
            ActionKind::Not { _In: inTrig, _Out: out } => {
                let  	res = self._Triggers.Not( inTrig);
                self.SetValue( out, res);
            }
            ActionKind::Xor { _In1: in1, _In2: in2, _Out: out } => {
                let  	res = self._Triggers.Xor( in1, in2);
                self.SetValue( out, res);
            }
            ActionKind::Nor { _In1: in1, _In2: in2, _Out: out } => {
                let  	res = self._Triggers.Nor( in1, in2);
                self.SetValue( out, res);
            }
            ActionKind::Xnor { _In1: in1, _In2: in2, _Out: out } => {
                let  	res = self._Triggers.Xnor( in1, in2);
                self.SetValue( out, res);
            }
            ActionKind::Custom( callback) => {
                callback( self);
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
