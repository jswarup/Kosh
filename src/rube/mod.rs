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
pub mod vcd_model;
pub mod fifo;
pub mod coro_kernel;
mod _tests;

pub use	adder::{ Adder, AdderPipeline, BusAdder32, FullAdder, HalfAdder };
pub use	coro_kernel::{ CoroInstance, CoroKernelFactory, CoroPorts, CoroWarp, CORO_MAX_PORTS };
pub use	engine::{ SimEngine, SimEngineMode };
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSLatch };
pub use	layout::{ HierarchyBuilder, Layout, LayoutError };
pub use	module::{
    CustomModule, CustomWarp, FastModule, FastWarp, HierModule, HierarchyError,
    KernelKind, KernelOp, Module, ModuleId, PortAccess, PortRef, PortSpec, SealedModule,
};
pub use	netlist::{ INetlist, Netlist };
pub use	port::{ PortDesc, PortDir, PortId, PortType };
pub use	reg::{ Reg, RegVal };

pub use	trigger::{ ITriggerWad, TriggerId, TriggerWad };
pub use vcd::VcdWriter;
pub use vcdio::{ ParseVcd, VcdScope };
pub use vcd_model::{ VcdDisplayModel, VcdSignal };
pub use fifo::Fifo;

//---------------------------------------------------------------------------------------------------------------------------------
