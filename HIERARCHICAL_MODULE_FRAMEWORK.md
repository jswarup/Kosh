# Hierarchical Module Framework Migration Plan

## Executive Summary

This document outlines the migration from the current Rube simulation framework to a **Standard Hierarchical Module Framework** that implements proper encapsulation, clear port semantics, and composable module hierarchy. The new framework enables:

1. **Proper Module Encapsulation**: Internal submodules and interconnections are invisible outside the module
2. **Clear Port Contracts**: Each module declares inports (inputs) and outports (outputs), top module has none
3. **Constructor-Driven Composition**: Module constructors define all internal interconnections
4. **Test-Driven Architecture**: Each test uses a `RubeTest_XXXX` top-module with I/O managed by CoroKernels
5. **Scalable Hierarchy**: Deep module hierarchies without exposure of internal structure

---

## Current State Analysis

### Existing Strengths
- ✅ Excellent kernel abstraction (`KernelKind` enum)
- ✅ Strong SoA temporal storage (`TriggerWad`)
- ✅ Module identity system (`ModuleId`, `IModule`)
- ✅ Clear trait-based design (`IModule`, `INetlist`, `ITriggerWad`)
- ✅ Flexible kernel types (Fast/Behavioral/Custom/Coro)

### Current Limitations

| Issue | Impact | Example |
|-------|--------|---------|
| **No inport/outport distinction** | Internals exposed to external connections | Can directly connect to submodule ports from parent scope |
| **Module hierarchy is optional** | No enforcement of encapsulation | Parent module can access internal submodule state |
| **Constructor-less composition** | Complex manual interconnection | Must manually wire submodules after creation |
| **No visibility rules** | All ports are globally accessible | No way to hide internal signals |
| **Test structure unclear** | Tests aren't explicitly layered | Test harnesses mixed with design under test |

### Code Example - Current Approach
```rust
// Current: Flat net composition
let mut layout = Layout::New();
let adder = layout.AddModule("BusAdder32", 32, 32, 32, KernelKind::Custom("BusAdder32"));
let latch1 = layout.AddModule("Latch1", 32, 0, 32, KernelKind::Custom("DLatch"));
let latch2 = layout.AddModule("Latch2", 32, 0, 32, KernelKind::Custom("DLatch"));

// Problem: No encapsulation - all wiring is global
layout.Connect(adder.OutPort(0), latch1.InPort(0))?;
layout.Connect(latch1.OutPort(0), latch2.InPort(0))?;
// External code can also connect directly to latch1.OutPort(0)!
```

---

## Proposed Hierarchical Architecture

### 1. Core Concepts

#### **Module Structure**
```
┌─ Module (TopModule)
│  ├─ Kernel (optional, None for containers)
│  ├─ InPorts: []  (always empty for TopModule)
│  ├─ OutPorts: [] (always empty for TopModule)
│  └─ SubModules
│     ├─ SubModule_A
│     │  ├─ Kernel: CoroKernel (I/O driver)
│     │  ├─ InPorts: [input1, input2, ...]
│     │  ├─ OutPorts: [output1, output2, ...]
│     │  └─ SubModules: []  (leaf node)
│     │
│     ├─ SubModule_B (container)
│     │  ├─ Kernel: None
│     │  ├─ InPorts: [A_in, B_in]
│     │  ├─ OutPorts: [A_out, B_out]
│     │  └─ SubModules
│     │     ├─ Internal_X (hidden)
│     │     └─ Internal_Y (hidden)
│     │
│     └─ SubModule_C
│        └─ (similar structure)
```

#### **Port Visibility Rules**

| Context | Can Access | Cannot Access |
|---------|-----------|---------------|
| External to Module | inports, outports | submodules, internal connections |
| Within Module Constructor | all internal nodes | nothing (being constructed) |
| After Module Sealed | inports, outports only | internal submodules |

#### **Module Lifecycle**
```
1. Module::New("name", inport_spec, outport_spec)
   ↓ Create with declared ports
2. Constructor {
     - Create submodules
     - Connect submodule ports (internal)
     - Connect submodules to this module's inports/outports
   }
   ↓ All internal connections made
3. Module::Seal()
   ↓ Declare finished, no more internal changes
4. Use in parent or layout
   ↓ External can only see inports/outports
```

