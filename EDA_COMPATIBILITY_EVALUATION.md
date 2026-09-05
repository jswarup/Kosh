# EDA Compatibility & Modularity Evaluation for Rube

## Executive Summary

Rube has **excellent performance characteristics** but has **limited EDA compatibility** and **modularity gaps** that restrict integration with standard industry tools. This document proposes modular, EDA-compatible changes aligned with:

- **SystemVerilog/VHDL** simulation interface conventions
- **OSI-TLM** (Open SystemC Initiative - Transaction Level Modeling)
- **Verilog VPI/DPI-C** simulation standards
- **OSCI SystemC** module patterns
- **Universal Verification Methodology (UVM)** hierarchies

---

## Part 1: Current State Analysis

### Strengths ✅

| Aspect | Status | Notes |
|--------|--------|-------|
| **Performance** | ⭐⭐⭐⭐⭐ | SIMT warp execution, zero-alloc hot path |
| **Register Model** | ⭐⭐⭐⭐☆ | IEEE 1364 4-state logic (0, 1, X) |
| **Kernel Types** | ⭐⭐⭐⭐☆ | Fast/Custom/Behavioral/Coro kernels |
| **Hierarchy** | ⭐⭐⭐☆☆ | Basic container support but limited encapsulation |
| **Port System** | ⭐⭐⭐☆☆ | Simple PortId/PortDir but limited metadata |

### Limitations ❌

| Gap | Severity | Impact |
|-----|----------|--------|
| **No Module Interface Standard** | High | Can't document or export module contracts |
| **String-based Kernel Registry** | Medium | Ad-hoc, no type safety, hard to compose |
| **No Simulation Control Protocol** | High | Can't pause/resume/step simulation from external tools |
| **Ports lack metadata** | High | No bus widths, signal types, documentation in type system |
| **No DPI/VPI Interface** | High | Can't integrate with Verilog/VHDL tools |
| **Tight Module-Kernel Coupling** | Medium | Hard to package modules independently |
| **No formal verification hooks** | High | Can't connect to assertion/coverage tools |
| **Internal Reg type only** | Medium | No interop with other simulators |
| **No module introspection** | Medium | Can't query module structure at runtime |
| **No IP packaging standard** | High | Hard to distribute/license/reuse complex modules |

---

## Part 2: EDA Compatibility Gaps

### Gap 1: Module Interface Definition (No Standard Contract)

**Current:**
```rust
// How do users know what this module does?
struct MyModule;
impl MyModule {
    pub fn New() -> HierModule { ... }
}

// No documentation in type system!
```

**EDA Standard (Verilog/SystemVerilog/VHDL):**
```verilog
module adder_pipeline #(
    parameter WIDTH = 32
) (
    input clk,
    input [WIDTH-1:0] a, b,
    output [WIDTH-1:0] sum,
    output carry
);
```

**Proposal:**
```rust
/// Module interface definition (like SystemVerilog module statement)
pub struct ModuleInterface {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub inports: &'static [PortInterface],
    pub outports: &'static [PortInterface],
    pub parameters: &'static [ParameterInterface],
}

pub struct PortInterface {
    pub name: &'static str,
    pub width: usize,
    pub data_type: DataType,
    pub direction: PortDir,
    pub description: &'static str,
}

pub enum DataType {
    Logic(usize),          // Bit width
    Bool,
    Integer,
    Real,
    Custom(&'static str),  // User-defined types
}

/// Trait: Every module must define its interface
pub trait IModuleInterface {
    fn interface() -> &'static ModuleInterface;
}

// Usage:
impl IModuleInterface for AdderPipeline {
    fn interface() -> &'static ModuleInterface {
        &ADDER_INTERFACE
    }
}

const ADDER_INTERFACE: ModuleInterface = ModuleInterface {
    name: "adder_pipeline",
    version: "1.0.0",
    description: "32-bit pipeline adder with latency control",
    inports: &[
        PortInterface {
            name: "a",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::In,
            description: "First operand",
        },
        // ... more ports
    ],
    outports: &[
        PortInterface {
            name: "result",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::Out,
            description: "Sum output",
        },
    ],
    parameters: &[],
};
```

