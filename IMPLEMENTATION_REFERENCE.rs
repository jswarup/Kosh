// ============================================================================
// HIERARCHICAL MODULE FRAMEWORK - QUICK REFERENCE IMPLEMENTATION GUIDE
// ============================================================================
// This file contains the core type definitions and API sketches for
// implementing the Hierarchical Module Framework in Kosh/Rube.

// ============================================================================
// 1. CORE TYPES (Add to module.rs)
// ============================================================================

use crate::silo::{Buff, Stash, U32, USeg};

/// Port specification: declares name, width, and direction
#[derive(Clone, Debug)]
pub struct PortSpec {
    pub name: String,
    pub width: U32,
    pub direction: PortDir,
}

impl PortSpec {
    pub fn Input(name: &str, width: U32) -> Self {
        Self {
            name: name.to_string(),
            width,
            direction: PortDir::In,
        }
    }

    pub fn Output(name: &str, width: U32) -> Self {
        Self {
            name: name.to_string(),
            width,
            direction: PortDir::Out,
        }
    }
}

/// How a port is accessed within its scope
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortAccess {
    InPort(usize),   // Index into Module._InPorts
    OutPort(usize),  // Index into Module._OutPorts
}

/// Visibility context for module access
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Internal,  // Can access full internal structure
    External,  // Can only access inports/outports
}

/// Connection reference within module hierarchy
#[derive(Clone, Debug)]
pub struct PortRef {
    pub module_id: ModuleId,
    pub access: PortAccess,
}

impl PortRef {
    pub fn InPort(module_id: ModuleId, index: usize) -> Self {
        Self {
            module_id,
            access: PortAccess::InPort(index),
        }
    }

    pub fn OutPort(module_id: ModuleId, index: usize) -> Self {
        Self {
            module_id,
            access: PortAccess::OutPort(index),
        }
    }
}

/// Encapsulated connection (internal to module)
#[derive(Clone, Debug)]
pub struct InternalConnection {
    pub src: PortRef,
    pub dst: PortRef,
}

/// Errors specific to hierarchy operations
#[derive(Clone, Debug)]
pub enum HierarchyError {
    ModuleAlreadySealed,
    InvalidPortIndex(usize),
    InvalidPortWidth,
    PortDirectionMismatch,
    SubModuleNotFound(ModuleId),
    NotSealed,
    InvalidPortType,
    PortNotInModule(PortRef),
}

// ============================================================================
// 2. MODULE STRUCT (Modified in module.rs)
// ============================================================================

/// Represents a hierarchical module in the simulation
#[derive(Clone, Debug)]
pub struct Module {
    // Identity
    pub _Id: ModuleId,
    pub _Parent: Option<ModuleId>,
    pub _Name: String,

    // External Interface (visible to parent)
    pub _InPorts: Vec<PortSpec>,   // NEW: Input port declarations
    pub _OutPorts: Vec<PortSpec>,  // NEW: Output port declarations

    // Internal Structure (hidden from parent)
    pub _SubModules: Stash<ModuleId>,           // Contained modules
    pub _SubModuleKernels: Stash<KernelKind>,   // Kernels for each submodule
    pub _Connections: Stash<InternalConnection>, // Internal nets

    // Boundary Mappings (how external ports connect internally)
    pub _InPortDrivers: Buff<PortRef>,   // Where each inport gets its signal
    pub _OutPortSources: Buff<PortRef>,  // Where each outport sources its signal

    // Kernel (optional, None for containers)
    pub _Kernel: KernelKind,

    // State (prevents modification after seal)
    pub _IsSealed: bool,
    pub _IsConstruction: bool,  // NEW: true during constructor, false after Seal
}

/// A module that has been sealed and is safe to use
pub struct SealedModule(pub Module);

impl SealedModule {
    pub fn as_module(&self) -> &Module {
        &self.0
    }

    pub fn into_module(self) -> Module {
        self.0
    }
}

// ============================================================================
// 3. MODULE IMPLEMENTATION (Add to module.rs)
// ============================================================================

