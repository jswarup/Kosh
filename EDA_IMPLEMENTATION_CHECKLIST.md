# EDA Compatibility Implementation Checklist

## Pre-Implementation Decisions (MUST COMPLETE FIRST)

### Design Decision 1: Port Metadata Storage Strategy
**Decision**: How to store rich port information in const arrays?

**Options**:
- **Option A (Recommended)**: Flat arrays of `PortInterface` structs
  ```rust
  const ADDER_PORTS: [PortInterface; 2] = [
      PortInterface { name: "a", width: 32, data_type: ..., attributes: ... },
      PortInterface { name: "b", width: 32, data_type: ..., attributes: ... },
  ];
  ```
  - ✅ Simple, no extra type parameters
  - ✅ Works with const generics seamlessly
  - ✅ No associated types needed
  - ❌ Port info duplicated (name in array AND in array index)

- **Option B**: Associated types in trait
  ```rust
  pub trait IModuleInterface {
      type PortMetadata: ?Sized;
      const PORTS: &'static PortMetadata;
  }
  ```
  - ✅ More flexible for complex structures
  - ✅ Avoids const array size limits
  - ❌ Adds type parameter complexity
  - ❌ Runtime pointer indirection

- **Option C**: Macro-generated arrays with metadata
  ```rust
  define_module_interface!(AdderPipeline {
      inports: [("a", 32), ("b", 32)],
      outports: [("sum", 32), ("carry", 1)],
  });
  ```
  - ✅ Less boilerplate
  - ✅ Consistent naming
  - ❌ Less flexible for custom attributes

**Recommendation**: **Option A (Flat arrays)** for Phase 1. Simplest, no compiler surprises.

**Status**: ☐ TEAM DECISION REQUIRED

---

### Design Decision 2: IKernel Type Parameters

**Question**: Should `IKernel` trait be generic over kernel instance or just static signature?

**Options**:
- **Option A (Recommended)**: Static trait with associated const
  ```rust
  pub trait IKernel {
      const SIGNATURE: KernelSignature;  // Compile-time, no allocations
      fn execute(&self, inputs: &[Reg], outputs: &mut [Reg]) -> Result<(), KernelError>;
  }
  ```
  - ✅ Zero allocations
  - ✅ Signature available at compile time
  - ✅ Simple to migrate existing kernels

- **Option B**: Generic parameter for instance type
  ```rust
  pub struct KernelRegistry<K: IKernel> { ... }
  ```
  - ✅ More flexible
  - ❌ Adds monomorphization bloat
  - ❌ Complicates module system

**Recommendation**: **Option A (associated const)**

**Status**: ☐ TEAM DECISION REQUIRED

---

### Design Decision 3: Introspection Overhead

**Question**: Should introspection API be always-available or feature-gated?

**Options**:
- **Option A (Recommended)**: Always available, minimal overhead
  ```rust
  impl<IN, OUT, SUBS, S> Module<IN, OUT, SUBS, S> {
      pub fn interface(&self) -> &'static ModuleInterface { /* O(1), no alloc */ }
      pub fn hierarchy_path(&self) -> String { /* Allocated only on demand */ }
  }
  ```
  - ✅ No feature-flag complexity
  - ✅ Available for testing, debugging
  - ✅ Can't accidentally be disabled in critical code

- **Option B**: Feature-gated
  ```rust
  #[cfg(feature = "introspection")]
  pub fn hierarchy_path(&self) -> String { ... }
  ```
  - ✅ Can remove from release builds if needed
  - ❌ Adds compile-time complexity
  - ❌ Tests may behave differently

**Recommendation**: **Option A (always available)**

**Status**: ☐ TEAM DECISION REQUIRED

---

### Design Decision 4: Backward Compatibility Strategy

**Question**: How to handle existing `KernelKind::Custom("string")` code during migration?

**Options**:
- **Option A (Recommended)**: Deprecation wrapper
  ```rust
  // Phase 2: Add new IKernel system
  pub enum KernelKind {
      Fast(KernelOp),
      Behavioral(Arc<dyn Fn>),
      Custom(&'static str),  // ← Still works but deprecated
      #[new] Trait(Arc<dyn IKernel>),  // ← New way
  }

  // Old code still works:
  Module::New(..., KernelKind::Custom("BusAdder32"))?;

  // New code uses:
  Module::New(..., KernelKind::Trait(Arc::new(BusAdder32)))?;
  ```
  - ✅ Doesn't break existing code
  - ✅ Clear migration path
  - ✅ Can deprecate gradually

- **Option B**: Hard break, force immediate migration
  - ❌ Breaks all existing code
  - ❌ Higher risk
  - ✅ Simpler implementation

**Recommendation**: **Option A (deprecation wrapper)**

**Status**: ☐ TEAM DECISION REQUIRED

---

### Design Decision 5: Phase 4 Simulation Control Protocol

**Question**: Should simulation control (pause/step/breakpoint) be optional or core?

