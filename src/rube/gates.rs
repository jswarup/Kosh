/// Logic Gate components
///
/// Direct Rust equivalent of `Fr_NandGate`, `Fr_AndGate`, `Fr_OrGate`, `Fr_NotGate`, `Fr_XorGate`.

use	crate::rube::trigger::TriggerId;
use	crate::rube::sim_context::{ActionId, ActionKind, SimContext};
use	crate::rube::trigger::TriggerSense;

/// 2-Input NAND Gate ( `Fr_NandGate`)
#[derive( Clone, Debug)]
pub struct NandGate
{
    _Name: String,
    _In1: TriggerId,
    _In2: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

impl NandGate
{
    pub fn	new( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
{
        let  	action = ActionKind::Nand { _In1: in1, _In2: in2, _Out: out };
        let  	action_id = ctxt.add_action(
            action,
            &[
                ( in1, TriggerSense::EDGE),
                ( in2, TriggerSense::EDGE),
            ],
        );
        Self {
            _Name: name.to_string(),
            _In1: in1,
            _In2: in2,
            _Out: out,
            _ActionId: action_id,
        }
    }
}

/// 2-Input AND Gate ( `Fr_AndGate`)
#[derive( Clone, Debug)]
pub struct AndGate
{
    _Name: String,
    _In1: TriggerId,
    _In2: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

impl AndGate
{
    pub fn	new( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
{
        let  	action = ActionKind::And { _In1: in1, _In2: in2, _Out: out };
        let  	action_id = ctxt.add_action(
            action,
            &[
                ( in1, TriggerSense::EDGE),
                ( in2, TriggerSense::EDGE),
            ],
        );
        Self {
            _Name: name.to_string(),
            _In1: in1,
            _In2: in2,
            _Out: out,
            _ActionId: action_id,
        }
    }
}

/// 2-Input OR Gate ( `Fr_OrGate`)
#[derive( Clone, Debug)]
pub struct OrGate
{
    _Name: String,
    _In1: TriggerId,
    _In2: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

impl OrGate
{
    pub fn	new( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
{
        let  	action = ActionKind::Or { _In1: in1, _In2: in2, _Out: out };
        let  	action_id = ctxt.add_action(
            action,
            &[
                ( in1, TriggerSense::EDGE),
                ( in2, TriggerSense::EDGE),
            ],
        );
        Self {
            _Name: name.to_string(),
            _In1: in1,
            _In2: in2,
            _Out: out,
            _ActionId: action_id,
        }
    }
}

/// 1-Input NOT / Inverter Gate ( `Fr_NotGate`)
#[derive( Clone, Debug)]
pub struct NotGate
{
    _Name: String,
    _InSig: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

impl NotGate
{
    pub fn	new( ctxt: &mut SimContext, name: &str, in_sig: TriggerId, out: TriggerId) -> Self
{
        let  	action = ActionKind::Not { _InSig: in_sig, _Out: out };
        let  	action_id = ctxt.add_action(
            action,
            &[( in_sig, TriggerSense::EDGE)],
        );
        Self {
            _Name: name.to_string(),
            _InSig: in_sig,
            _Out: out,
            _ActionId: action_id,
        }
    }
}

/// 2-Input XOR Gate ( `Fr_XorGate`)
#[derive( Clone, Debug)]
pub struct XorGate
{
    _Name: String,
    _In1: TriggerId,
    _In2: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

impl XorGate
{
    pub fn	new( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
{
        let  	action = ActionKind::Xor { _In1: in1, _In2: in2, _Out: out };
        let  	action_id = ctxt.add_action(
            action,
            &[
                ( in1, TriggerSense::EDGE),
                ( in2, TriggerSense::EDGE),
            ],
        );
        Self {
            _Name: name.to_string(),
            _In1: in1,
            _In2: in2,
            _Out: out,
            _ActionId: action_id,
        }
    }
}