---

## 2. API Design

### 2.1 Core Types

```rust
/// Module port specification
#[derive(Clone, Debug)]
pub struct PortSpec {
    pub name: String,
    pub width: U32,
    pub direction: PortDir,
}

/// Hierarchical Module
#[derive(Clone, Debug)]
pub struct Module {
    pub _Id: ModuleId,
    pub _Parent: Option<ModuleId>,
    pub _Name: String,
    
    // External interface (visible to parent)
    pub _InPorts: Vec<PortSpec>,      // New: explicit inports
    pub _OutPorts: Vec<PortSpec>,     // New: explicit outports
    
    // Internal structure (invisible to parent)
    pub _SubModules: Stash<ModuleId>,  // Hidden
    pub _Connections: Netlist,         // Hidden internal connectivity
    pub _Kernel: KernelKind,           // Optional
    pub _IsSealed: bool,               // Prevents further modification
}

/// Port access (scoped to owner)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortAccess {
    InPort(usize),   // Index into _InPorts
    OutPort(usize),  // Index into _OutPorts
}

/// Module reference with visibility context
pub struct ModuleHandle {
    pub id: ModuleId,
    pub visibility: Visibility,
}

pub enum Visibility {
    Internal,    // Can access full internal structure
    External,    // Can only access inports/outports
}
```

### 2.2 Builder API

```rust
impl Module {
    /// Create a new module with declared interface
    pub fn New(
        name: &str,
        inport_specs: Vec<PortSpec>,
        outport_specs: Vec<PortSpec>,
    ) -> Self {
        // Module starts unsealed with empty submodules
        // Constructor will be called immediately
    }

    /// Add a submodule (only valid in constructor)
    pub fn AddSubModule(
        &mut self,
        name: &str,
        kernel: KernelKind,
    ) -> Result<ModuleId, HierarchyError> {
        // Only callable before Seal()
        // Returns submodule ID for internal wiring
    }

    /// Connect submodule port to this module's port
    /// (Maps internal to external boundary)
    pub fn BindInPort(
        &mut self,
        this_inport: usize,
        submodule_id: ModuleId,
        submodule_outport: usize,
    ) -> Result<(), HierarchyError> {
        // Validates port exists and is appropriate
        // Internal connection - hidden from parent
    }

    pub fn BindOutPort(
        &mut self,
        this_outport: usize,
        submodule_id: ModuleId,
        submodule_inport: usize,
    ) -> Result<(), HierarchyError> {
        // Same validation
    }

    /// Connect two submodule ports
    pub fn ConnectSubModules(
        &mut self,
        src_module: ModuleId,
        src_port: usize,
        dst_module: ModuleId,
        dst_port: usize,
    ) -> Result<(), HierarchyError> {
        // Internal interconnect only
    }

    /// Seal the module - no more internal changes
    pub fn Seal(mut self) -> Result<Self, HierarchyError> {
        // Validate all submodule connections are complete
        // Mark _IsSealed = true
        // Return owned module
    }
}
```

### 2.3 Top-Level Composition

```rust
/// Builder for composing module hierarchy
pub struct HierarchyBuilder {
    root_module: Module,
    // Internal tracking
}

impl HierarchyBuilder {
    pub fn New(name: &str) -> Self {
        // Create top-level module with no ports
        let root = Module::New(name, vec![], vec![]);
        HierarchyBuilder { root_module: root }
    }

    pub fn AddModule<F>(mut self, name: &str, kernel: KernelKind, setup: F) -> Result<Self, HierarchyError>
    where
        F: FnOnce(&mut Module) -> Result<(), HierarchyError>,
    {
        let mut module = Module::New(name, vec![], vec![]);
        setup(&mut module)?;
        let sealed = module.Seal()?;
        self.root_module.AddSubModule(name, sealed)?;
        Ok(self)
    }

    pub fn Build(self) -> Result<Layout, HierarchyError> {
        // Flatten hierarchy into Layout for simulation
        // Generate all triggers, compile warps, etc.
    }
}
```