**Options**:
- **Option A (Recommended)**: Core trait, but operations can be no-ops
  ```rust
  pub trait ISimulationController {
      fn execute(&mut self, cmd: SimulationCommand) -> Result<SimulationEvent, SimError>;
  }

  // Default impl for non-controllable engines
  impl ISimulationController for BasicSimEngine {
      fn execute(&mut self, cmd: SimulationCommand) -> Result<SimulationEvent, SimError> {
          match cmd {
              SimulationCommand::Run(n) => { for _ in 0..n { self.Drive()?; } Ok(Event::Completed) }
              SimulationCommand::Pause => Ok(Event::Paused),  // No-op for basic engine
              _ => Err(SimError::NotSupported),
          }
      }
  }
  ```
  - ✅ Unified interface
  - ✅ Flexible per engine
  - ✅ No feature flags needed

**Recommendation**: **Option A (core trait with selective support)**

**Status**: ☐ TEAM DECISION REQUIRED

---

## Phase 0: Validation & Setup (1 week)

Before starting Phase 1, ensure foundation is solid.

### Task 0.1: Validate Const-Generic Port Arrays
- [ ] Create test: `Module<2, 1, 3, Construction>` with 2 inports, 1 outport
- [ ] Verify: Can initialize `_InPorts: [PortSpec; 2]` in const context
- [ ] Benchmark: Compare memory layout vs current HierModule
- [ ] Document: Show const array initialization pattern

**Acceptance Criteria**:
- ✅ Code compiles without warnings
- ✅ Memory usage ≤ 5% increase vs baseline
- ✅ Pattern documented with examples

**Owner**: @
**Deadline**: Week 1, Day 2

---

### Task 0.2: Set Up PortInterface Struct
- [ ] Define `PortInterface` with all required fields
- [ ] Implement `PortInterface::input()`, `PortInterface::output()`, `PortInterface::clock()`, `PortInterface::reset()`
- [ ] Implement `PortInterface::to_systemverilog_port()` method
- [ ] Create comprehensive unit tests

**Code Skeleton**:
```rust
// rube/interface.rs (NEW FILE)

#[derive(Clone, Debug)]
pub struct PortInterface {
    pub name: &'static str,
    pub width: usize,
    pub data_type: DataType,
    pub direction: PortDir,
    pub bus_type: BusType,
    pub attributes: PortAttributes,
    pub documentation: Option<&'static str>,
}

impl PortInterface {
    pub fn input(name: &'static str, width: usize, doc: Option<&'static str>) -> Self {
        Self {
            name,
            width,
            data_type: DataType::Logic(width),
            direction: PortDir::In,
            bus_type: BusType::Bus(width),
            attributes: PortAttributes::default(),
            documentation: doc,
        }
    }

    pub fn to_systemverilog_port(&self) -> String {
        match self.bus_type {
            BusType::Bus(w) if w > 1 => format!("    input [{:1}:0] {}", w - 1, self.name),
            BusType::Single => format!("    input {}", self.name),
            _ => panic!("Unsupported"),
        }
    }
}

#[test]
fn test_port_interface_creation() {
    let port = PortInterface::input("data", 32, Some("Input data"));
    assert_eq!(port.width, 32);
    assert_eq!(port.name, "data");
}

#[test]
fn test_to_systemverilog() {
    let port = PortInterface::input("data", 32, None);
    assert_eq!(port.to_systemverilog_port(), "    input [31:0] data");
}
```

**Acceptance Criteria**:
- ✅ All basic constructors working
- ✅ SystemVerilog export produces valid HDL
- ✅ All tests passing
- ✅ No clippy warnings

**Owner**: @
**Deadline**: Week 1, Day 3

---

### Task 0.3: Create ModuleInterface Trait
- [ ] Define `ModuleInterface` struct (name, version, inports, outports, parameters)
- [ ] Implement `IModuleInterface` trait for adoption
- [ ] Implement `ModuleInterface::validate_ports()` (uniqueness, width consistency)
- [ ] Implement `ModuleInterface::to_systemverilog()` full module export
- [ ] Create validation tests

**Code Skeleton**:
```rust
// Add to rube/interface.rs

#[derive(Clone, Debug)]
pub struct ModuleInterface {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub vendor: Option<&'static str>,
    pub inports: &'static [PortInterface],
    pub outports: &'static [PortInterface],
    pub parameters: &'static [ParameterInterface],
}

pub trait IModuleInterface {
    fn interface() -> &'static ModuleInterface;

    fn name(&self) -> &str {
        Self::interface().name
    }
}

impl ModuleInterface {
    pub fn validate_ports(&self) -> Result<(), InterfaceError> {
        // Check uniqueness
        let mut names = std::collections::HashSet::new();
        for port in self.inports.iter().chain(self.outports.iter()) {
            if !names.insert(port.name) {
                return Err(InterfaceError::DuplicatePortName(port.name));
            }
        }
        Ok(())
    }

    pub fn to_systemverilog(&self) -> String {
        let mut sv = format!("module {} (\n", self.name);
        // ... generate SystemVerilog module definition
        sv.push_str(");\nendmodule\n");
        sv
    }
}

#[test]
fn test_module_interface_validation() {
    // Ensure no duplicate ports
}

#[test]
fn test_systemverilog_export() {
    // Verify valid SystemVerilog output
}
```

**Acceptance Criteria**:
- ✅ Validation catches duplicate port names
- ✅ SystemVerilog export is valid (parseable by Verilator)
- ✅ All tests passing
- ✅ No allocations in hot path

**Owner**: @
**Deadline**: Week 1, Day 4

---

