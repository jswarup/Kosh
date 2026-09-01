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
pub mod vcd;
pub mod vcdio;
mod _tests;

pub use	adder::{ Adder, BusAdder32, FullAdder, HalfAdder };
pub use	engine::SimEngine;
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSLatch };
pub use	layout::{ Layout, LayoutError };
pub use	module::{ CustomModule, FastModule, KernelKind, KernelOp, Module, ModuleId };
pub use	port::{ PortDesc, PortDir, PortId, PortSensitivity, PortType };
pub use	reg::{ Reg, RegVal };

pub use	trigger::{ ITriggerWad, TriggerId, TriggerWad };
pub use vcd::VcdWriter;

//---------------------------------------------------------------------------------------------------------------------------------
