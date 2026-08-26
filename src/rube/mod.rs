//-- mod.rs -------------------------------------------------------------------------------------------------------------------------
pub mod adder;
pub mod compiler;
pub mod engine;
pub mod gates;
pub mod latches;
pub mod layout;
pub mod modlayout;
pub mod module;
pub mod port;
pub mod portlayout;
pub mod reg;
pub mod regval;
pub mod signal;
pub mod sim_context;
pub mod trigger;
mod _tests;

pub use	adder::{ Adder, BusAdder32, FullAdder, HalfAdder };
pub use	compiler::NetCompiler;
pub use	engine::{ CustomModule, FastModule, SimEngine };
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSFlipFlop, RSLatch };
pub use	layout::{ Layout, LayoutError };
pub use	modlayout::{ ModLayout, Module };
pub use	module::{ KernelKind, KernelOp, ModuleDescriptor, ModuleId };
pub use	port::{ Port, PortDesc, PortDir, PortId, PortType };
pub use	portlayout::{ PortLayout, TopologyPort };
pub use	reg::Reg;
pub use	regval::RegVal;
pub use	signal::{ SignalMeta, SignalState };
pub use	sim_context::{ ActionId, ActionKind, Sensitivity, SimContext };
pub use	trigger::{ TriggerId, TriggerSense, TriggerWad };

//---------------------------------------------------------------------------------------------------------------------------------