### Task 0.4: Update Module Struct to Use PortInterface
- [ ] Change `Module._InPorts: Vec<PortSpec>` → `[PortInterface; IN]`
- [ ] Change `Module._OutPorts: Vec<PortSpec>` → `[PortInterface; OUT]`
- [ ] Update all existing code that touches ports
- [ ] Verify const-generic compilation still works
- [ ] Update existing tests

**Acceptance Criteria**:
- ✅ `Module<2, 1, 3, Sealed>` compiles
- ✅ Port access syntax unchanged (backward compatible)
- ✅ All existing tests pass
- ✅ No performance regression (≤2%)

**Owner**: @
**Deadline**: Week 1, Day 5

---

### Task 0.5: Create Phase Completion Report
- [ ] Document design decisions made
- [ ] List all code files modified
- [ ] Performance baseline measurements
- [ ] Identify any blockers for Phase 1
- [ ] Get team sign-off to proceed

**Deliverables**:
- Summary document listing all changes
- Performance comparison report
- Any architectural adjustments needed
- Go/No-Go decision for Phase 1

**Owner**: @
**Deadline**: Week 1, Day 5 EOD

**Status**: ☐ Ready for review

---

---

## Phase 1: Module Interface Implementation (2-3 weeks)

### Task 1.1: Implement IModuleInterface for All Standard Modules
- [ ] `BusAdder32` → define const `BUSADDER32_INTERFACE`
- [ ] `DLatch` → define const `DLATCH_INTERFACE`
- [ ] `Fifo` (synchronous) → define const `FIFO_INTERFACE`
- [ ] `NandGate`, `AndGate`, `OrGate`, ... → define interfaces
- [ ] Create helper macro to reduce boilerplate

**Code Skeleton**:
```rust
// In gates.rs

#[macro_export]
macro_rules! define_gate_interface {
    ($gate_type:ty, $name:expr, $version:expr, $description:expr,
     inports: [$($in_name:expr, $in_width:expr),*],
     outports: [$($out_name:expr, $out_width:expr),*]) => {
        const GATE_INTERFACE: ModuleInterface = ModuleInterface {
            name: $name,
            version: $version,
            description: $description,
            vendor: Some("OrioleDesigns"),
            inports: &[
                $(PortInterface::input($in_name, $in_width, None),)*
            ],
            outports: &[
                $(PortInterface::output($out_name, $out_width, None),)*
            ],
            parameters: &[],
        };

        impl IModuleInterface for $gate_type {
            fn interface() -> &'static ModuleInterface {
                &GATE_INTERFACE
            }
        }
    };
}

// Usage:
define_gate_interface!(
    BusAdder32,
    "bus_adder_32",
    "1.0.0",
    "32-bit binary adder",
    inports: [("a", 32), ("b", 32)],
    outports: [("sum", 32), ("carry", 1)]
);
```

**Acceptance Criteria**:
- ✅ All standard kernels have interfaces
- ✅ Macro reduces code by >50%
- ✅ `cargo test --lib` all passing
- ✅ No runtime allocations introduced

**Owner**: @
**Deadline**: Week 2, Day 2

---

### Task 1.2: Implement Introspection API
- [ ] Create `ModuleIntrospection` struct (name, hierarchy_path, port list)
- [ ] Implement `IModuleIntrospection` trait
- [ ] Add `list_inports()`, `list_outports()` methods returning `Vec<PortIntrospection>`
- [ ] Add `get_hierarchy_path()` method
- [ ] Add optional `ModuleStatistics` collection

**Code Skeleton**:
```rust
// rube/introspect.rs (NEW FILE)

pub trait IModuleIntrospection {
    fn interface(&self) -> &'static ModuleInterface;
    fn hierarchy_path(&self) -> String;
    fn list_inports(&self) -> Vec<PortIntrospection>;
    fn list_outports(&self) -> Vec<PortIntrospection>;
}

#[derive(Clone, Debug)]
pub struct PortIntrospection {
    pub name: String,
    pub width: usize,
    pub data_type: DataType,
    pub direction: PortDir,
    pub documentation: Option<&'static str>,
}

impl<IN, OUT, SUBS, S> IModuleIntrospection for Module<IN, OUT, SUBS, S> {
    fn interface(&self) -> &'static ModuleInterface {
        Self::interface_static()  // From IModuleInterface trait
    }

    fn hierarchy_path(&self) -> String {
        // Walk parent chain to build path
        format!("top.{}.{}", self._Name, /* parent path */)
    }

    fn list_inports(&self) -> Vec<PortIntrospection> {
        self._InPorts.iter().map(|p| PortIntrospection {
            name: p.name.to_string(),
            width: p.width,
            // ... other fields
        }).collect()
    }
}

#[test]
fn test_introspection_paths() {
    let module = BusAdder32::interface();
    assert_eq!(module.list_inports().len(), 2);
    assert_eq!(module.list_outports().len(), 2);
}
```

**Acceptance Criteria**:
- ✅ Can introspect any module's interface at runtime
- ✅ Hierarchy paths correct for nested modules
- ✅ All tests passing
- ✅ Zero allocations for interface queries

**Owner**: @
**Deadline**: Week 2, Day 4

---

### Task 1.3: Implement ModuleInterface Export to SystemVerilog
- [ ] Extend `ModuleInterface::to_systemverilog()` to be fully correct
- [ ] Handle parameterized modules (generate parameter block)
- [ ] Handle array ports (generate [N:0] syntax correctly)
- [ ] Add optional documentation comments in generated code
- [ ] Create test suite that validates output vs. Verilator parser

