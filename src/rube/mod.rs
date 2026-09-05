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
pub mod interface;
pub mod introspect;
pub mod kernel;
pub mod coro_kernel;
pub mod sim_ctrl;
pub mod dpi;
pub mod package;
pub mod _tests;

pub use interface::{ ModuleInterface, PortInterface, DataType, BusType, IModuleInterface };
pub use introspect::{ IModuleIntrospection, PortIntrospection };
pub use kernel::{ IKernel, KernelSignature, KernelError };

pub use	adder::{ Adder, AdderPipeline, BusAdder32, FullAdder, HalfAdder };
pub use	coro_kernel::{ CoroInstance, CoroKernelFactory, CoroPorts, CoroWarp, CORO_MAX_PORTS };
pub use	engine::{ SimEngine, SimEngineMode };
pub use	gates::{ AndGate, NandGate, NotGate, OrGate, XorGate };
pub use	latches::{ CRSLatch, DLatch, RSLatch };
pub use	layout::{ Layout, LayoutError };
pub use	module::{
    CustomModule, CustomWarp, FastModule, FastWarp,
    KernelKind, KernelOp, Module, ModuleId,
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
