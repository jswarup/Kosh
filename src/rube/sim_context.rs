//-- sim_context.rs -----------------------------------------------------------------------------------------------------------------
use	std::collections::BTreeSet;
use	std::sync::Arc;
use	crate::{
    rube::{
        reg::Reg,
        trigger::{ TriggerId, TriggerSense, TriggerWad },
    },
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
    Not { _InSig: TriggerId, _Out: TriggerId },
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
    pending_actions: BTreeSet< ActionId>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for SimContext
{
    fn	default() -> Self
    {
        Self::new()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SimContext
{
    pub fn	new() -> Self
    {
        Self {
            _Triggers: TriggerWad::new(),
            _Actions: Vec::new(),
            _Sensitivities: Vec::new(),
            _ArmedTriggers: BTreeSet::new(),
            pending_actions: BTreeSet::new(),
        }
    }

    /// Add a new trigger ( signal) to the simulation context
    #[inline]
    pub fn	add_trigger( &mut self, name: &str, initial: Reg) -> TriggerId
    {
        return self._Triggers.add( name, initial);
    }

    /// Query the current value of a trigger
    #[inline]
    pub fn	get_value( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.get( id);
    }

    /// Query the future value of a trigger ( if staged)
    #[inline]
    pub fn	get_future_value( &self, id: TriggerId) -> Reg
    {
        return self._Triggers.get_future( id);
    }

    /// Query trigger name
    #[inline]
    pub fn	get_trigger_name( &self, id: TriggerId) -> &str
    {
        return self._Triggers.name( id);
    }

    /// Initialize a trigger value directly without scheduling events
    #[inline]
    pub fn	init_value( &mut self, id: TriggerId, val: Reg)
    {
        self._Triggers.init_value( id, val);
        self._ArmedTriggers.remove( &id);
    }

    /// Set a new future value on a trigger, arming it if changed
    #[inline]
    pub fn	set_value( &mut self, id: TriggerId, val: Reg)
    {
        if self._Triggers.set_future_value( id, val) {
            self._ArmedTriggers.insert( id);
        }
    }

    /// Register a gate or action along with its input sensitivities
    pub fn	add_action( &mut self, action: ActionKind, sensitivities: &[( TriggerId, TriggerSense)]) -> ActionId
    {
        let  	act_id = self._Actions.len();
        self._Actions.push( action);
        for &( trigger_id, sense) in sensitivities {
            self._Sensitivities.push( Sensitivity {
                _TriggerId: trigger_id,
                _Sense: sense,
                _ActionId: act_id,
            });
        }
        return act_id;
    }

    /// Perform delta-cycle propagation until all triggers reach steady state
    pub fn	drive( &mut self) -> usize
    {
        let  	mut delta_cycles = 0;
        const MAX_DELTA_CYCLES: usize = 10_000;

        loop {
            if self._ArmedTriggers.is_empty() && self.pending_actions.is_empty() {
                break;
            }

            delta_cycles += 1;
            if delta_cycles > MAX_DELTA_CYCLES {
                eprintln!( "[SimContext::drive] Warning: Maximum delta cycles ( {}) reached, possible oscillation/metastability.", MAX_DELTA_CYCLES);
                break;
            }

            // Step 1: Advance armed triggers and collect triggered actions
            let  	armed: Vec< TriggerId> = self._ArmedTriggers.iter().copied().collect();
            self._ArmedTriggers.clear();

            for trigger_id in armed {
                if self._Triggers.is_armed( trigger_id) {
                    self._Triggers.advance( trigger_id);

                    for s in &self._Sensitivities {
                        if s._TriggerId == trigger_id && s._Sense.matches( &self._Triggers, trigger_id) {
                            self.pending_actions.insert( s._ActionId);
                        }
                    }
                }
            }

            // Step 2: Fire all pending actions
            let  	pending: Vec< ActionId> = self.pending_actions.iter().copied().collect();
            self.pending_actions.clear();

            for act_id in pending {
                self.fire_action( act_id);
            }
        }

        return delta_cycles;
    }

    /// Internal evaluation of an action
    fn	fire_action( &mut self, act_id: ActionId)
    {
        let  	action = self._Actions[act_id].clone();
        match action {
            ActionKind::Nand { _In1: in1, _In2: in2, _Out: out } => {
                self.set_value( out, self._Triggers.nand( in1, in2));
            }
            ActionKind::And { _In1: in1, _In2: in2, _Out: out } => {
                self.set_value( out, self._Triggers.and( in1, in2));
            }
            ActionKind::Or { _In1: in1, _In2: in2, _Out: out } => {
                self.set_value( out, self._Triggers.or( in1, in2));
            }
            ActionKind::Not { _InSig: in_sig, _Out: out } => {
                self.set_value( out, self._Triggers.not( in_sig));
            }
            ActionKind::Xor { _In1: in1, _In2: in2, _Out: out } => {
                self.set_value( out, self._Triggers.xor( in1, in2));
            }
            ActionKind::Nor { _In1: in1, _In2: in2, _Out: out } => {
                self.set_value( out, self._Triggers.nor( in1, in2));
            }
            ActionKind::Xnor { _In1: in1, _In2: in2, _Out: out } => {
                self.set_value( out, self._Triggers.xnor( in1, in2));
            }
            ActionKind::Custom( callback) => {
                callback( self);
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