**Code Skeleton**:
```rust
impl ModuleInterface {
    pub fn to_systemverilog(&self) -> String {
        let mut sv = format!("// {}\n", self.description);
        sv.push_str(&format!("module {} (\n", self.name));

        // Parameters
        if !self.parameters.is_empty() {
            sv.push_str("    #(\n");
            for param in self.parameters {
                sv.push_str(&format!("        parameter {} {}\n",
                    param.param_type_str(), param.name));
            }
            sv.push_str("    )\n");
        }

        // Ports
        let mut first = true;
        for port in self.inports {
            if !first { sv.push(','); }
            sv.push_str(&format!("    {} {} {}",
                port.direction_str(),
                port.data_type_str(),
                port.name));
            first = false;
        }
        for (i, port) in self.outports.iter().enumerate() {
            if i > 0 || !self.inports.is_empty() { sv.push(','); }
            sv.push_str(&format!("    {} {} {}",
                port.direction_str(),
                port.data_type_str(),
                port.name));
        }

        sv.push_str("\n);\n");
        sv.push_str("endmodule\n");
        sv
    }
}

#[test]
fn test_systemverilog_adder_export() {
    let sv = BusAdder32::interface().to_systemverilog();
    assert!(sv.contains("module bus_adder_32"));
    assert!(sv.contains("input [31:0] a"));
    assert!(sv.contains("input [31:0] b"));
    assert!(sv.contains("output [31:0] sum"));
    assert!(sv.contains("output carry"));
}

#[test]
fn test_systemverilog_verilator_compatible() {
    let sv = BusAdder32::interface().to_systemverilog();
    // Try to parse with Verilator parser
    let verilator_output = std::process::Command::new("verilator")
        .args(&["--lint-only", "-"])
        .stdin(std::process::Stdio::piped())
        .output()
        .expect("Verilator not found");

    assert!(verilator_output.status.success(),
        "Generated SystemVerilog invalid: {}",
        String::from_utf8_lossy(&verilator_output.stderr));
}
```

**Acceptance Criteria**:
- ✅ Generated SystemVerilog parses with Verilator (if available)
- ✅ All port widths and directions correct
- ✅ Parameters exported correctly
- ✅ Documentation in comments preserved
- ✅ All tests passing

**Owner**: @
**Deadline**: Week 3, Day 1

---

### Task 1.4: Backward Compatibility Validation
- [ ] Ensure existing code using `Module` still compiles
- [ ] Verify no performance regression (≤2% overhead)
- [ ] Run full test suite (cargo test)
- [ ] Benchmark comparison: old vs new port metadata

**Acceptance Criteria**:
- ✅ All existing tests pass
- ✅ No breaking changes to public API
- ✅ Performance within 2% of baseline
- ✅ Binary size increase <10%

**Owner**: @
**Deadline**: Week 3, Day 2

---

### Task 1.5: Phase 1 Documentation & Examples
- [ ] Update HIERARCHICAL_MODULE_FRAMEWORK.md with Phase 1 completion status
- [ ] Create new file: `PHASE1_MODULE_INTERFACES.md` with:
  - Quick start guide for adding interfaces to new modules
  - Examples: simple gate, complex module with parameters
  - Common patterns and pitfalls
  - Troubleshooting guide
- [ ] Add rustdoc comments to all public items
- [ ] Generate docs: `cargo doc --open`

**Deliverables**:
- Module interface quick start guide
- 3-5 worked examples
- Common questions & answers
- Rustdoc-style API documentation

**Owner**: @
**Deadline**: Week 3, Day 3

---

### ✅ Phase 1 Gate Review

**Checklist**:
- [ ] All tasks 1.1-1.5 complete
- [ ] All tests passing (`cargo test` clean)
- [ ] No clippy warnings
- [ ] Performance benchmarks show ≤2% overhead
- [ ] Code reviewed by 2 team members
- [ ] Documentation complete and reviewed
- [ ] Example modules working correctly

**Sign-Off**: ______________________  **Date**: _________

**Go/No-Go for Phase 2**: ☐ GO  ☐ NO-GO

---

---

## Phase 2: Type-Safe Kernel System (2-3 weeks)

### Task 2.1: Define IKernel Trait
- [ ] Create `IKernel` trait with:
  - `const NAME: &'static str`
  - `const VERSION: &'static str`
  - `const SIGNATURE: KernelSignature` (port count/types)
  - `fn execute(&self, inputs: &[Reg], outputs: &mut [Reg]) -> Result<(), KernelError>`
- [ ] Create `KernelSignature` struct (input_ports, output_ports, params)
- [ ] Add validation: port count must match signature

**Code Skeleton**:
```rust
// rube/kernel.rs (NEW FILE / extends existing)

pub struct KernelSignature {
    pub input_ports: usize,
    pub output_ports: usize,
    pub parameters: &'static [ParameterInterface],
}

pub trait IKernel: Send + Sync {
    const NAME: &'static str;
    const VERSION: &'static str;
    const SIGNATURE: KernelSignature;

    fn execute(&self, inputs: &[Reg], outputs: &mut [Reg]) -> Result<(), KernelError>;

    fn validate_signature(&self) -> Result<(), KernelError> {
        if inputs.len() != Self::SIGNATURE.input_ports {
            return Err(KernelError::InputPortMismatch);
        }
        if outputs.len() != Self::SIGNATURE.output_ports {
            return Err(KernelError::OutputPortMismatch);
        }
        Ok(())
    }
}

pub enum KernelError {
    InputPortMismatch,
    OutputPortMismatch,
    ExecutionFailed(String),
}
```

