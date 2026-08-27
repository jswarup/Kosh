//-- mod.rs -------------------------------------------------------------------------------------------------------------------------
pub mod adder;
pub mod engine;
pub mod gates;
pub mod latches;
pub mod layout;
pub mod module;
pub mod port;
pub mod reg;

pub mod trigger;
mod _tests;

pub use	adder::{ Adder, BusAdder32, FullAdder, HalfAdder };
pub use	engine::{ CustomModule, FastModule, SimEngine };
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSLatch };
pub use	layout::{ Layout, LayoutError };
pub use	module::{ KernelKind, KernelOp, Module, ModuleId };
pub use	port::{ PortDesc, PortDir, PortId, PortType };
pub use	reg::{ Reg, RegVal };

pub use	trigger::{ TriggerId, TriggerMeta, TriggerState };

//---------------------------------------------------------------------------------------------------------------------------------
