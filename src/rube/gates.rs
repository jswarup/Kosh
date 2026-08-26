//-- gates.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::{
    rube::{
        modlayout::IModule,
        sim_context::{ ActionId, ActionKind, SimContext },
        trigger::{ TriggerId, TriggerSense },
    },
    silo::U32,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INandGate
{
    fn	In1( &self) -> TriggerId;
    fn	In2( &self) -> TriggerId;
    fn	Out( &self) -> TriggerId;
    fn	Name( &self) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

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

//---------------------------------------------------------------------------------------------------------------------------------

impl NandGate
{
    pub fn	New( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
    {
        let  	action = ActionKind::Nand { _In1: in1, _In2: in2, _Out: out };
        let  	actionId = ctxt.AddAction(
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
            _ActionId: actionId,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl INandGate for NandGate
{
    #[inline]
    fn	In1( &self) -> TriggerId
    {
        return self._In1;
    }

    #[inline]
    fn	In2( &self) -> TriggerId
    {
        return self._In2;
    }

    #[inline]
    fn	Out( &self) -> TriggerId
    {
        return self._Out;
    }

    #[inline]
    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IAndGate
{
    fn	In1( &self) -> TriggerId;
    fn	In2( &self) -> TriggerId;
    fn	Out( &self) -> TriggerId;
    fn	Name( &self) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input AND Gate ( `Fr_AndGate`)
#[derive( Clone, Debug)]
pub struct AndGate
{
    _ModuleId: U32,
    _Name: String,
    _In1: TriggerId,
    _In2: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AndGate
{
    pub fn	New( ctxt: &mut SimContext, moduleId: U32, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
    {
        let  	action = ActionKind::And { _In1: in1, _In2: in2, _Out: out };
        let  	actionId = ctxt.AddAction(
            action,
            &[
                ( in1, TriggerSense::EDGE),
                ( in2, TriggerSense::EDGE),
            ],
        );
        Self {
            _ModuleId: moduleId,
            _Name: name.to_string(),
            _In1: in1,
            _In2: in2,
            _Out: out,
            _ActionId: actionId,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IAndGate for AndGate
{
    #[inline]
    fn	In1( &self) -> TriggerId
    {
        return self._In1;
    }

    #[inline]
    fn	In2( &self) -> TriggerId
    {
        return self._In2;
    }

    #[inline]
    fn	Out( &self) -> TriggerId
    {
        return self._Out;
    }

    #[inline]
    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for AndGate
{
    fn	ModuleId( &self) -> U32
    {
        return self._ModuleId;
    }

    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IOrGate
{
    fn	In1( &self) -> TriggerId;
    fn	In2( &self) -> TriggerId;
    fn	Out( &self) -> TriggerId;
    fn	Name( &self) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

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

//---------------------------------------------------------------------------------------------------------------------------------

impl OrGate
{
    pub fn	New( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
    {
        let  	action = ActionKind::Or { _In1: in1, _In2: in2, _Out: out };
        let  	actionId = ctxt.AddAction(
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
            _ActionId: actionId,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IOrGate for OrGate
{
    #[inline]
    fn	In1( &self) -> TriggerId
    {
        return self._In1;
    }

    #[inline]
    fn	In2( &self) -> TriggerId
    {
        return self._In2;
    }

    #[inline]
    fn	Out( &self) -> TriggerId
    {
        return self._Out;
    }

    #[inline]
    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INotGate
{
    fn	InSig( &self) -> TriggerId;
    fn	Out( &self) -> TriggerId;
    fn	Name( &self) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Input NOT / Inverter Gate ( `Fr_NotGate`)
#[derive( Clone, Debug)]
pub struct NotGate
{
    _Name: String,
    _InSig: TriggerId,
    _Out: TriggerId,
    _ActionId: ActionId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NotGate
{
    pub fn	New( ctxt: &mut SimContext, name: &str, inSig: TriggerId, out: TriggerId) -> Self
    {
        let  	action = ActionKind::Not { _InSig: inSig, _Out: out };
        let  	actionId = ctxt.AddAction(
            action,
            &[( inSig, TriggerSense::EDGE)],
        );
        Self {
            _Name: name.to_string(),
            _InSig: inSig,
            _Out: out,
            _ActionId: actionId,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl INotGate for NotGate
{
    #[inline]
    fn	InSig( &self) -> TriggerId
    {
        return self._InSig;
    }

    #[inline]
    fn	Out( &self) -> TriggerId
    {
        return self._Out;
    }

    #[inline]
    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IXorGate
{
    fn	In1( &self) -> TriggerId;
    fn	In2( &self) -> TriggerId;
    fn	Out( &self) -> TriggerId;
    fn	Name( &self) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

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

//---------------------------------------------------------------------------------------------------------------------------------

impl XorGate
{
    pub fn	New( ctxt: &mut SimContext, name: &str, in1: TriggerId, in2: TriggerId, out: TriggerId) -> Self
    {
        let  	action = ActionKind::Xor { _In1: in1, _In2: in2, _Out: out };
        let  	actionId = ctxt.AddAction(
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
            _ActionId: actionId,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXorGate for XorGate
{
    #[inline]
    fn	In1( &self) -> TriggerId
    {
        return self._In1;
    }

    #[inline]
    fn	In2( &self) -> TriggerId
    {
        return self._In2;
    }

    #[inline]
    fn	Out( &self) -> TriggerId
    {
        return self._Out;
    }

    #[inline]
    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