**Acceptance Criteria**:
- ✅ Trait is clean and intuitive
- ✅ Can be implemented by existing kernels
- ✅ Signature validation works
- ✅ All tests passing

**Owner**: @
**Deadline**: Week 4, Day 1

---

### Task 2.2: Migrate Standard Kernels to IKernel
- [ ] `BusAdder32` implements `IKernel`
- [ ] `DLatch` implements `IKernel`
- [ ] `FIFO` implements `IKernel`
- [ ] `NandGate`, `AndGate`, ... implement `IKernel`
- [ ] Update `execute()` method to match signature
- [ ] Add comprehensive unit tests per kernel

**Code Skeleton**:
```rust
// In gates.rs or separate module

pub struct BusAdder32;

impl IKernel for BusAdder32 {
    const NAME: &'static str = "bus_adder_32";
    const VERSION: &'static str = "1.0.0";
    const SIGNATURE: KernelSignature = KernelSignature {
        input_ports: 2,
        output_ports: 2,
        parameters: &[],
    };

    fn execute(&self, inputs: &[Reg], outputs: &mut [Reg]) -> Result<(), KernelError> {
        if inputs.len() != 2 || outputs.len() != 2 {
            return Err(KernelError::PortMismatch);
        }

        let a = inputs[0].Val();
        let b = inputs[1].Val();
        let sum = a.wrapping_add(b);
        let carry = (a + b) > u32::MAX as u64;

        outputs[0] = Reg::FromU32(U32(sum as u32));
        outputs[1] = Reg::FromBool(carry);

        Ok(())
    }
}

// Link to ModuleInterface
impl IModuleInterface for BusAdder32 {
    fn interface() -> &'static ModuleInterface {
        &BUSADDER32_INTERFACE
    }
}

#[test]
fn test_busadder32_kernel() {
    let kernel = BusAdder32;
    let inputs = [Reg::FromU32(U32(10)), Reg::FromU32(U32(20))];
    let mut outputs = [Reg::default(), Reg::default()];

    kernel.execute(&inputs, &mut outputs).expect("execute failed");

    assert_eq!(outputs[0].Val(), 30);  // sum
    assert!(!outputs[1].AsBoolean()); // no carry
}

#[test]
fn test_busadder32_with_carry() {
    let kernel = BusAdder32;
    let inputs = [Reg::FromU32(U32(u32::MAX)), Reg::FromU32(U32(1))];
    let mut outputs = [Reg::default(), Reg::default()];

    kernel.execute(&inputs, &mut outputs).expect("execute failed");

    // sum overflows but wraps to 0
    assert_eq!(outputs[0].Val(), 0);
    // carry should be set
    assert!(outputs[1].AsBoolean());
}
```

**Acceptance Criteria**:
- ✅ All standard kernels implement `IKernel`
- ✅ Each has comprehensive unit tests
- ✅ Signature validation working
- ✅ Execution behavior unchanged from original
- ✅ All tests passing (`cargo test`)

**Owner**: @
**Deadline**: Week 4, Day 3

---

### Task 2.3: Create Deprecation Wrapper for KernelKind::Custom
- [ ] Add new variant: `KernelKind::Trait(Arc<dyn IKernel>)`
- [ ] Keep `KernelKind::Custom(string)` but mark `#[deprecated]`
- [ ] Implement bridge from string registry to `IKernel`
- [ ] Update `Layout` to handle both old and new styles
- [ ] Add migration guide documentation

**Code Skeleton**:
```rust
// In module.rs or kernel.rs

pub enum KernelKind {
    Fast(KernelOp),
    Behavioral(Arc<dyn Fn>),
    Custom(&'static str),  // ← Keep for backward compat, mark deprecated
    #[new]
    Trait(Arc<dyn IKernel>),  // ← New way
    None,
}

// Bridge implementation
pub struct StringKernelAdapter {
    name: &'static str,
    factory: Arc<dyn Fn() -> Arc<dyn IKernel>>,
}

// For existing code:
// KernelKind::Custom("BusAdder32") → looks up BusAdder32 in registry → wraps as StringKernelAdapter
// For new code:
// KernelKind::Trait(Arc::new(BusAdder32)) → direct usage

#[deprecated(
    since = "0.6.0",
    note = "Use KernelKind::Trait with IKernel implementations instead"
)]
pub fn create_custom_kernel(name: &'static str) -> KernelKind {
    KernelKind::Custom(name)
}

#[test]
fn test_backward_compat_custom_kernel() {
    let old_style = KernelKind::Custom("BusAdder32");
    let new_style = KernelKind::Trait(Arc::new(BusAdder32));
    // Both should execute identically
}
```

**Acceptance Criteria**:
- ✅ Old code continues to work
- ✅ Deprecation warning shown when using Custom variant
- ✅ Bridge converts string → trait seamlessly
- ✅ No performance regression
- ✅ All existing tests still pass

**Owner**: @
**Deadline**: Week 4, Day 4

---

