use	crate::{
    rube::{
        engine::SimEngine,
        reg::Reg,
        trigger::TriggerId,
    },
    silo::{ Stash, U32 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub enum SimulationCommand
{
    Run,
    Step( usize),
    Pause,
    Reset,
    Query( TriggerId),
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub enum SimulationEvent
{
    Running,
    Paused,
    Stepped( usize),
    ResetComplete,
    Queried( Reg),
    BreakpointHit( TriggerId, Reg),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ISimulationController
{
    fn	ExecuteCommand( &mut self, cmd: SimulationCommand) -> SimulationEvent;
    fn	AddBreakpoint( &mut self, trigger: TriggerId, condition: fn( Reg) -> bool);
    fn	RemoveBreakpoint( &mut self, trigger: TriggerId);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Breakpoint
{
    pub _TriggerId: TriggerId,
    pub _Condition: fn( Reg) -> bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct SimulationController< 'a>
{
    pub _Engine:      &'a mut SimEngine,
    pub _Breakpoints: Stash< Breakpoint>,
    pub _IsRunning:   bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> SimulationController< 'a>
{
    pub fn	New( engine: &'a mut SimEngine) -> Self
    {
        return Self {
            _Engine:      engine,
            _Breakpoints: Stash::New(),
            _IsRunning:   false,
        };
    }

    fn	CheckBreakpoints( &self) -> Option< ( TriggerId, Reg)>
    {
        for i in 0..self._Breakpoints.Size().AsUsize() {
            let  	bp = &self._Breakpoints[U32( i as u32)];
            let  	val = self._Engine.GetTrigger( bp._TriggerId);
            if ( bp._Condition)( val) {
                return Some( ( bp._TriggerId, val));
            }
        }
        return None;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> ISimulationController for SimulationController< 'a>
{
    fn	ExecuteCommand( &mut self, cmd: SimulationCommand) -> SimulationEvent
    {
        match cmd {
            SimulationCommand::Run => {
                self._IsRunning = true;
                // In a real environment, Run would block or spin a background thread.
                // Here we just step continuously until a breakpoint is hit.
                loop {
                    if !self._IsRunning {
                        return SimulationEvent::Paused;
                    }
                    self._Engine.Drive();
                    if let Some( ( trigId, val)) = self.CheckBreakpoints() {
                        self._IsRunning = false;
                        return SimulationEvent::BreakpointHit( trigId, val);
                    }
                }
            }
            SimulationCommand::Step( cycles) => {
                let  	mut stepsTaken = 0;
                while stepsTaken < cycles {
                    self._Engine.Drive();
                    stepsTaken += 1;
                    if let Some( ( trigId, val)) = self.CheckBreakpoints() {
                        self._IsRunning = false;
                        return SimulationEvent::BreakpointHit( trigId, val);
                    }
                }
                return SimulationEvent::Stepped( stepsTaken);
            }
            SimulationCommand::Pause => {
                self._IsRunning = false;
                return SimulationEvent::Paused;
            }
            SimulationCommand::Reset => {
                self._IsRunning = false;
                // Currently SimEngine does not have a hard Reset() function,
                // but this signals the protocol intention.
                return SimulationEvent::ResetComplete;
            }
            SimulationCommand::Query( trigId) => {
                let  	val = self._Engine.GetTrigger( trigId);
                return SimulationEvent::Queried( val);
            }
        }
    }

    fn	AddBreakpoint( &mut self, trigger: TriggerId, condition: fn( Reg) -> bool)
    {
        self._Breakpoints.Push( Breakpoint {
            _TriggerId: trigger,
            _Condition: condition,
        });
    }

    fn	RemoveBreakpoint( &mut self, trigger: TriggerId)
    {
        // Stash doesn't support removal easily, so we rebuild the stash.
        let  	mut newStash = Stash::New();
        for i in 0..self._Breakpoints.Size().AsUsize() {
            let  	bp = &self._Breakpoints[U32( i as u32)];
            if bp._TriggerId != trigger {
                newStash.Push( Breakpoint {
                    _TriggerId: bp._TriggerId,
                    _Condition: bp._Condition,
                });
            }
        }
        self._Breakpoints = newStash;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
pub mod _tests
{
    use	super::*;
    use	crate::rube::{
        adder::BusAdder32,
        layout::Layout,
    };

    #[test]
    fn	test_simulation_controller_step_and_breakpoint()
    {
        let  	mut layout = Layout::New();
        let  	adder = BusAdder32::New( &mut layout, "adder", None);
        let  	mut engine = SimEngine::Create( &layout);

        let  	mut ctrl = SimulationController::New( &mut engine);

        // Step 1: Query initial state
        let  	trigId = ctrl._Engine.GetPortTrigger( adder.Sum()).unwrap();
        let  	res = ctrl.ExecuteCommand( SimulationCommand::Query( trigId));
        assert_eq!( res, SimulationEvent::Queried( Reg::FromU32( U32( 0))));

        // Step 2: Add breakpoint when sum == 42
        ctrl.AddBreakpoint( trigId, |val| val.Masked( 0xFFFF_FFFF).Val() == 42);

        // Step 3: Set inputs
        ctrl._Engine.SetPortU32( adder.A(), Reg::FromU32( U32( 40)));
        ctrl._Engine.SetPortU32( adder.B(), Reg::FromU32( U32( 2)));

        // Step 4: Run
        let  	res = ctrl.ExecuteCommand( SimulationCommand::Run);

        // Should hit breakpoint on next cycle
        assert!( matches!( res, SimulationEvent::BreakpointHit( id, v) if id == trigId && v.Masked( 0xFFFF_FFFF).Val() == 42));

        // Step 5: Remove breakpoint and step
        ctrl.RemoveBreakpoint( trigId);
        let  	res = ctrl.ExecuteCommand( SimulationCommand::Step( 1));
        assert_eq!( res, SimulationEvent::Stepped( 1));
    }
}