**Benefits:**
- ✅ Self-documenting modules (interface in type system)
- ✅ Can export to SystemVerilog/VHDL
- ✅ Enables automated verification hooks
- ✅ Supports reflection/introspection
- ✅ Runtime configuration validation

---

### Gap 2: Kernel Registry (String-based, No Type Safety)

**Current Problem:**
```rust
// Ad-hoc string registration
registry._Map.insert("BusAdder32_Kernel", Arc::new(|inputs, outputs| {
    // What are the expected inputs/outputs?
    // How many? What types? NO DOCUMENTATION!
}));

// Lookup is unsafe:
if let Some(cb) = registry._Map.get("BusAdder32_Kernel") {
    cb(&inputs, &mut outputs);  // Runtime check only
}
```

**EDA Standard (SystemC/TLM):**
```cpp
class AdderModule : public sc_module {
public:
    sc_in<uint32_t> a, b;
    sc_out<uint32_t> sum;
    sc_out<bool> carry;

    void process() { /* behavior */ }
};
```

**Proposal: Trait-Based Kernel Registry**

```rust
/// Standard kernel trait with full type information
pub trait IKernel: Send + Sync {
    fn name() -> &'static str;
    fn version() -> &'static str;
    fn signature() -> &'static KernelSignature;  // Port definitions
    fn execute(&self, inputs: &[Reg], outputs: &mut [Reg]) -> Result<(), KernelError>;
}

pub struct KernelSignature {
    pub input_ports: &'static [PortInterface],
    pub output_ports: &'static [PortInterface],
}

/// Type-safe kernel registry
pub struct KernelRegistry<K: IKernel> {
    kernels: Arc<HashMap<&'static str, Arc<K>>>,
}

impl<K: IKernel> KernelRegistry<K> {
    pub fn register<T: IKernel + 'static>(mut self, kernel: Arc<T>) -> Self {
        self.kernels.insert(T::name(), kernel as Arc<dyn IKernel>);
        self
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn IKernel>> {
        self.kernels.get(name).cloned()
    }
}

// Example implementation
pub struct BusAdder32;

impl IKernel for BusAdder32 {
    fn name() -> &'static str { "bus_adder_32" }
    fn version() -> &'static str { "1.0.0" }

    fn signature() -> &'static KernelSignature {
        &BUSADDER_SIGNATURE
    }

    fn execute(&self, inputs: &[Reg], outputs: &mut [Reg]) -> Result<(), KernelError> {
        if inputs.len() != 2 || outputs.len() != 2 {
            return Err(KernelError::PortCountMismatch);
        }
        let a = inputs[0].Val();
        let b = inputs[1].Val();
        outputs[0] = Reg::FromU32(U32((a.wrapping_add(b)) as u32));
        outputs[1] = Reg::FromBool((a + b) > u32::MAX as u64);
        Ok(())
    }
}

const BUSADDER_SIGNATURE: KernelSignature = KernelSignature {
    input_ports: &[
        PortInterface {
            name: "a",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::In,
            description: "First operand",
        },
        PortInterface {
            name: "b",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::In,
            description: "Second operand",
        },
    ],
    output_ports: &[
        PortInterface {
            name: "sum",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::Out,
            description: "32-bit sum",
        },
        PortInterface {
            name: "carry",
            width: 1,
            data_type: DataType::Bool,
            direction: PortDir::Out,
            description: "Carry-out",
        },
    ],
};
```

**Benefits:**
- ✅ Type-safe kernel definition
- ✅ Self-documenting kernel signatures
- ✅ Compile-time port count verification
- ✅ Runtime signature matching validation
- ✅ Can export to HDL/SystemC

---

### Gap 3: No Simulation Control Protocol