impl Module {
    /// Create a new module with declared interface
    pub fn New(
        name: &str,
        inport_specs: Vec<PortSpec>,
        outport_specs: Vec<PortSpec>,
    ) -> Self {
        let id = ModuleId(rand_unique_id());  // Or use registry

        let n_inports = inport_specs.len();
        let n_outports = outport_specs.len();

        Self {
            _Id: id,
            _Parent: None,
            _Name: name.to_string(),
            _InPorts: inport_specs,
            _OutPorts: outport_specs,
            _SubModules: Stash::WithCapacity(10),
            _SubModuleKernels: Stash::WithCapacity(10),
            _Connections: Stash::WithCapacity(20),
            _InPortDrivers: Buff::WithCapacity(n_inports),
            _OutPortSources: Buff::WithCapacity(n_outports),
            _Kernel: KernelKind::None,
            _IsSealed: false,
            _IsConstruction: true,
        }
    }

    /// Add a submodule to this module (constructor only)
    pub fn AddSubModule(
        &mut self,
        name: &str,
        kernel: KernelKind,
    ) -> Result<ModuleId, HierarchyError> {
        if self._IsSealed {
            return Err(HierarchyError::ModuleAlreadySealed);
        }

        let sub_id = ModuleId(rand_unique_id());
        self._SubModules.Push(sub_id);
        self._SubModuleKernels.Push(kernel);

        Ok(sub_id)
    }

    /// Connect two submodules internally
    pub fn ConnectSubModules(
        &mut self,
        src_id: ModuleId,
        src_port_idx: usize,
        dst_id: ModuleId,
        dst_port_idx: usize,
    ) -> Result<(), HierarchyError> {
        if self._IsSealed {
            return Err(HierarchyError::ModuleAlreadySealed);
        }

        // Validate ports exist in submodules
        let src = PortRef::OutPort(src_id, src_port_idx);
        let dst = PortRef::InPort(dst_id, dst_port_idx);

        self._Connections.Push(InternalConnection {
            src,
            dst,
        });

        Ok(())
    }

    /// Map this module's inport to a submodule's outport
    pub fn BindInPort(
        &mut self,
        this_inport_idx: usize,
        submodule_id: ModuleId,
        submodule_outport_idx: usize,
    ) -> Result<(), HierarchyError> {
        if self._IsSealed {
            return Err(HierarchyError::ModuleAlreadySealed);
        }

        if this_inport_idx >= self._InPorts.len() {
            return Err(HierarchyError::InvalidPortIndex(this_inport_idx));
        }

        let source = PortRef::OutPort(submodule_id, submodule_outport_idx);
        self._InPortDrivers[this_inport_idx] = source;

        Ok(())
    }

    /// Map this module's outport to a submodule's inport
    pub fn BindOutPort(
        &mut self,
        this_outport_idx: usize,
        submodule_id: ModuleId,
        submodule_inport_idx: usize,
    ) -> Result<(), HierarchyError> {
        if self._IsSealed {
            return Err(HierarchyError::ModuleAlreadySealed);
        }

        if this_outport_idx >= self._OutPorts.len() {
            return Err(HierarchyError::InvalidPortIndex(this_outport_idx));
        }

        let source = PortRef::InPort(submodule_id, submodule_inport_idx);
        self._OutPortSources[this_outport_idx] = source;

        Ok(())
    }

    /// Seal the module - make it immutable and safe to use
    pub fn Seal(mut self) -> Result<SealedModule, HierarchyError> {
        // Validate all connections are complete
        if self._IsConstruction {
            // Perform validation here
            // - Check all inports have drivers
            // - Check all outports have sources
            // - Check no dangling connections
        }

        self._IsSealed = true;
        self._IsConstruction = false;

        Ok(SealedModule(self))
    }

    /// Get inport by index (external API)
    pub fn GetInPort(&self, index: usize) -> Result<PortRef, HierarchyError> {
        if index >= self._InPorts.len() {
            Err(HierarchyError::InvalidPortIndex(index))
        } else {
            Ok(PortRef::InPort(self._Id, index))
        }
    }

