# EDA Compatibility Implementation Guide - Phase 1

## Quick Start: Module Interface System

This guide shows how to implement Phase 1 (Module Interface Standards) step-by-step.

---

## Step 1: Define Interface Types (New file: `interface.rs`)

```rust
//-- rube/interface.rs ---

use crate::rube::port::PortDir;

/// Port data type specification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    Logic(usize),      // Bit width (e.g., Logic(32) for 32-bit)
    Bool,
    Integer,
    Real,
}

impl DataType {
    pub fn width(&self) -> usize {
        match self {
            DataType::Logic(w) => *w,
            DataType::Bool => 1,
            DataType::Integer => 32,
            DataType::Real => 64,
        }
    }
}

/// Bus type structure
#[derive(Clone, Debug)]
pub enum BusType {
    Single,
    Bus(usize),
    Struct {
        fields: &'static [PortField],
    },
    Array {
        element_width: usize,
        length: usize,
    },
}

/// Field within a structured port
#[derive(Clone, Debug)]
pub struct PortField {
    pub name: &'static str,
    pub bit_high: usize,
    pub bit_low: usize,
    pub data_type: DataType,
}

/// Port attributes (clock, reset, valid, etc.)
#[derive(Clone, Debug, Default)]
pub struct PortAttributes {
    pub is_clock: bool,
    pub is_reset: bool,
    pub is_valid: bool,
    pub is_ready: bool,
    pub clock_polarity: Polarity,
    pub reset_polarity: Polarity,
    pub tags: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polarity {
    Positive,   // Rising edge / active high
    Negative,   // Falling edge / active low
}

impl Default for Polarity {
    fn default() -> Self {
        Polarity::Positive
    }
}

/// Complete port specification
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

    pub fn output(name: &'static str, width: usize, doc: Option<&'static str>) -> Self {
        Self {
            name,
            width,
            data_type: DataType::Logic(width),
            direction: PortDir::Out,
            bus_type: BusType::Bus(width),
            attributes: PortAttributes::default(),
            documentation: doc,
        }
    }

    pub fn clock(name: &'static str) -> Self {
        let mut attrs = PortAttributes::default();
        attrs.is_clock = true;
        attrs.clock_polarity = Polarity::Positive;

        Self {
            name,
            width: 1,
            data_type: DataType::Bool,
            direction: PortDir::In,
            bus_type: BusType::Single,
            attributes: attrs,
            documentation: Some("Clock signal (rising edge)"),
        }
    }

    pub fn reset(name: &'static str, active_high: bool) -> Self {
        let mut attrs = PortAttributes::default();
        attrs.is_reset = true;
        attrs.reset_polarity = if active_high {
            Polarity::Positive
        } else {
            Polarity::Negative
        };

        Self {
            name,
            width: 1,
            data_type: DataType::Bool,
            direction: PortDir::In,
            bus_type: BusType::Single,
            attributes: attrs,
            documentation: if active_high {
                Some("Reset signal (active high)")
            } else {
                Some("Reset signal (active low)")
            },
        }
    }

    pub fn valid_ready(name: &'static str) -> Self {
        let mut attrs = PortAttributes::default();
        attrs.is_valid = true;

        Self {
            name,
            width: 1,
            data_type: DataType::Bool,
            direction: PortDir::Out,
            bus_type: BusType::Single,
            attributes: attrs,
            documentation: Some("Valid signal for handshake"),
        }
    }
}

/// Parameter specification
#[derive(Clone, Debug)]
pub struct ParameterInterface {
    pub name: &'static str,
    pub param_type: ParameterType,
    pub default: Option<&'static str>,
    pub documentation: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterType {
    Integer,
    String,
    Real,
    Boolean,
}

/// Complete module interface definition
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

impl ModuleInterface {
    /// Validate that all required ports are connected
    pub fn validate_ports(&self) -> Result<(), InterfaceError> {
        // Check port name uniqueness
        let mut in_names = std::collections::HashSet::new();
        for port in self.inports {
            if !in_names.insert(port.name) {
                return Err(InterfaceError::DuplicatePortName(port.name));
            }
        }

        let mut out_names = std::collections::HashSet::new();
        for port in self.outports {
            if !out_names.insert(port.name) {
                return Err(InterfaceError::DuplicatePortName(port.name));
            }
        }

        // Check width consistency
        for port in self.inports {
            if port.width != port.data_type.width() {
                return Err(InterfaceError::WidthMismatch(port.name));
            }
        }

        Ok(())
    }

    /// Get inport by name
    pub fn get_inport(&self, name: &str) -> Option<&'static PortInterface> {
        self.inports.iter().find(|p| p.name == name)
    }

    /// Get outport by name
    pub fn get_outport(&self, name: &str) -> Option<&'static PortInterface> {
        self.outports.iter().find(|p| p.name == name)
    }

    /// Export to SystemVerilog module definition
    pub fn to_systemverilog(&self) -> String {
        let mut sv = format!("module {} (\n", self.name);

        // Add input ports
        for port in self.inports {
            match port.bus_type {
                BusType::Bus(width) if width > 1 => {
                    sv.push_str(&format!(
                        "    input [{:1}:0] {},\n",
                        width - 1,
                        port.name
                    ));
                }
                _ => {
                    sv.push_str(&format!("    input {},\n", port.name));
                }
            }
        }

        // Add output ports
        for (i, port) in self.outports.iter().enumerate() {
            let suffix = if i == self.outports.len() - 1 {
                "\n"
            } else {
                ",\n"
            };
            match port.bus_type {
                BusType::Bus(width) if width > 1 => {
                    sv.push_str(&format!(
                        "    output [{:1}:0] {}{}",
                        width - 1,
                        port.name,
                        suffix
                    ));
                }
                _ => {
                    sv.push_str(&format!("    output {}{}", port.name, suffix));
                }
            }
        }

        sv.push_str(");\n");
        sv.push_str(&format!("    // {}\n", self.description));
        sv.push_str("endmodule\n");

        sv
    }
}

/// Interface errors
#[derive(Clone, Debug)]
pub enum InterfaceError {
    DuplicatePortName(&'static str),
    WidthMismatch(&'static str),
    InvalidDataType,
}

impl std::fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceError::DuplicatePortName(name) => {
                write!(f, "Duplicate port name: {}", name)
            }
            InterfaceError::WidthMismatch(name) => {
                write!(f, "Width mismatch for port: {}", name)
            }
            InterfaceError::InvalidDataType => {
                write!(f, "Invalid data type")
            }
        }
    }
}

impl std::error::Error for InterfaceError {}
```