**Current:**
```rust
let mut engine = SimEngine::Create(&layout)?;
for _i in 0..100 {
    engine.Drive()?;  // No pause, step, introspection!
}
```

**EDA Standard (Verilog VPI):**
```c
// Can pause/resume/step/query
vpi_control(vpiFinish, 1);  // Pause
vpi_control(vpiStep, 10);    // Step 10 cycles
```

**Proposal: Simulation Control Interface**

```rust
pub enum SimulationCommand {
    Run(usize),              // Run N cycles
    Step(usize),             // Single-step N cycles
    Pause,                   // Pause simulation
    Reset,                   // Reset to initial state
    Query(SimulationQuery),  // Query simulation state
}

pub enum SimulationQuery {
    CycleCount,
    PortValue(PortId),
    ModuleState(ModuleId),
    TriggerValue(TriggerId),
    ProbeSignal(String),  // Named signal path
}

pub enum SimulationEvent {
    Paused,
    Completed,
    BreakpointHit(BreakpointId),
    SignalChanged(PortId, Reg),
    PortWrite(PortId, Reg),
}

pub trait ISimulationController {
    fn execute(&mut self, cmd: SimulationCommand) -> Result<SimulationEvent, SimError>;
    fn query(&self, query: SimulationQuery) -> Result<SimulationValue, SimError>;
    fn set_breakpoint(&mut self, bp: Breakpoint) -> BreakpointId;
    fn register_probe(&mut self, name: String, target: ProbeTarget) -> ProbeId;
}

pub struct BreakpointSpec {
    pub target: BreakpointTarget,
    pub condition: Option<BreakpointCondition>,
}

pub enum BreakpointTarget {
    PortChange(PortId),
    PortValue(PortId, Reg),
    CycleCount(usize),
    ModuleState(ModuleId),
}

// Implementation
impl ISimulationController for SimEngine {
    fn execute(&mut self, cmd: SimulationCommand) -> Result<SimulationEvent, SimError> {
        match cmd {
            SimulationCommand::Run(cycles) => {
                for _ in 0..cycles {
                    self.Drive()?;
                }
                Ok(SimulationEvent::Completed)
            }
            SimulationCommand::Step(cycles) => {
                for _ in 0..cycles {
                    self.Drive()?;
                    // Check breakpoints after each cycle
                }
                Ok(SimulationEvent::Paused)
            }
            // ...
        }
    }

    fn query(&self, query: SimulationQuery) -> Result<SimulationValue, SimError> {
        match query {
            SimulationQuery::CycleCount => {
                Ok(SimulationValue::Integer(self._CycleCount as i64))
            }
            SimulationQuery::PortValue(port_id) => {
                let trigger_id = self._PortToTrigger[port_id.0.AsUsize()];
                Ok(SimulationValue::Register(self._Triggers.Current(trigger_id)))
            }
            // ...
        }
    }
}
```

**Benefits:**
- ✅ Standard simulation control (pause/resume/step)
- ✅ Runtime introspection
- ✅ Breakpoint support
- ✅ Probe/watchpoint system
- ✅ Can integrate with debuggers/trace tools

---

### Gap 4: Port Metadata Limitation

**Current:**
```rust
pub struct PortDesc {
    pub _Id: PortId,
    pub _Name: String,
    pub _Owner: ModuleId,
    pub _Type: PortType,
}

// PortType has no width information!
pub enum PortType {
    Bool,
    U8Val,
    U16Val,
    U32Val,
    U64Val,
}
```

**Problem:** Can't represent buses, bit-slices, or custom types in type system

**Proposal: Rich Port Metadata**