    /// Get outport by index (external API)
    pub fn GetOutPort(&self, index: usize) -> Result<PortRef, HierarchyError> {
        if index >= self._OutPorts.len() {
            Err(HierarchyError::InvalidPortIndex(index))
        } else {
            Ok(PortRef::OutPort(self._Id, index))
        }
    }

    /// Private: Access submodule port (internal only)
    #[inline]
    fn GetSubModuleInPort(&self, submodule_id: ModuleId, index: usize)
        -> Result<PortRef, HierarchyError>
    {
        Ok(PortRef::InPort(submodule_id, index))
    }

    #[inline]
    fn GetSubModuleOutPort(&self, submodule_id: ModuleId, index: usize)
        -> Result<PortRef, HierarchyError>
    {
        Ok(PortRef::OutPort(submodule_id, index))
    }
}

// ============================================================================
// 4. HIERARCHY BUILDER (New type in layout.rs)
// ============================================================================

pub struct HierarchyBuilder {
    root: SealedModule,
    // Could store intermediate state here
}

impl HierarchyBuilder {
    pub fn New(top_name: &str) -> Self {
        let root_module = Module::New(top_name, vec![], vec![]);
        Self {
            root: SealedModule(root_module),
        }
    }

    pub fn AddModule<F>(
        mut self,
        name: &str,
        kernel: KernelKind,
        setup: F,
    ) -> Result<Self, HierarchyError>
    where
        F: FnOnce(&mut Module) -> Result<(), HierarchyError>,
    {
        let mut module = Module::New(name, vec![], vec![]);
        setup(&mut module)?;
        let sealed = module.Seal()?;

        // Add to root's submodules
        let mut root_mut = self.root.0;
        root_mut.AddSubModule(name, sealed.0._Kernel)?;
        self.root = SealedModule(root_mut);

        Ok(self)
    }

    pub fn Build(self) -> Result<Layout, HierarchyError> {
        let layout = Layout::New();
        // Flatten hierarchy into layout
        // Walk root and all submodules
        // Generate triggers and compile warps
        Ok(layout)
    }
}

// ============================================================================
// 5. FLATTENING ALGORITHM SKETCH (Add to layout.rs)
// ============================================================================

impl Layout {
    /// Recursively flatten module hierarchy into flat netlist
    pub fn FlattenModule(
        &mut self,
        module: &Module,
        parent_scope: Option<ModuleId>,
    ) -> Result<(), HierarchyError> {
        // 1. Register this module's ports in the flat layout
        for (i, inport_spec) in module._InPorts.iter().enumerate() {
            let port_id = self.AddPort(
                module._Id,
                &inport_spec.name,
                inport_spec.width,
                inport_spec.direction,
            );
            // Record mapping: (module._Id, InPort(i)) -> port_id
        }

        for (i, outport_spec) in module._OutPorts.iter().enumerate() {
            let port_id = self.AddPort(
                module._Id,
                &outport_spec.name,
                outport_spec.width,
                outport_spec.direction,
            );
            // Record mapping: (module._Id, OutPort(i)) -> port_id
        }

        // 2. Recursively flatten submodules
        for submodule_id in &module._SubModules {
            // Note: You need to track which submodule is which
            // This requires a registry or lookup structure
            // let submodule = self.GetModule(submodule_id);
            // self.FlattenModule(submodule, Some(module._Id))?;
        }

        // 3. Flatten internal connections
        for conn in &module._Connections {
            let src_port = self.ResolvePortRef(&conn.src)?;
            let dst_port = self.ResolvePortRef(&conn.dst)?;
            self._Netlist.Connect(src_port, dst_port)?;
        }

        // 4. Flatten boundary connections (inports/outports)
        for (i, inport_driver) in module._InPortDrivers.Arr().Traverse(|r| r.clone()).collect::<Vec<_>>().iter().enumerate() {
            let this_inport = self.ResolvePortRef(&PortRef::InPort(module._Id, i))?;
            let driver = self.ResolvePortRef(inport_driver)?;
            self._Netlist.Connect(driver, this_inport)?;
        }

        Ok(())
    }