---

## Step 2: Module Interface Trait (Add to `module.rs`)

```rust
// Add to module.rs after imports

use crate::rube::interface::ModuleInterface;

/// Trait: Every module must define its interface
pub trait IModuleInterface {
    /// Return the static module interface
    fn interface() -> &'static ModuleInterface;

    /// Get module name
    fn name(&self) -> &str {
        Self::interface().name
    }

    /// Validate this module matches its interface
    fn validate(&self) -> Result<(), crate::rube::interface::InterfaceError> {
        Self::interface().validate_ports()
    }
}
```

---

## Step 3: Implement for Existing Modules (Example)

```rust
// In gates.rs or wherever AdderGate is defined

use crate::rube::interface::{
    DataType, PortInterface, PortAttributes, PortDir, BusType, ModuleInterface, Polarity,
};

// Define the interface as a static const
const BUSADDER32_INTERFACE: ModuleInterface = ModuleInterface {
    name: "bus_adder_32",
    version: "1.0.0",
    description: "32-bit binary adder with carry-out",
    vendor: Some("OrioleDesigns"),
    inports: &[
        PortInterface {
            name: "a",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::In,
            bus_type: BusType::Bus(32),
            attributes: PortAttributes {
                is_clock: false,
                is_reset: false,
                is_valid: false,
                is_ready: false,
                clock_polarity: Polarity::Positive,
                reset_polarity: Polarity::Positive,
                tags: &["operand"],
            },
            documentation: Some("First operand (32-bit)"),
        },
        PortInterface {
            name: "b",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::In,
            bus_type: BusType::Bus(32),
            attributes: PortAttributes {
                tags: &["operand"],
                ..Default::default()
            },
            documentation: Some("Second operand (32-bit)"),
        },
    ],
    outports: &[
        PortInterface {
            name: "sum",
            width: 32,
            data_type: DataType::Logic(32),
            direction: PortDir::Out,
            bus_type: BusType::Bus(32),
            attributes: PortAttributes::default(),
            documentation: Some("Sum output (32-bit)"),
        },
        PortInterface {
            name: "carry",
            width: 1,
            data_type: DataType::Bool,
            direction: PortDir::Out,
            bus_type: BusType::Single,
            attributes: PortAttributes::default(),
            documentation: Some("Carry-out bit"),
        },
    ],
    parameters: &[],
};

// Implement IModuleInterface for your module/kernel
impl IModuleInterface for BusAdder32Kernel {
    fn interface() -> &'static ModuleInterface {
        &BUSADDER32_INTERFACE
    }
}

// Test it
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adder_interface() {
        let interface = BusAdder32Kernel::interface();
        assert_eq!(interface.name, "bus_adder_32");
        assert_eq!(interface.inports.len(), 2);
        assert_eq!(interface.outports.len(), 2);

        // Export to SystemVerilog
        let sv = interface.to_systemverilog();
        println!("{}", sv);
        // Output:
        // module bus_adder_32 (
        //     input [31:0] a,
        //     input [31:0] b,
        //     output [31:0] sum,
        //     output carry
        // );
        //     // 32-bit binary adder with carry-out
        // endmodule
    }
}
```