```rust
pub struct PortDesc {
    pub _Id: PortId,
    pub _Name: String,
    pub _Owner: ModuleId,
    pub _Type: PortType,
    pub _Metadata: PortMetadata,
}

pub struct PortMetadata {
    pub width: usize,
    pub data_type: DataType,
    pub bus_type: BusType,
    pub attributes: PortAttributes,
    pub documentation: Option<&'static str>,
}

pub enum BusType {
    Single,                    // Single-bit wire
    Bus(usize),               // Multi-bit bus
    Struct {                  // Compound type
        fields: &'static [PortField],
    },
    Array {                   // Array of signals
        element_type: Box<BusType>,
        length: usize,
    },
}

pub struct PortField {
    pub name: &'static str,
    pub bit_range: (usize, usize),  // [high:low]
    pub data_type: DataType,
}

pub struct PortAttributes {
    pub is_clock: bool,
    pub is_reset: bool,
    pub is_valid: bool,
    pub clock_polarity: Polarity,
    pub reset_polarity: Polarity,
    pub tags: &'static [&'static str],  // For filtering (e.g., "async", "critical_path")
}

pub enum Polarity {
    Positive,  // Rising edge / active high
    Negative,  // Falling edge / active low
}

// Usage
const ADDER_INPORT_A: PortMetadata = PortMetadata {
    width: 32,
    data_type: DataType::Logic(32),
    bus_type: BusType::Bus(32),
    attributes: PortAttributes {
        is_clock: false,
        is_reset: false,
        is_valid: false,
        clock_polarity: Polarity::Positive,
        reset_polarity: Polarity::Positive,
        tags: &["operand", "input"],
    },
    documentation: Some("32-bit operand A"),
};

const ADDER_CLOCK: PortMetadata = PortMetadata {
    width: 1,
    data_type: DataType::Bool,
    bus_type: BusType::Single,
    attributes: PortAttributes {
        is_clock: true,
        is_reset: false,
        is_valid: false,
        clock_polarity: Polarity::Positive,
        reset_polarity: Polarity::Positive,
        tags: &["clock"],
    },
    documentation: Some("Rising-edge triggered clock"),
};
```

**Benefits:**
- ✅ Rich type information in port system
- ✅ Clock/reset/valid signal recognition
- ✅ Bus width and structure in metadata
- ✅ Verification framework integration
- ✅ Can export to SystemVerilog/VHDL with full type info

---

### Gap 5: No DPI/VPI Simulation Interface

**Problem:** Can't integrate Rube with external Verilog/VHDL/SystemC simulators

**Proposal: Dual-Mode Simulation Kernel**

```rust
/// Standard simulation socket for external tool integration
pub trait ISimulationSocket {
    fn connect(&mut self, remote: Box<dyn IRemoteSimulator>) -> Result<(), SimError>;
    fn exchange(&mut self) -> Result<SimulationCycle, SimError>;
}

pub struct SimulationCycle {
    pub inputs: HashMap<String, Reg>,
    pub outputs: HashMap<String, Reg>,
    pub cycle_count: usize,
}

pub trait IRemoteSimulator {
    fn step(&mut self, inputs: &HashMap<String, Reg>) -> Result<HashMap<String, Reg>, SimError>;
    fn get_interface(&self) -> &'static ModuleInterface;
}

// Example: Verilog co-simulation via VPI
pub struct VpiSocket {
    module_name: String,
    port_bindings: HashMap<String, PortId>,
    // ... VPI handle
}

impl ISimulationSocket for VpiSocket {
    fn connect(&mut self, remote: Box<dyn IRemoteSimulator>) -> Result<(), SimError> {
        // Bind to external Verilog simulator
        Ok(())
    }

    fn exchange(&mut self) -> Result<SimulationCycle, SimError> {
        // Read outputs from Verilog, write inputs back
        Ok(SimulationCycle { /* ... */ })
    }
}
```

**Benefits:**
- ✅ Co-simulation with Verilog/VHDL
- ✅ Can mix Rube modules with external RTL
- ✅ Seamless HDL integration
- ✅ Verification tool connectivity

---

## Part 3: Modularity Improvements

### Issue 1: Tight Module-Kernel Coupling

**Current:**
```rust
// Kernel must be registered in global registry
// Module and kernel are tightly coupled
pub struct Module {
    pub _Kernel: KernelKind,  // Embedded kernel
    // No separate kernel loading mechanism
}
```

