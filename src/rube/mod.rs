//-- mod.rs -------------------------------------------------------------------------------------------------------------------------
pub mod adder;
pub mod engine;
pub mod gates;
pub mod latches;
pub mod layout;
pub mod module;
pub mod netlist;
pub mod port;
pub mod reg;
pub mod registry;
pub mod trigger;
pub mod vcd;
pub mod vcdio;
pub mod fifo;
mod _tests;

pub use	adder::{ Adder, BusAdder32, FullAdder, HalfAdder };
pub use	engine::SimEngine;
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSLatch };
pub use	layout::{ Layout, LayoutError };
pub use	module::{ CustomModule, CustomWarp, FastModule, FastWarp, KernelKind, KernelOp, Module, ModuleId };
pub use	netlist::{ INetlist, Netlist };
pub use	port::{ PortDesc, PortDir, PortId, PortType };
pub use	reg::{ Reg, RegVal };

pub use	trigger::{ ITriggerWad, TriggerId, TriggerWad };
pub use vcd::VcdWriter;
pub use fifo::Fifo;

//---------------------------------------------------------------------------------------------------------------------------------