---

## 3. Test Framework

### 3.1 Test Top-Module Pattern

Each test has a dedicated `RubeTest_XXXX` top-module:

```rust
pub struct RubeTest_AdderChain {
    top_module: Module,
    // Corokernel instances for test control
}

impl RubeTest_AdderChain {
    pub fn New() -> Result<Self, HierarchyError> {
        let mut top = Module::New("RubeTest_AdderChain", vec![], vec![]);

        // Add submodules
        let adder1_id = top.AddSubModule("Adder1", KernelKind::Custom("BusAdder32"))?;
        let adder2_id = top.AddSubModule("Adder2", KernelKind::Custom("BusAdder32"))?;
        let test_io = top.AddSubModule(
            "TestIO",
            KernelKind::Coro(create_test_io_kernel())
        )?;

        // Internal interconnections
        top.ConnectSubModules(adder1_id, 0, adder2_id, 0)?;  // adder1_out -> adder2_in
        top.BindOutPort(0, adder2_id, 0)?;  // Final output to test_io

        let sealed_top = top.Seal()?;
        Ok(Self { top_module: sealed_top })
    }

    pub fn Run(&mut self, layout: &Layout) -> Result<Vec<Reg>, SimError> {
        let mut engine = SimEngine::Create(&layout)?;
        
        // Drive simulation
        let mut results = Vec::new();
        for cycle in 0..100 {
            engine.Drive()?;
            results.push(engine.GetPortValue(/* test_output */));
        }
        Ok(results)
    }
}
```

### 3.2 CoroKernel for Test I/O

```rust
pub fn create_test_io_kernel() -> CoroKernelFactory {
    Arc::new(|| {
        Box::new(TestIOCoroKernel {
            cycle: 0,
            inputs: vec![],
            outputs: vec![],
        })
    })
}

pub struct TestIOCoroKernel {
    cycle: U32,
    inputs: Vec<Reg>,
    outputs: Vec<Reg>,
}

impl CoroKernel for TestIOCoroKernel {
    fn Step(&mut self, input_regs: &[Reg], output_regs: &mut [Reg]) {
        // Test cycle control
        match self.cycle {
            0..=10 => {
                // Drive stimulus
                output_regs[0] = Reg::Known(self.cycle as u64);
            }
            11..=20 => {
                // Observe response
                self.inputs.push(input_regs[0]);
            }
            _ => {}
        }
        self.cycle += 1;
    }
}
```

---

## 4. Migration Roadmap

### Phase 1: Core Hierarchy (Weeks 1-2)
- [ ] Define `PortSpec`, `PortAccess`, `Visibility` types
- [ ] Extend `Module` struct with inport/outport vectors
- [ ] Implement `Module::New()`, `Module::AddSubModule()`
- [ ] Implement visibility validation in connections
- [ ] Add `Module::Seal()` and `IsSealed` checks
- [ ] Update `layout.rs` to respect module boundaries

### Phase 2: Flatten & Compile (Weeks 2-3)
- [ ] Implement hierarchy-to-netlist flattening
- [ ] Extend `Layout::Build()` to traverse hierarchy
- [ ] Map internal signals to trigger IDs
- [ ] Validate no cross-hierarchy connections bypass ports
- [ ] Generate `EdgeBroadcast` from flattened netlist

### Phase 3: Test Framework (Weeks 3-4)
- [ ] Create `RubeTest_XXXX` module templates
- [ ] Implement test I/O CoroKernel pattern
- [ ] Migrate existing `_tests.rs` to use new hierarchy
- [ ] Verify test results match current behavior

### Phase 4: Backward Compatibility (Weeks 4-5)
- [ ] Provide `LegacyLayout` wrapper for flat composition
- [ ] Ensure old code continues to work
- [ ] Add deprecation warnings for direct port access
- [ ] Document migration path for external users

### Phase 5: Optimization & Polish (Weeks 5+)
- [ ] Profile performance (should be identical)
- [ ] Add module introspection API
- [ ] Create helper macros for common patterns
- [ ] Update documentation and examples