**Proposal: Modular Kernel System**

```rust
/// Kernel package (like an IP core)
pub struct KernelPackage {
    pub interface: &'static ModuleInterface,
    pub metadata: PackageMetadata,
    pub factory: KernelFactory,
}

pub struct PackageMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub vendor: &'static str,
    pub license: &'static str,
    pub dependencies: &'static [PackageDependency],
}

pub struct PackageDependency {
    pub name: &'static str,
    pub version_constraint: &'static str,
}

pub trait KernelFactory: Send + Sync {
    fn create(&self) -> Box<dyn IKernel>;
    fn create_with_params(&self, params: &[Parameter]) -> Result<Box<dyn IKernel>, KernelError>;
}

pub struct Parameter {
    pub name: &'static str,
    pub value_type: ParameterType,
    pub default: Option<String>,
}

pub enum ParameterType {
    Integer,
    String,
    Real,
    Boolean,
}

/// Kernel library (like a design library in EDA)
pub struct KernelLibrary {
    packages: HashMap<String, KernelPackage>,
}

impl KernelLibrary {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    pub fn register_package(&mut self, package: KernelPackage) -> Result<(), LibraryError> {
        self.packages.insert(package.interface.name.to_string(), package);
        Ok(())
    }

    pub fn instantiate(
        &self,
        kernel_name: &str,
        params: &[Parameter],
    ) -> Result<Box<dyn IKernel>, LibraryError> {
        let package = self.packages.get(kernel_name)
            .ok_or(LibraryError::PackageNotFound)?;
        package.factory.create_with_params(params)
            .map_err(LibraryError::KernelError)
    }
}
```

**Benefits:**
- ✅ Kernel packages are self-contained
- ✅ Separation of kernel definition from module use
- ✅ Parameterizable kernels
- ✅ Dependency management
- ✅ Can package/distribute IP cores

---

### Issue 2: No Module Introspection

**Current:**
```rust
// Can't query module structure at runtime
// No way to iterate ports, submodules, etc.
```

**Proposal: Module Introspection API**

```rust
pub trait IModuleIntrospection {
    fn get_interface(&self) -> &'static ModuleInterface;
    fn list_inports(&self) -> Vec<PortIntrospection>;
    fn list_outports(&self) -> Vec<PortIntrospection>;
    fn list_submodules(&self) -> Vec<ModuleIntrospection>;
    fn get_submodule(&self, name: &str) -> Option<Box<dyn IModuleIntrospection>>;
    fn get_hierarchy_path(&self) -> String;  // e.g., "top.pipeline.adder"
    fn get_statistics(&self) -> ModuleStatistics;
}

pub struct PortIntrospection {
    pub name: String,
    pub port_id: PortId,
    pub metadata: PortMetadata,
    pub current_value: Reg,
}

pub struct ModuleIntrospection {
    pub name: String,
    pub module_id: ModuleId,
    pub interface: &'static ModuleInterface,
    pub hierarchy_depth: usize,
}

pub struct ModuleStatistics {
    pub cycle_count: usize,
    pub port_changes: HashMap<PortId, usize>,
    pub execution_time_ns: u64,
}

// Implementation
impl IModuleIntrospection for Module {
    fn list_inports(&self) -> Vec<PortIntrospection> {
        self._InPorts.iter().enumerate().map(|(i, spec)| {
            PortIntrospection {
                name: spec._Name.clone(),
                port_id: PortId::In(i as U32),
                metadata: spec._Metadata.clone(),
                current_value: Reg::default(),  // Read from engine
            }
        }).collect()
    }

    fn get_hierarchy_path(&self) -> String {
        let mut path = vec![self._Name.clone()];
        let mut current = self._Parent;
        while let Some(parent_id) = current {
            // Lookup parent in layout
            path.push(get_module(parent_id)._Name.clone());
            current = get_module(parent_id)._Parent;
        }
        path.reverse();
        path.join(".")
    }
}
```

