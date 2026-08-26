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

pub use	adder::{ Adder, FullAdder, HalfAdder, IAdder, IFullAdder, IHalfAdder };
pub use	gates::{
    AndGate, IAndGate, INandGate, INotGate, IOrGate, IXorGate, NandGate, NotGate, OrGate, XorGate,
};
pub use	latches::{
    CRSLatch, DLatch, ICRSLatch, IDLatch, IRSFlipFlop, IRSLatch, RSFlipFlop, RSLatch,
};
pub use	modlayout::{ IModLayout, IModule, ModLayout, Module };
pub use	portlayout::{ IPort, IPortLayout, Port, PortLayout };
pub use	reg::{ IReg, IRegBool, Reg };
pub use	sim_context::{ ActionId, ActionKind, ISimContext, Sensitivity, SimContext };
pub use	trigger::{ ITriggerWad, ITriggerWadBool, TriggerId, TriggerSense, TriggerWad };

//---------------------------------------------------------------------------------------------------------------------------------
