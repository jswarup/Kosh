//-- mod.rs -------------------------------------------------------------------------------------------------------------------------
pub mod adder;
pub mod gates;
pub mod latches;
pub mod portlayout;
pub mod reg;
pub mod sim_context;
pub mod trigger;
mod _tests;

pub use	portlayout::{ IPortLayout, PortLayout, IPort, Port };
pub use	trigger::{ TriggerId, TriggerSense, TriggerWad };

//---------------------------------------------------------------------------------------------------------------------------------