**Benefits:**
- ✅ Runtime module structure queries
- ✅ Hierarchy path reporting
- ✅ Port value inspection
- ✅ Module statistics
- ✅ Integration with debuggers/trace tools

---

### Issue 3: No Module Packaging Standard

**Current:**
```rust
// Hard to package and distribute complex modules
// No versioning, licensing, or dependency management
```

**Proposal: Module Package Manifest**

```rust
pub struct ModulePackage {
    pub manifest: PackageManifest,
    pub root_module: Box<dyn IModuleInterface>,
    pub dependencies: Vec<ModulePackage>,
}

pub struct PackageManifest {
    pub name: String,
    pub version: semver::Version,
    pub description: String,
    pub vendor: String,
    pub license: String,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: Vec<PackageDependency>,
    pub exports: Vec<ModuleExport>,
}

pub struct ModuleExport {
    pub name: &'static str,
    pub path: String,
    pub interface: &'static ModuleInterface,
}

// Package loading
pub struct ModuleLoader {
    search_paths: Vec<PathBuf>,
    loaded_packages: HashMap<String, Arc<ModulePackage>>,
}

impl ModuleLoader {
    pub fn load_package(&mut self, name: &str) -> Result<Arc<ModulePackage>, LoadError> {
        if let Some(pkg) = self.loaded_packages.get(name) {
            return Ok(pkg.clone());
        }

        for path in &self.search_paths {
            let manifest_path = path.join(format!("{}.toml", name));
            if manifest_path.exists() {
                let manifest = load_manifest(&manifest_path)?;
                let pkg = Arc::new(ModulePackage {
                    manifest,
                    root_module: create_module()?,
                    dependencies: vec![],
                });
                self.loaded_packages.insert(name.to_string(), pkg.clone());
                return Ok(pkg);
            }
        }

        Err(LoadError::PackageNotFound(name.to_string()))
    }
}

// Package manifest file (TOML)
/*
[package]
name = "rube-adder-pipeline"
version = "1.0.0"
description = "High-performance 32-bit pipeline adder"
vendor = "OrioleDesigns"
license = "Apache-2.0"
authors = ["Your Name <you@example.com>"]

[dependencies]
rube-core = "0.5.0"

[[exports]]
name = "AdderPipeline"
interface = "adder_pipeline"
description = "32-bit 3-stage pipeline adder"

[[exports]]
name = "AdderPipelineConfigurable"
interface = "adder_pipeline_configurable"
description = "Configurable-width pipeline adder"
*/
```

**Benefits:**
- ✅ Standard packaging format (like Cargo)
- ✅ Dependency management
- ✅ Version tracking
- ✅ Licensing/attribution
- ✅ Can publish to central repository (like crates.io)

---

## Part 4: Proposed Architecture Diagram

### Current (Monolithic)

```
┌────────────────────────────────────────────────────┐
│ Simulation System (Monolithic)                     │
├────────────────────────────────────────────────────┤
│                                                    │
│  SimEngine                                         │
│  ├─ Drive()                  ← No external control│
│  ├─ _Triggers                                      │
│  └─ _PortToTrigger                                │
│                                                    │
│  Layout                                            │
│  ├─ _Modules                                       │
│  ├─ _Ports                  ← Limited metadata   │
│  └─ _Netlist                                       │
│                                                    │
│  KernelRegistry                                    │
│  └─ _Map[String]  ← String-based, no type safety │
│                                                    │
│  Module                                            │
│  ├─ _Kernel                                        │
│  └─ _IsSealed: bool  ← Runtime check              │
│                                                    │
│  (External tools can't integrate)  ❌              │
└────────────────────────────────────────────────────┘
```

### Proposed (Modular + EDA-Compatible)