---

## 5. Implementation Details

### 5.1 Hierarchy Flattening Algorithm

```rust
impl Layout {
    fn Flatten(module: &Module, parent_scope: Option<ModuleId>) -> Result<(), LayoutError> {
        // 1. Create ports in current scope
        let inports = module._InPorts.clone();
        let outports = module._OutPorts.clone();

        // 2. Recursively process submodules
        for submodule_id in &module._SubModules {
            let submodule = self._Modules.Get(*submodule_id);
            self.Flatten(submodule, Some(module._Id))?;
        }

        // 3. Flatten internal connections
        for connection in &module._Connections {
            // Map (src_module, src_port) -> (dst_module, dst_port)
            let src_trigger = self.ResolvePort(&connection.src)?;
            let dst_trigger = self.ResolvePort(&connection.dst)?;
            self._Netlist.Connect(src_trigger, dst_trigger)?;
        }

        // 4. Expose boundary connections (inports/outports)
        for (i, inport_spec) in inports.iter().enumerate() {
            let inport = PortId::new(module._Id, PortAccess::InPort(i));
            // Map to internal driver via PortAccess::InPort
        }

        Ok(())
    }

    fn ResolvePort(&self, port_ref: &PortRef) -> Result<TriggerId, LayoutError> {
        match port_ref {
            PortRef::Local(module_id, port_access) => {
                let module = self._Modules.Get(*module_id);
                match port_access {
                    PortAccess::InPort(idx) => {
                        // Look up internal driver of this inport
                        module._InPortDrivers[*idx]
                    }
                    PortAccess::OutPort(idx) => {
                        // Look up internal source of this outport
                        module._OutPortSources[*idx]
                    }
                }
            }
            _ => Err(LayoutError::InvalidPort),
        }
    }
}
```

### 5.2 Visibility Enforcement

```rust
impl Module {
    /// External API - only for inports/outports
    pub fn GetInPort(&self, index: usize) -> Result<PortId, HierarchyError> {
        if index >= self._InPorts.len() {
            return Err(HierarchyError::InvalidPortIndex);
        }
        Ok(PortId::new(self._Id, PortAccess::InPort(index)))
    }

    pub fn GetOutPort(&self, index: usize) -> Result<PortId, HierarchyError> {
        if index >= self._OutPorts.len() {
            return Err(HierarchyError::InvalidPortIndex);
        }
        Ok(PortId::new(self._Id, PortAccess::OutPort(index)))
    }

    /// Internal-only API (panics if called externally - enforced by Rust visibility)
    fn GetSubModulePort(&self, submodule_id: ModuleId, port_access: PortAccess) 
        -> Result<PortId, HierarchyError> 
    {
        // Only callable within module.rs
        Ok(PortId::new(submodule_id, port_access))
    }
}
```

### 5.3 SealedModule Guarantee

```rust
/// After Seal(), this type proves all internal connections are complete
pub struct SealedModule(Module);

impl SealedModule {
    /// Can be used in layout or as submodule
    pub fn AsModule(&self) -> &Module {
        &self.0
    }

    /// Extract for flattening
    pub fn into_module(self) -> Module {
        self.0
    }
}

/// Compiler ensures:
/// - Module can't be used in Layout until Sealed
/// - Sealed module can't be modified
/// - Only Sealed modules can be submodules
```

---

## 6. Example: Migrating an Adder Pipeline

### Before (Flat)
```rust
let mut layout = Layout::New();
let adder1 = layout.AddModule("Adder1", 32, 32, 32, KernelKind::Custom("BusAdder32"));
let adder2 = layout.AddModule("Adder2", 32, 32, 32, KernelKind::Custom("BusAdder32"));
let latch = layout.AddModule("Latch", 32, 32, 0, KernelKind::Custom("DLatch"));

// Problem: Vulnerable to accidental external connections to latch output
layout.Connect(adder1.OutPort(0), adder2.InPort(0))?;
layout.Connect(adder2.OutPort(0), latch.InPort(0))?;

let frozen = layout.Freeze()?;
let engine = SimEngine::Create(&frozen)?;
```