    /// Resolve a PortRef to an actual PortId in the flattened layout
    fn ResolvePortRef(&self, port_ref: &PortRef) -> Result<PortId, HierarchyError> {
        // Use a lookup table built during flattening
        // portref_to_portid_map.get(port_ref)?
        Ok(PortId(0))  // Placeholder
    }

    fn AddPort(
        &mut self,
        owner: ModuleId,
        name: &str,
        width: U32,
        direction: PortDir,
    ) -> PortId {
        // Add to _Ports stash
        // Return allocated PortId
        PortId(0)  // Placeholder
    }
}

// ============================================================================
// 6. TEST FRAMEWORK EXAMPLE (New file: tests/rube_test_adder.rs)
// ============================================================================

pub struct RubeTest_Adder;

impl RubeTest_Adder {
    pub fn BuildHierarchy() -> Result<SealedModule, HierarchyError> {
        // Test input/output kernel
        let io_kernel = KernelKind::Coro(create_adder_test_io_kernel());

        let mut top = Module::New("RubeTest_Adder", vec![], vec![]);

        // Submodules
        let adder1 = top.AddSubModule("Adder1",
            KernelKind::Custom("BusAdder32"))?;
        let adder2 = top.AddSubModule("Adder2",
            KernelKind::Custom("BusAdder32"))?;
        let test_io = top.AddSubModule("TestIO", io_kernel)?;

        // Internal connections
        top.ConnectSubModules(adder1, 0, adder2, 0)?;  // chain adders
        top.ConnectSubModules(adder2, 0, test_io, 0)?; // adder2 -> test_io

        top.Seal()
    }

    pub fn Run() -> Result<Vec<Reg>, SimError> {
        let sealed = Self::BuildHierarchy()?;
        let layout = HierarchyBuilder::New("AdderTest")
            .AddModule("DUT", sealed.0._Kernel, |_| Ok(()))?
            .Build()?;

        let mut engine = SimEngine::Create(&layout)?;

        let mut results = Vec::new();
        for _cycle in 0..100 {
            engine.Drive()?;
            // Read test results from test_io CoroKernel
            // results.push(...);
        }

        Ok(results)
    }
}

fn create_adder_test_io_kernel() -> CoroKernelFactory {
    Arc::new(|| Box::new(AdderTestIOKernel::new()))
}

pub struct AdderTestIOKernel {
    cycle: U32,
    test_vectors: Vec<(Reg, Reg)>,  // (input, expected_output)
}

impl AdderTestIOKernel {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            test_vectors: vec![
                (Reg::Known(5), Reg::Known(10)),
                (Reg::Known(100), Reg::Known(200)),
                // ... more vectors
            ],
        }
    }
}

impl CoroKernel for AdderTestIOKernel {
    fn Step(&mut self, input_regs: &[Reg], output_regs: &mut [Reg]) {
        if self.cycle < self.test_vectors.len() as U32 {
            let (test_in, expected) = self.test_vectors[self.cycle as usize];
            output_regs[0] = test_in;
            // Verify input_regs[0] == expected
        }
        self.cycle += 1;
    }
}

// ============================================================================
// USAGE PATTERN
// ============================================================================

/*
Example of complete usage:

#[test]
fn test_adder_pipeline() {
    let result = RubeTest_Adder::Run().expect("Test failed");
    assert!(!result.is_empty());
}

Example of creating custom module:

struct MyCustomModule;
impl MyCustomModule {
    pub fn New() -> Result<SealedModule, HierarchyError> {
        let mut m = Module::New("MyCustom",
            vec![PortSpec::Input("in", 32)],
            vec![PortSpec::Output("out", 32)]
        );

        let sub1 = m.AddSubModule("Sub1", KernelKind::Custom("..."));
        let sub2 = m.AddSubModule("Sub2", KernelKind::Custom("..."));

        m.ConnectSubModules(sub1?, 0, sub2?, 0)?;
        m.BindInPort(0, sub1?, 0)?;
        m.BindOutPort(0, sub2?, 0)?;

        m.Seal()
    }
}
*/

// ============================================================================
// END OF QUICK REFERENCE GUIDE
// ============================================================================