```
┌──────────────────────────────────────────────────────────────────────┐
│ Rube Simulation Framework (Modular + EDA-Compatible)                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────┐  ┌──────────────────────────────────────┐ │
│  │ Simulation Kernel   │  │ Module System                        │ │
│  ├─────────────────────┤  ├──────────────────────────────────────┤ │
│  │ SimEngine           │  │ Module<IN,OUT,SUBS,State> (const)   │ │
│  │ ├─ ISimController   │  │ ├─ Type-safe sealing                │ │
│  │ │  • execute()      │  │ ├─ Static emplacement               │ │
│  │ │  • query()        │  │ └─ IModuleInterface trait           │ │
│  │ ├─ ISimSocket       │  │                                      │ │
│  │ │  • connect()      │  │ Module Interface                    │ │
│  │ │  • exchange()     │  │ ├─ Ports + Metadata                │ │
│  │ ├─ Breakpoints      │  │ ├─ Parameters                       │ │
│  │ ├─ Probes           │  │ ├─ Documentation                    │ │
│  │ └─ Introspection    │  │ └─ Version info                     │ │
│  └─────────────────────┘  └──────────────────────────────────────┘ │
│                                                                       │
│  ┌──────────────────────────┐  ┌──────────────────────────────────┐ │
│  │ Kernel System            │  │ Library & Package Management     │ │
│  ├──────────────────────────┤  ├──────────────────────────────────┤ │
│  │ IKernel Trait            │  │ KernelLibrary                    │ │
│  │ ├─ IKernel types         │  │ ├─ register_package()           │ │
│  │ ├─ KernelSignature       │  │ ├─ instantiate()                │ │
│  │ ├─ KernelFactory         │  │ └─ query_interface()            │ │
│  │ └─ Parameters            │  │                                  │ │
│  │                          │  │ ModulePackage                    │ │
│  │ Predefined Kernels:      │  │ ├─ PackageManifest (TOML)       │ │
│  │ ├─ BusAdder32            │  │ ├─ Dependencies                 │ │
│  │ ├─ DLatch                │  │ ├─ Version management           │ │
│  │ ├─ FIFO                  │  │ └─ Licensing info               │ │
│  │ └─ Custom (user)         │  │                                  │ │
│  └──────────────────────────┘  │ ModuleLoader                     │ │
│                                 │ ├─ load_package()               │ │
│                                 │ ├─ search_paths                 │ │
│                                 │ └─ dependency resolution        │ │
│                                 └──────────────────────────────────┘ │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ External Integration Layer                                   │   │
│  ├──────────────────────────────────────────────────────────────┤   │
│  │                                                               │   │
│  │ DPI/VPI Interface          Co-Simulation Socket              │   │
│  │ ├─ Verilog C interface    ├─ Remote simulator protocol      │   │
│  │ ├─ VHDL interfaces        ├─ Cycle exchange                 │   │
│  │ └─ Foreign models         └─ Cross-tool integration         │   │
│  │                                                               │   │
│  │ Verification Hooks         Debugger Integration              │   │
│  │ ├─ Assertion checks       ├─ Breakpoints                    │   │
│  │ ├─ Coverage collection    ├─ Waveform export                │   │
│  │ └─ Property monitoring    └─ Runtime introspection          │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Port System (Rich Metadata)                                  │   │
│  ├──────────────────────────────────────────────────────────────┤   │
│  │                                                               │   │
│  │ PortMetadata                                                 │   │
│  │ ├─ Width information       BusType (structural)              │   │
│  │ ├─ DataType (Logic/Bool)   ├─ Single bit                    │   │
│  │ ├─ Clock/Reset/Valid tags  ├─ Multi-bit bus                 │   │
│  │ ├─ Polarity (rising/fall)  ├─ Struct (compound)             │   │
│  │ └─ Documentation           └─ Array                         │   │
│  │                                                               │   │
│  │ Can export HDL type info → SystemVerilog/VHDL               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
       ⬇ Seamless Integration ⬇
┌──────────────────────────────────────────────────────────────────────┐
│ External EDA Ecosystem                                               │
│ ├─ Verilog/VHDL Simulators (VCS, ModelSim, xcelium)               │
│ ├─ Formal Verification (Cadence, OneSpin)                         │
│ ├─ Hardware Emulation (Palladium, Veloce)                         │
│ ├─ UVM Testbenches                                                │
│ ├─ Waveform Analysis (GTKWave, Verdi)                             │
│ └─ Coverage/Assertion Tools (Cadence VCM, Mentor, etc.)           │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Part 5: Implementation Roadmap

### Phase 1: Module Interface Standards (2-3 weeks)

- [ ] Define `ModuleInterface` trait and structures
- [ ] Add `IModuleInterface` trait to all modules
- [ ] Create module interface documentation system
- [ ] Add port metadata (width, data type, attributes)
- [ ] Implement module introspection API

**Deliverable:** All modules self-document their interface

---

### Phase 2: Type-Safe Kernel System (2-3 weeks)

- [ ] Create `IKernel` trait with signature
- [ ] Implement `KernelPackage` and `KernelFactory`
- [ ] Create type-safe `KernelRegistry<K>`
- [ ] Migrate existing kernels to new trait
- [ ] Add kernel parameter support

**Deliverable:** Kernel registry is type-safe and documented

---

### Phase 3: Simulation Control Protocol (2-3 weeks)

- [ ] Implement `ISimulationController` trait
- [ ] Add breakpoint and probe systems
- [ ] Implement query interface
- [ ] Add pause/resume/step commands
- [ ] Create event notification system

**Deliverable:** External tools can control simulation

---

### Phase 4: External Integration (3-4 weeks)

- [ ] Create DPI/VPI wrapper layer
- [ ] Implement co-simulation socket (`ISimulationSocket`)
- [ ] Add Verilog integration example
- [ ] Create verification hook system
- [ ] Add debugger integration

**Deliverable:** Can integrate with external simulators

---

### Phase 5: Module Packaging (2 weeks)

- [ ] Create `ModulePackage` and manifest format
- [ ] Implement `ModuleLoader` with dependency resolution
- [ ] Add package repository support
- [ ] Create example packages
- [ ] Document packaging best practices

**Deliverable:** Modules can be packaged, versioned, and distributed

---

## Part 6: Benefits Summary

| Improvement | Benefit | EDA Impact |
|-------------|---------|-----------|
| **Module Interface** | Self-documenting | Can export to HDL |
| **Type-Safe Kernels** | Compile-time verification | No runtime surprises |
| **Sim Control** | External tool integration | Works with debuggers |
| **Port Metadata** | Rich type information | Full HDL compatibility |
| **DPI/VPI Support** | Co-simulation | Mix Rube + Verilog |
| **Introspection** | Runtime queries | Verification integration |
| **Packaging** | Modular distribution | IP core support |

---

## Part 7: Backward Compatibility

**Key Principle:** Existing Rube code continues to work

```rust
// Old way (still supported)
let module = Module::New(id, None, inports, outports, kernel);

// New way (recommended)
let module = Module::New(id, None, inports, outports, kernel);
// + Must implement IModuleInterface
// + Can use type-safe kernels
// + Gets introspection for free
```

**Compatibility Layer:**
- Old `KernelKind::Custom(string)` still works
- Legacy modules get default interface
- String registry deprecated but functional
- Gradual migration path

---

## Conclusion

**Rube needs EDA compatibility and modularity enhancements to:**

1. ✅ **Integrate with industry tools** (VCS, ModelSim, Cadence)
2. ✅ **Support standard module patterns** (like SystemVerilog)
3. ✅ **Enable IP packaging** (like FPGA IP cores)
4. ✅ **Support verification** (UVM, assertions, coverage)
5. ✅ **Reduce coupling** (independent kernel/module definitions)

**This roadmap provides:**
- Self-documenting modules
- Type-safe kernels
- Simulation control protocol
- External tool integration
- Module packaging & distribution

**Effort:** ~10-12 weeks for full implementation
**Benefit:** Rube becomes industry-standard compatible while maintaining performance excellence