### After (Hierarchical)
```rust
pub struct AdderPipeline;

impl AdderPipeline {
    pub fn New() -> Result<SealedModule, HierarchyError> {
        let mut module = Module::New(
            "AdderPipeline",
            vec![PortSpec { name: "in".into(), width: 32, direction: In }],
            vec![PortSpec { name: "out".into(), width: 32, direction: Out }],
        );

        // Internal submodules
        let adder1 = module.AddSubModule("Adder1", KernelKind::Custom("BusAdder32"))?;
        let adder2 = module.AddSubModule("Adder2", KernelKind::Custom("BusAdder32"))?;
        let latch = module.AddSubModule("Latch", KernelKind::Custom("DLatch"))?;

        // Internal connections (hidden from parent)
        module.ConnectSubModules(adder1, 0, adder2, 0)?;  // adder1 -> adder2
        module.ConnectSubModules(adder2, 0, latch, 0)?;   // adder2 -> latch

        // Boundary connections
        module.BindInPort(0, adder1, 0)?;   // this.in -> adder1.in
        module.BindOutPort(0, latch, 0)?;   // latch.out -> this.out

        module.Seal()
    }
}

// Top-level composition
let mut layout = HierarchyBuilder::New("TestTop");
let pipeline = AdderPipeline::New()?;
layout = layout.AddModule("Pipeline", pipeline, |_| Ok(()))?;
let frozen = layout.Build()?;
let engine = SimEngine::Create(&frozen)?;
```

**Benefits**:
- ✅ Latch internals are private - can't be accidentally modified
- ✅ Interface is explicit (1 input, 1 output)
- ✅ Implementation details can change without affecting consumers
- ✅ Reusable in different contexts

---

## 7. Validation & Testing Strategy

### 7.1 Unit Tests per Phase

```rust
#[cfg(test)]
mod hierarchy_tests {
    #[test]
    fn test_module_creation() {
        let module = Module::New("Test", vec![], vec![]);
        assert_eq!(module._InPorts.len(), 0);
        assert!(!module._IsSealed);
    }

    #[test]
    fn test_submodule_visibility() {
        let mut parent = Module::New("Parent", vec![], vec![]);
        let sub = parent.AddSubModule("Sub", KernelKind::None).unwrap();
        
        // Can access submodule internally
        assert!(parent._SubModules.Contains(sub));
    }

    #[test]
    fn test_sealed_immutability() {
        let module = Module::New("Test", vec![], vec![]).Seal().unwrap();
        // Compiler prevents: module.AddSubModule(...);
    }

    #[test]
    fn test_hierarchy_flattening() {
        let pipeline = AdderPipeline::New().unwrap();
        let layout = HierarchyBuilder::New("Top")
            .AddModule("Pipeline", pipeline.clone(), |_| Ok(()))?
            .Build()
            .unwrap();
        
        // Verify flattened netlist contains all modules
        assert!(layout._Modules.Size() > 1);
    }
}
```

### 7.2 Integration Tests

- Migrate one complete test module (e.g., `RubeTest_Adder`)
- Verify output matches current behavior
- Measure performance (should be identical)
- Repeat for each existing test

---

## 8. Summary & Decision Points

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Inport/Outport Model** | Yes, explicit vectors | Enables visibility control |
| **Kernel on Containers** | Optional (can be None) | Allows both atomic and composite modules |
| **Flattening Strategy** | Recursive, at Freeze time | No runtime penalty, compile-time verification |
| **Backward Compatibility** | LegacyLayout wrapper | Gradual migration path |
| **Test Pattern** | RubeTest_XXXX per test | Clear test isolation |

---

## 9. Next Steps

1. **Review & Feedback**: Discuss this design with team
2. **Prototype**: Implement Phase 1 (core hierarchy types) as a feature branch
3. **Validate**: Ensure performance is identical to current implementation
4. **Migrate**: Progressively convert existing tests to new hierarchy
5. **Document**: Create coding guidelines for module authoring

