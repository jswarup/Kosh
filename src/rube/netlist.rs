//-- netlist.rs ---------------------------------------------------------------------------------------------------------------------

use	crate::{
    rube::{
        layout::LayoutError,
        port::{ PortId, PortType },
        trigger::TriggerId,
    },
    silo::{ Buff, DisjointSet, IDisjointSet, Stash, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct Netlist
{
    pub _Equiv:         DisjointSet,
    pub _Driver:        Stash< PortId>,
    pub _RootTrigger:   Stash< TriggerId>,
    pub _NextTriggerId: U32,
    pub _TriggerTypes:  Stash< PortType>,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INetlist
{
    fn	Grow( &mut self, count: U32);
    fn	FindRoot( &mut self, port: PortId) -> U32;
    fn	Connect( &mut self, driver: PortId, sink: PortId) -> Result< (), LayoutError>;
    fn	DriverOf( &mut self, port: PortId) -> PortId;
    fn	AssignTrigger( &mut self, rootIdx: U32, portType: PortType) -> TriggerId;
    fn	TriggerOf( &mut self, port: PortId) -> TriggerId;
    fn	HasTrigger( &mut self, port: PortId) -> bool;
    fn	BuildPortToTrigger( &mut self) -> Buff< TriggerId>;
    fn	TriggerCount( &self) -> U32;
    fn	TriggerType( &self, trigId: TriggerId) -> PortType;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Netlist
{
    pub fn	New() -> Self
    {
        return Self {
            _Equiv:         DisjointSet::New(),
            _Driver:        Stash::New(),
            _RootTrigger:   Stash::New(),
            _NextTriggerId: U32::_0,
            _TriggerTypes:  Stash::New(),
        };
    }

    #[inline]
    pub fn	FindRootConst( &self, port: PortId) -> U32
    {
        return self._Equiv.FindConst( port.Index());
    }

    #[inline]
    pub fn	HasTriggerConst( &self, port: PortId) -> bool
    {
        let  	root = self.FindRootConst( port);
        return self._RootTrigger[root] != U32::_X;
    }

    pub fn	BuildPortToTriggerConst( &self) -> Buff< TriggerId>
    {
        let  	count = self._Equiv.Size();
        let  	mut portToTrigger = Stash::WithCapacity( count);
        USeg::New( U32::_0, count).Traverse( |i| {
            let  	root = self._Equiv.FindConst( i);
            let  	trig = self._RootTrigger[root];
            assert!( trig != U32::_X, "Port index {} was not assigned a TriggerId before build", i.0);
            portToTrigger.Push( trig);
        });
        return portToTrigger.IntoBuff();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Netlist
{
    fn	default() -> Self
    {
        return Self::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl INetlist for Netlist
{
    fn	Grow( &mut self, count: U32)
    {
        self._Equiv.Grow( count);
        USeg::New( U32::_0, count).Traverse( |_| {
            self._Driver.Push( PortId( U32::_X));
            self._RootTrigger.Push( U32::_X);
        });
    }

    #[inline]
    fn	FindRoot( &mut self, port: PortId) -> U32
    {
        return self._Equiv.Find( port.Index());
    }

    fn	Connect( &mut self, driver: PortId, sink: PortId) -> Result< (), LayoutError>
    {
        let  	sinkIdx = sink.Index();
        let  	existingDriver = self._Driver[sinkIdx];
        if existingDriver.0 != U32::_X {
            if existingDriver != driver {
                return Err( LayoutError::DuplicateInputDriver {
                    _DstIn:        sink,
                    _ExistingSrc:  existingDriver,
                    _AttemptedSrc: driver,
                });
            }
        }

        self._Driver[sinkIdx] = driver;
        self._Equiv.Union( driver.Index(), sink.Index());

        return Ok( ());
    }

    #[inline]
    fn	DriverOf( &mut self, port: PortId) -> PortId
    {
        let  	root = self.FindRoot( port);
        return self._Driver[root];
    }

    fn	AssignTrigger( &mut self, rootIdx: U32, portType: PortType) -> TriggerId
    {
        let  	actualRoot = self._Equiv.Find( rootIdx);
        let  	existing = self._RootTrigger[actualRoot];
        if existing != U32::_X {
            return existing;
        }

        let  	trigId = self._NextTriggerId;
        self._NextTriggerId += U32::_1;
        self._RootTrigger[actualRoot] = trigId;
        self._TriggerTypes.Push( portType);
        return trigId;
    }

    #[inline]
    fn	TriggerOf( &mut self, port: PortId) -> TriggerId
    {
        let  	root = self.FindRoot( port);
        return self._RootTrigger[root];
    }

    #[inline]
    fn	HasTrigger( &mut self, port: PortId) -> bool
    {
        let  	root = self.FindRoot( port);
        return self._RootTrigger[root] != U32::_X;
    }

    fn	BuildPortToTrigger( &mut self) -> Buff< TriggerId>
    {
        let  	count = self._Equiv.Size();
        let  	mut portToTrigger = Stash::WithCapacity( count);
        USeg::New( U32::_0, count).Traverse( |i| {
            let  	root = self._Equiv.Find( i);
            let  	trig = self._RootTrigger[root];
            assert!( trig != U32::_X, "Port index {} was not assigned a TriggerId before build", i.0);
            portToTrigger.Push( trig);
        });
        return portToTrigger.IntoBuff();
    }

    #[inline]
    fn	TriggerCount( &self) -> U32
    {
        return self._NextTriggerId;
    }

    #[inline]
    fn	TriggerType( &self, trigId: TriggerId) -> PortType
    {
        return self._TriggerTypes[trigId];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