### Task 2.4: Type-Safe Kernel Registry
- [ ] Create `KernelRegistry` trait/struct
- [ ] Implement `register(name, kernel_factory)` method
- [ ] Implement `get(name) -> Option<Arc<dyn IKernel>>`
- [ ] Validate kernel signature on registration
- [ ] Add support for parameterization

**Code Skeleton**:
```rust
// rube/kernel.rs

pub struct KernelRegistry {
    kernels: HashMap<&'static str, Arc<dyn IKernel>>,
}

impl KernelRegistry {
    pub fn new() -> Self {
        Self {
            kernels: HashMap::new(),
        }
    }

    pub fn register(&mut self, kernel: Arc<dyn IKernel>) -> Result<(), RegistryError> {
        kernel.validate_signature()?;
        self.kernels.insert(kernel.NAME, kernel);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn IKernel>> {
        self.kernels.get(name).cloned()
    }

    pub fn default() -> Self {
        let mut reg = Self::new();
        reg.register(Arc::new(BusAdder32)).unwrap();
        reg.register(Arc::new(DLatch)).unwrap();
        // ... add all standard kernels
        reg
    }
}

#[test]
fn test_kernel_registry() {
    let mut registry = KernelRegistry::new();
    let kernel = Arc::new(BusAdder32);
    registry.register(kernel).expect("registration failed");

    let retrieved = registry.get("bus_adder_32").expect("kernel not found");
    assert_eq!(retrieved.NAME, "bus_adder_32");
}
```

**Acceptance Criteria**:
- ✅ Type-safe kernel lookup
- ✅ Signature validated on registration
- ✅ Default registry includes all standard kernels
- ✅ Can be extended with custom kernels
- ✅ All tests passing

**Owner**: @
**Deadline**: Week 4, Day 5

---

### Task 2.5: Phase 2 Completion & Gate Review
- [ ] All standard kernels migrated to IKernel
- [ ] Backward compatibility layer working
- [ ] Type-safe registry functional
- [ ] All tests passing
- [ ] Code reviewed by 2 team members
- [ ] Performance benchmarks ≤2% overhead
- [ ] Documentation updated

**Deliverables**:
- Migration guide: "Converting Custom Kernels to IKernel"
- Performance benchmark report
- Architecture diagram: old vs new kernel system
- Rustdoc for IKernel trait

**Owner**: @
**Deadline**: Week 5, Day 2

**Sign-Off**: ______________________  **Date**: _________

**Go/No-Go for Phase 3**: ☐ GO  ☐ NO-GO

---

---

## Phase 3: Simulation Control Protocol (2-3 weeks)

### Task 3.1: Define ISimulationController Trait
- [ ] Create `SimulationCommand` enum (Run, Step, Pause, Reset, Query)
- [ ] Create `SimulationEvent` enum (Paused, Completed, BreakpointHit, etc.)
- [ ] Create `ISimulationController` trait
- [ ] Implement for existing `SimEngine`

**Code Skeleton**:
```rust
// rube/sim_ctrl.rs (NEW FILE)

pub enum SimulationCommand {
    Run(usize),                    // Run N cycles
    Step(usize),                   // Step N cycles and pause
    Pause,                         // Pause immediately
    Reset,                         // Reset to initial state
    Query(SimulationQuery),        // Query simulation state
}

pub enum SimulationQuery {
    CycleCount,
    PortValue(PortId),
    ModuleState(ModuleId),
}

pub enum SimulationEvent {
    Paused,
    Completed,
    BreakpointHit(BreakpointId),
    Error(String),
}

pub enum SimulationValue {
    Cycle(usize),
    Register(Reg),
    Boolean(bool),
}

pub trait ISimulationController {
    fn execute(&mut self, cmd: SimulationCommand)
        -> Result<SimulationEvent, SimulationError>;
    fn query(&self, query: SimulationQuery)
        -> Result<SimulationValue, SimulationError>;
    fn set_breakpoint(&mut self, bp: Breakpoint) -> BreakpointId;
    fn clear_breakpoint(&mut self, bp_id: BreakpointId) -> Result<(), SimulationError>;
}

pub struct Breakpoint {
    pub target: BreakpointTarget,
    pub condition: Option<BreakpointCondition>,
}

pub enum BreakpointTarget {
    PortChange(PortId),
    PortValue(PortId, Reg),
    CycleCount(usize),
}

pub enum BreakpointCondition {
    Always,
    OncePerCycle,
    Custom(String),  // e.g., "value > 100"
}
```

**Acceptance Criteria**:
- ✅ Clear, intuitive API
- ✅ Can express common debugging scenarios
- ✅ Extensible for future control commands
- ✅ No performance impact when not in use

**Owner**: @
**Deadline**: Week 5, Day 3

---

### Task 3.2: Implement ISimulationController for SimEngine
- [ ] Add breakpoint storage to `SimEngine`
- [ ] Implement `execute()` method for each command
- [ ] Implement `query()` method for state inspection
- [ ] Implement `set_breakpoint()` and `clear_breakpoint()`
- [ ] Add cycle stepping with pause capability

