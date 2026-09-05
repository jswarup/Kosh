//-- dpi.rs -------------------------------------------------------------------------------------------------------------------------

use	crate::{
    rube::{
        engine::SimEngine,
        port::PortId,
        reg::Reg,
    },
    silo::{ Stash, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Standard interface for VPI/DPI co-simulation with external HDL simulators
pub trait ISimulationSocket
{
    fn	Connect( &mut self, hostName: &str, port: u16) -> bool;
    fn	SendUpdate( &mut self, portId: PortId, val: Reg);
    fn	ReceiveUpdates( &mut self, engine: &mut SimEngine);
    fn	AdvanceCycle( &mut self, engine: &mut SimEngine) -> bool;
    fn	Disconnect( &mut self);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// A simple reference socket that buffers updates locally
pub struct LocalSocket
{
    pub _OutboundUpdates: Stash< ( PortId, Reg)>,
    pub _InboundUpdates:  Stash< ( PortId, Reg)>,
    pub _IsConnected:     bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for LocalSocket
{
    fn	default() -> Self
    {
        Self::New()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl LocalSocket
{
    pub fn	New() -> Self
    {
        return Self {
            _OutboundUpdates: Stash::New(),
            _InboundUpdates:  Stash::New(),
            _IsConnected:     false,
        };
    }

    pub fn	QueueInbound( &mut self, portId: PortId, val: Reg)
    {
        self._InboundUpdates.Push( ( portId, val));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ISimulationSocket for LocalSocket
{
    fn	Connect( &mut self, _hostName: &str, _port: u16) -> bool
    {
        self._IsConnected = true;
        return true;
    }

    fn	SendUpdate( &mut self, portId: PortId, val: Reg)
    {
        if self._IsConnected {
            self._OutboundUpdates.Push( ( portId, val));
        }
    }

    fn	ReceiveUpdates( &mut self, engine: &mut SimEngine)
    {
        for i in 0..self._InboundUpdates.Size().AsUsize() {
            let  	( pId, val) = self._InboundUpdates[U32( i as u32)];
            engine.SetPortValue( pId, val);
        }
        self._InboundUpdates = Stash::New();
    }

    fn	AdvanceCycle( &mut self, engine: &mut SimEngine) -> bool
    {
        if !self._IsConnected {
            return false;
        }

        self.ReceiveUpdates( engine);
        engine.Drive();

        // After drive, we would typically collect outputs and push them
        // to `_OutboundUpdates`, but for simplicity, they are queued directly via SendUpdate.
        return true;
    }

    fn	Disconnect( &mut self)
    {
        self._IsConnected = false;
        self._InboundUpdates = Stash::New();
        self._OutboundUpdates = Stash::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

