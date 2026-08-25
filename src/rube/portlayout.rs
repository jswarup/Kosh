//-- portlayout.rs -------------------------------------------------------------------------------------------------------------------
use	crate::{
    rube::trigger::TriggerId,
    silo::{ EdgeBroadcast, EdgeConnect, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IPortLayout
{
    fn	PortConn( &self) -> &EdgeConnect;
    fn	PortConnMut( &mut self) -> &mut EdgeConnect;
    fn	PortCast( &self) -> &EdgeBroadcast;
    fn	PortCastMut( &mut self) -> &mut EdgeBroadcast;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct PortLayout
{
    pub _PortConn: EdgeConnect,
    pub _PortCast: EdgeBroadcast,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PortLayout
{
    pub fn	New( mxVert: U32) -> Self
    {
        Self {
            _PortConn: EdgeConnect::New(),
            _PortCast: EdgeBroadcast::New( mxVert),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IPortLayout for PortLayout
{
    fn	PortConn( &self) -> &EdgeConnect
    {
        return &self._PortConn;
    }

    fn	PortConnMut( &mut self) -> &mut EdgeConnect
    {
        return &mut self._PortConn;
    }

    fn	PortCast( &self) -> &EdgeBroadcast
    {
        return &self._PortCast;
    }

    fn	PortCastMut( &mut self) -> &mut EdgeBroadcast
    {
        return &mut self._PortCast;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub struct Port
{
    pub _Name: String,
    pub _Trigger: TriggerId,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IPort
{
    fn	Name( &self) -> &str;
    fn	Trigger( &self) -> TriggerId;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Port
{
    pub fn	New( name: impl Into< String>, trigger: TriggerId) -> Self
    {
        Self {
            _Name: name.into(),
            _Trigger: trigger,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IPort for Port
{
    fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    fn	Trigger( &self) -> TriggerId
    {
        return self._Trigger;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