**Code Skeleton**:
```rust
// In engine.rs

impl SimEngine {
    pub fn breakpoints(&mut self) -> &mut Vec<Breakpoint> {
        &mut self._Breakpoints
    }
}

impl ISimulationController for SimEngine {
    fn execute(&mut self, cmd: SimulationCommand)
        -> Result<SimulationEvent, SimulationError>
    {
        match cmd {
            SimulationCommand::Run(n) => {
                for _ in 0..n {
                    self.Drive()?;
                    self._CycleCount += 1;

                    // Check breakpoints
                    if self.check_breakpoints()? {
                        return Ok(SimulationEvent::BreakpointHit(/* ... */));
                    }
                }
                Ok(SimulationEvent::Completed)
            }
            SimulationCommand::Step(n) => {
                for _ in 0..n {
                    self.Drive()?;
                    self._CycleCount += 1;
                }
                Ok(SimulationEvent::Paused)
            }
            SimulationCommand::Pause => Ok(SimulationEvent::Paused),
            SimulationCommand::Reset => {
                self._CycleCount = 0;
                self._Triggers.Reset();
                Ok(SimulationEvent::Completed)
            }
            SimulationCommand::Query(q) => {
                // Delegate to query() method
                self.query(q).map(|_| SimulationEvent::Completed)
            }
        }
    }

    fn query(&self, query: SimulationQuery)
        -> Result<SimulationValue, SimulationError>
    {
        match query {
            SimulationQuery::CycleCount => {
                Ok(SimulationValue::Cycle(self._CycleCount as usize))
            }
            SimulationQuery::PortValue(port_id) => {
                let trigger_id = self._PortToTrigger[port_id.0.AsUsize()];
                Ok(SimulationValue::Register(self._Triggers.Current(trigger_id)))
            }
            _ => Err(SimulationError::NotSupported),
        }
    }

    fn set_breakpoint(&mut self, bp: Breakpoint) -> BreakpointId {
        let id = BreakpointId(self._Breakpoints.len());
        self._Breakpoints.push(bp);
        id
    }
}

#[test]
fn test_simulation_run_command() {
    let layout = create_test_layout();
    let mut engine = SimEngine::Create(&layout).unwrap();

    let result = engine.execute(SimulationCommand::Run(10)).unwrap();
    assert!(matches!(result, SimulationEvent::Completed));

    let cycle_count = engine.query(SimulationQuery::CycleCount).unwrap();
    if let SimulationValue::Cycle(c) = cycle_count {
        assert_eq!(c, 10);
    }
}

#[test]
fn test_simulation_step_pause() {
    let layout = create_test_layout();
    let mut engine = SimEngine::Create(&layout).unwrap();

    let result = engine.execute(SimulationCommand::Step(5)).unwrap();
    assert!(matches!(result, SimulationEvent::Paused));

    // Can resume from pause
    let result = engine.execute(SimulationCommand::Step(5)).unwrap();
    assert!(matches!(result, SimulationEvent::Paused));
}

#[test]
fn test_breakpoint_on_cycle_count() {
    let layout = create_test_layout();
    let mut engine = SimEngine::Create(&layout).unwrap();

    let bp_id = engine.set_breakpoint(Breakpoint {
        target: BreakpointTarget::CycleCount(50),
        condition: None,
    });

    let result = engine.execute(SimulationCommand::Run(100)).unwrap();
    // Should stop at cycle 50
    assert!(matches!(result, SimulationEvent::BreakpointHit(id) if id == bp_id));
}
```

**Acceptance Criteria**:
- ✅ Can run N cycles and complete
- ✅ Can step and pause
- ✅ Can query cycle count and port values
- ✅ Breakpoints work on cycle count
- ✅ Reset functionality works
- ✅ All tests passing
- ✅ No performance overhead when not using breakpoints

**Owner**: @
**Deadline**: Week 5, Day 5

---

### Task 3.3: Breakpoint and Probe System
- [ ] Extend breakpoint support to port value changes
- [ ] Implement probe system (register named signal watches)
- [ ] Add callback support for breakpoints
- [ ] Add signal collection for waveform export
- [ ] Create `ProbeId` and `ProbeRegistry`

**Code Skeleton**:
```rust
// rube/probe.rs (NEW FILE)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProbeId(usize);

pub enum ProbeTarget {
    Port(PortId),
    InternalSignal(String),  // e.g., "pipeline.stage1.carry"
}

pub struct Probe {
    pub id: ProbeId,
    pub name: String,
    pub target: ProbeTarget,
    pub sample_every_n_cycles: usize,
}

pub struct ProbeRegistry {
    probes: HashMap<ProbeId, Probe>,
    next_id: usize,
}

impl ProbeRegistry {
    pub fn register(&mut self, name: String, target: ProbeTarget) -> ProbeId {
        let id = ProbeId(self.next_id);
        self.next_id += 1;
        self.probes.insert(id, Probe {
            id,
            name,
            target,
            sample_every_n_cycles: 1,
        });
        id
    }

    pub fn sample(&self, cycle_count: usize) -> HashMap<ProbeId, Reg> {
        // Collect values for probes at this cycle
        let mut samples = HashMap::new();
        for (id, probe) in &self.probes {
            if cycle_count % probe.sample_every_n_cycles == 0 {
                // Get value for probe target
                // samples.insert(*id, value);
            }
        }
        samples
    }
}

#[test]
fn test_probe_registration() {
    let mut registry = ProbeRegistry::new();
    let probe_id = registry.register("output".to_string(),
        ProbeTarget::Port(PortId::Out(0)));

    assert!(registry.get(probe_id).is_some());
}
```

