//-- mod.rs -------------------------------------------------------------------------------------------------------------------------
pub mod adder;
pub mod gates;
pub mod latches;
pub mod modlayout;
pub mod portlayout;
pub mod reg;
pub mod sim_context;
pub mod trigger;
mod _tests;

pub use	modlayout::{ IModLayout, IModule, ModLayout, Module };
pub use	portlayout::{ IPort, IPortLayout, Port, PortLayout };
pub use	trigger::{ TriggerId, TriggerSense, TriggerWad };

//---------------------------------------------------------------------------------------------------------------------------------