---

## Step 4: Module Introspection API (New file: `introspect.rs`)

```rust
//-- rube/introspect.rs ---

use crate::rube::{
    interface::ModuleInterface,
    module::ModuleId,
    port::PortId,
    reg::Reg,
};
use std::collections::HashMap;

/// Runtime module introspection
pub trait IModuleIntrospection {
    fn get_interface(&self) -> &'static ModuleInterface;
    fn get_hierarchy_path(&self) -> String;
    fn list_inports(&self) -> Vec<PortIntrospection>;
    fn list_outports(&self) -> Vec<PortIntrospection>;
    fn get_submodules(&self) -> Vec<ModuleIntrospection>;
    fn get_module_id(&self) -> ModuleId;
}

/// Runtime port information
#[derive(Clone, Debug)]
pub struct PortIntrospection {
    pub name: String,
    pub port_id: PortId,
    pub width: usize,
    pub direction: crate::rube::port::PortDir,
    pub current_value: Option<Reg>,  // If we can query engine
}

/// Runtime module information
#[derive(Clone, Debug)]
pub struct ModuleIntrospection {
    pub name: String,
    pub module_id: ModuleId,
    pub interface_name: &'static str,
    pub hierarchy_depth: usize,
    pub port_count: (usize, usize),  // (inports, outports)
}

/// Module statistics during simulation
#[derive(Clone, Debug)]
pub struct ModuleStatistics {
    pub name: String,
    pub cycle_count: usize,
    pub port_changes: HashMap<PortId, usize>,
    pub execution_time_ns: Option<u64>,
}

impl ModuleStatistics {
    pub fn new(name: String) -> Self {
        Self {
            name,
            cycle_count: 0,
            port_changes: HashMap::new(),
            execution_time_ns: None,
        }
    }

    pub fn record_port_change(&mut self, port_id: PortId) {
        *self.port_changes.entry(port_id).or_insert(0) += 1;
    }
}

/// Query engine for module information
pub struct ModuleIntrospector {
    stats: HashMap<ModuleId, ModuleStatistics>,
}

impl ModuleIntrospector {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    pub fn get_statistics(&self, module_id: ModuleId) -> Option<&ModuleStatistics> {
        self.stats.get(&module_id)
    }

    pub fn get_port_change_count(&self, module_id: ModuleId, port_id: PortId) -> usize {
        self.stats
            .get(&module_id)
            .and_then(|s| s.port_changes.get(&port_id))
            .copied()
            .unwrap_or(0)
    }
}
```