**Acceptance Criteria**:
- ✅ Can register probes for monitoring
- ✅ Probes can be sampled at regular intervals
- ✅ Can export probe data for waveform analysis
- ✅ Probe overhead minimal (only active when registered)

**Owner**: @
**Deadline**: Week 6, Day 2

---

### Task 3.4: Phase 3 Completion & Documentation
- [ ] All control commands working
- [ ] Breakpoints functional
- [ ] Probes collecting data
- [ ] Integration with SimEngine complete
- [ ] Comprehensive test suite
- [ ] Performance benchmarks
- [ ] User documentation

**Deliverables**:
- "Simulation Control User Guide"
- Example: using breakpoints for debugging
- Example: collecting probe data for waveforms
- Rustdoc for ISimulationController trait
- Architecture diagram

**Owner**: @
**Deadline**: Week 6, Day 3

**Sign-Off**: ______________________  **Date**: _________

**Go/No-Go for Phase 4**: ☐ GO  ☐ NO-GO

---

---

## Phase 4: DPI/VPI Co-Simulation (3-4 weeks)

### Task 4.1: ISimulationSocket Trait
- [ ] Define `ISimulationSocket` trait
- [ ] Design cycle exchange protocol
- [ ] Define `RemoteSimulator` interface
- [ ] Create mock implementations for testing

**Status**: ☐ To be detailed in next update

---

## Phase 5: Module Packaging (2 weeks)

### Task 5.1: ModulePackage Manifest
- [ ] Design TOML-based manifest format
- [ ] Implement TOML parsing
- [ ] Define versioning rules
- [ ] Create example packages

**Status**: ☐ To be detailed in next update

---

---

## Success Metrics

### Code Quality
- [ ] All tests passing: `cargo test` clean
- [ ] No clippy warnings: `cargo clippy` clean
- [ ] Documentation: `cargo doc --open` compiles
- [ ] Code coverage: >80% on new modules

### Performance
- [ ] Hot-path simulation: ≤2% overhead
- [ ] Binary size: ≤15% increase
- [ ] Introspection: <1μs for interface queries
- [ ] Breakpoint checking: <1% simulation overhead

### Functionality
- [ ] All 5 phases complete
- [ ] Module interface fully self-documenting
- [ ] Kernels type-safe and validated
- [ ] Simulation controllable externally
- [ ] Can export to SystemVerilog
- [ ] Can introspect module hierarchy

### User Experience
- [ ] Clear migration guide from old to new
- [ ] Comprehensive examples for each phase
- [ ] Responsive to user feedback
- [ ] Documentation accessible and helpful

---

## Team Assignments

| Phase | Owner | Reviewer | Deadline |
|-------|-------|----------|----------|
| 0 (Validation) | TBD | TBD | Week 1, Day 5 |
| 1 (Interface) | TBD | TBD | Week 3, Day 3 |
| 2 (Kernels) | TBD | TBD | Week 5, Day 2 |
| 3 (Control) | TBD | TBD | Week 6, Day 3 |
| 4 (DPI/VPI) | TBD | TBD | Week 10, Day 2 |
| 5 (Packaging) | TBD | TBD | Week 12, Day 1 |

---

## Risk Register

### Risk 1: Const Generic Complexity
**Risk**: Compile errors difficult to debug, monomorphization bloat
**Mitigation**: Phase 0 validation, incremental type parameter addition
**Owner**: @
**Status**: ☐ Monitored

### Risk 2: Performance Regression
**Risk**: Introspection, breakpoints, probes slow down simulation
**Mitigation**: Benchmarking after each phase, feature-gate if needed
**Owner**: @
**Status**: ☐ Monitored

### Risk 3: Breaking Changes
**Risk**: Existing user code breaks during migration
**Mitigation**: Deprecation wrappers, backward compatibility layer
**Owner**: @
**Status**: ☐ Monitored

### Risk 4: Scope Creep
**Risk**: Additional features added, delays phases
**Mitigation**: Strict phase boundaries, defer to Phase 6+
**Owner**: @
**Status**: ☐ Monitored

---

## Notes & Decisions Log

| Date | Decision | Rationale | Status |
|------|----------|-----------|--------|
| | Design: PortInterface stored in const arrays | Simple, no extra type params | ✅ Approved |
| | Design: IKernel uses associated const | Zero allocations | ✅ Approved |
| | Design: Backward compat wrapper for Custom | No breaking changes | ✅ Approved |
| | | | |

---

## Go/No-Go Gate Summary

| Gate | Phase | Status | Decision | Date |
|------|-------|--------|----------|------|
| Pre-Implementation | 0 | ☐ Ready | ☐ GO / ☐ NO-GO | ___ |
| Phase 1 | Interface | ☐ Ready | ☐ GO / ☐ NO-GO | ___ |
| Phase 2 | Kernels | ☐ Ready | ☐ GO / ☐ NO-GO | ___ |
| Phase 3 | Control | ☐ Ready | ☐ GO / ☐ NO-GO | ___ |
| Phase 4 | DPI/VPI | ☐ Ready | ☐ GO / ☐ NO-GO | ___ |
| Phase 5 | Packaging | ☐ Ready | ☐ GO / ☐ NO-GO | ___ |

---

**Document Created**: 2026-09-05
**Last Updated**: 2026-09-05
**Version**: 1.0

