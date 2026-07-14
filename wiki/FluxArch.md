# Flux Architecture

The Flux framework is a unified, high-performance data exchange standard in Kosh. It defines how data streams (like bytes from a file or network) flow into and out of arbitrary memory layouts and structures without imposing intermediate Abstract Syntax Trees (ASTs) or heavy memory allocations.

Flux operates on a strict zero-allocation philosophy, decoupling the structure of memory from the serialization format (e.g., JSON).

## Core Design Principles

1. **Zero-Heap Allocation**: The Flux injection (`FluxIn`) and extraction (`FluxOut`) pipelines utilize statically dispatched traits and temporary borrows, meaning that parsing and serialization generate zero heap allocations.
2. **Direct Injection/Extraction**: Data is read out of and written directly into the destination struct's memory layout (via closures and typed Sinks), bypassing generic intermediate representations like `serde_json::Value`.
3. **Decoupled Serialization Logic**: The specific structure (like a `TermTree` or `Arr`) does not need to know about JSON or XML. It simply fulfills the Flux trait contracts (`IFluxOutSource` / `IFluxInSource`), and generic serializers/parsers map those to formats.

---

## The Extraction Pipeline: FluxOut

The `FluxOut` pipeline ([fluxout.rs](../src/flux/fluxout.rs)) standardizes how data is read out of any structure for serialization.

### 1. `FieldOut` Enum
A comprehensive enum defining the primitive and structured types that Kosh supports exporting.
* Primitives: `String`, `Bool`, `U64`, `F64`, `Null`, etc.
* Structures:
  * `Obj(Box<dyn FnMut(&mut FieldOut<'_>) -> bool>)`: Represents a key-value object. The closure yields key-value pairs one by one to the serializer.
  * `Arr(Box<dyn FnMut(&mut FieldOut<'_>) -> bool>)`: Represents an ordered array. The closure yields elements one by one.
  * `FluxSource(&dyn IFluxOutSource)`: An indirection that defers evaluation to another source.

### 2. `IFluxOutSource` Trait
Any data structure that wants to be serializable implements this trait.
```rust
pub trait IFluxOutSource {
    fn ToFieldOut<'a>(&'a self, field: &mut FieldOut<'a>);
}
```
The implementation specifies how the structure represents itself in the generic `FieldOut` vocabulary.

### 3. `IFluxOutSink` Trait
Implemented by serializers (e.g., `JsonOutStream`). The sink accepts a `FieldOut` and converts it into the final output format (like appending to a string buffer or writing to a network socket).

---

## The Injection Pipeline: FluxIn

The `FluxIn` pipeline ([fluxin.rs](../src/flux/fluxin.rs)) governs parsing and the flow of structured data directly into memory layouts.

### 1. `FieldIn` Enum
A comprehensive enum defining the destination sinks into which parsers can deposit data.
* Primitives: `String(&mut String)`, `Bool(&mut bool)`, `U64(&mut u64)`, `F64(&mut f64)`, `Null`.
* Structures:
  * `Obj(Box<dyn FnMut(&str, &mut FieldIn<'_>) -> bool>)`: Represents a mapping closure. When a parser encounters a key, it invokes this closure with the key name, and the closure provides a `FieldIn` sink pointing to the correct field in the target memory layout.
  * `Arr(Box<dyn FnMut(&mut FieldIn<'_>) -> bool>)`: Represents an array closure. The parser requests a sink for the next array element.
  * `FluxSink(&mut dyn IFluxInSink)`: An indirection for custom logic.
  * `FluxSource(&dyn IFluxInSource)`: A deferred instruction for resolution.

### 2. `IFluxInSource` & `IFluxInSink`
* **`IFluxInSource`**: Defines a target that knows how to provide a `FieldIn` mapping for itself.
* **`IFluxInSink`**: Defines a target that knows how to dynamically assimilate data from a provided `FieldIn`.

### 3. Integration with the Parser Framework
The `Parser` and its grammars (`IGrammar`) tightly integrate with `FluxIn`. 
* When `IGrammar::Match` executes, it accepts a `FieldIn` sink.
* As the parser consumes tokens (e.g., matching a JSON Object using `JsonShard`), it delegates to the structure's `FieldIn::Obj` closure to retrieve pointers to actual struct fields.
* Primitives (like `RealShard` and `MatchString`) deposit the final parsed value directly into the yielded pointers.

**Result**: A JSON string is parsed directly into a Rust struct in a single pass with zero intermediate memory allocations.

---

## Stream Management

Flux introduces the `IStream` trait ([stream.rs](../src/flux/stream.rs)) to abstract away the origin of data being parsed.
* Provides uniform methods like `BytesAt` and standard indexing for reading memory safely.
* Implementations include `FixedStream` (for in-memory string slices) and `FileStream` (for robust disk I/O streaming using memory-mapped concepts).
