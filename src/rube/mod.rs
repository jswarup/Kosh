//-- mod.rs -------------------------------------------------------------------------------------------------------------------------
pub mod adder;
pub mod compiler;
pub mod engine;
pub mod gates;
pub mod latches;
pub mod layout;
pub mod module;
pub mod port;
pub mod reg;
pub mod sim_context;
pub mod trigger;
mod _tests;

pub use	adder::{ Adder, BusAdder32, FullAdder, HalfAdder };
pub use	compiler::NetCompiler;
pub use	engine::{ CustomModule, FastModule, SimEngine };
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSLatch };
pub use	layout::{ Layout, LayoutError };
pub use	module::{ KernelKind, KernelOp, ModuleDescriptor, ModuleId };
pub use	port::{ Port, PortDesc, PortDir, PortId, PortLayout, PortType, TopologyPort };
pub use	reg::{ Reg, RegVal };
pub use	sim_context::{ ActionId, ActionKind, Sensitivity, SimContext };
pub use	trigger::{ TriggerId, TriggerMeta, TriggerSense, TriggerState, TriggerWad };

//---------------------------------------------------------------------------------------------------------------------------------
