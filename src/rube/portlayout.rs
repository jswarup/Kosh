//-- portlayout.rs -------------------------------------------------------------------------------------------------------------------
use	crate::{
    rube::trigger::TriggerId,
    silo::{ EdgeBroadcast, EdgeConnect, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub struct TopologyPort
{
    pub _Name: String,
    pub _Trigger: TriggerId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TopologyPort
{
    pub fn	New( name: impl Into< String>, trigger: TriggerId) -> Self
    {
        return Self {
            _Name: name.into(),
            _Trigger: trigger,
        };
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }

    #[inline]
    pub fn	Trigger( &self) -> TriggerId
    {
        return self._Trigger;
    }
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
        return Self {
            _PortConn: EdgeConnect::New(),
            _PortCast: EdgeBroadcast::New( mxVert),
        };
    }

    #[inline]
    pub fn	PortConn( &self) -> &EdgeConnect
    {
        return &self._PortConn;
    }

    #[inline]
    pub fn	PortConnMut( &mut self) -> &mut EdgeConnect
    {
        return &mut self._PortConn;
    }

    #[inline]
    pub fn	PortCast( &self) -> &EdgeBroadcast
    {
        return &self._PortCast;
    }

    #[inline]
    pub fn	PortCastMut( &mut self) -> &mut EdgeBroadcast
    {
        return &mut self._PortCast;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