---

## Step 5: Update Module.rs (Add mod declarations)

```rust
// In rube/mod.rs, add:

pub mod interface;
pub mod introspect;

pub use interface::{ModuleInterface, PortInterface, DataType, BusType, IModuleInterface};
pub use introspect::{IModuleIntrospection, ModuleIntrospection, PortIntrospection};
```

---

## Step 6: Update All Gate Implementations

Create a macro to reduce boilerplate:

```rust
// In gates.rs

/// Macro to define gate interface
#[macro_export]
macro_rules! define_gate_interface {
    (
        $gate_name:ident,
        $display_name:expr,
        $version:expr,
        $description:expr,
        inports: [$($in_port:expr),*],
        outports: [$($out_port:expr),*]
    ) => {
        const GATE_INTERFACE: $crate::rube::ModuleInterface = $crate::rube::ModuleInterface {
            name: $display_name,
            version: $version,
            description: $description,
            vendor: Some("OrioleDesigns"),
            inports: &[$($in_port),*],
            outports: &[$($out_port),*],
            parameters: &[],
        };

        impl $crate::rube::IModuleInterface for $gate_name {
            fn interface() -> &'static $crate::rube::ModuleInterface {
                &GATE_INTERFACE
            }
        }
    };
}

// Usage:
define_gate_interface!(
    AndGate,
    "and_gate",
    "1.0.0",
    "2-input AND gate",
    inports: [
        PortInterface::input("a", 1, Some("First input")),
        PortInterface::input("b", 1, Some("Second input")),
    ],
    outports: [
        PortInterface::output("y", 1, Some("AND output")),
    ]
);
```

---

## Testing Phase 1

```rust
#[cfg(test)]
mod interface_tests {
    use super::*;
    use crate::rube::{IModuleInterface, gates::*};

    #[test]
    fn test_all_gates_have_interface() {
        // BusAdder32Kernel
        let adder_if = BusAdder32Kernel::interface();
        assert!(!adder_if.name.is_empty());
        assert!(adder_if.validate_ports().is_ok());

        // DLatch
        let dlatch_if = DLatch::interface();
        assert!(!dlatch_if.name.is_empty());
        assert!(dlatch_if.validate_ports().is_ok());
    }

    #[test]
    fn test_interface_export_to_systemverilog() {
        let interface = BusAdder32Kernel::interface();
        let sv_code = interface.to_systemverilog();

        // Should contain module definition
        assert!(sv_code.contains("module bus_adder_32"));
        assert!(sv_code.contains("input [31:0] a"));
        assert!(sv_code.contains("input [31:0] b"));
        assert!(sv_code.contains("output [31:0] sum"));
        assert!(sv_code.contains("output carry"));

        println!("Generated SystemVerilog:\n{}", sv_code);
    }

    #[test]
    fn test_port_query_by_name() {
        let interface = BusAdder32Kernel::interface();

        let a_port = interface.get_inport("a");
        assert!(a_port.is_some());
        assert_eq!(a_port.unwrap().width, 32);

        let sum_port = interface.get_outport("sum");
        assert!(sum_port.is_some());
        assert_eq!(sum_port.unwrap().width, 32);
    }
}
```

---

## Benefits After Phase 1

✅ **Self-documenting modules**: Interface in type system
✅ **Automatic HDL export**: Generate SystemVerilog from interfaces
✅ **Runtime introspection**: Query module structure at runtime
✅ **Type safety**: Compile-time port validation
✅ **Better documentation**: Markdown in port specs

---

## Next Steps: Phase 2

Once Phase 1 is complete and tested, move to:
- Type-safe kernel registry
- Kernel packages and factory pattern
- Parameter system
- DPI/VPI integration

