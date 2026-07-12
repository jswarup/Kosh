# Shard Architecture

The Shard framework is a robust, zero-heap-allocation Abstract Syntax Tree (AST) designed for high-performance grammar parsing and backtracking recursive descent execution. The architecture focuses on CPU cache locality, eliminating dynamic memory allocation during tree construction, and maintaining a strict, generic interface via the `IGrammar` trait.

## Core Design Principles

1. **Zero-Heap Allocation**: The Shard AST nodes prioritize stack-allocated references rather than owned heap allocations (like `Box`). This design significantly reduces overhead during parser initialization and ensures that tree traversal is highly cache-efficient.
2. **Rust Temporary Lifetime Extension**: The node construction logic relies heavily on inline struct instantiation directly within the `ShardTree!` macro. By expanding structs like `BinShard` and `StrShard` behind borrow references (`&`) inline, Rust's temporary lifetime extension guarantees that these stack-allocated trees live as long as the enclosing block.
3. **Trait-driven Modularity**: Nodes implement `IGrammar` (to define the recursive parser matching semantics) and `IXFluxSource` (to enable serialization).

## Node Types

The Shard AST consists of consolidated, lightweight structs, separated broadly into leaves, binary nodes, and unary modifiers.

### Leaves (`leaves.rs`)
Terminal nodes in the AST represent basic consumable elements from the input stream.
* **`StrShard`**: Wraps a static or temporary string slice (`&str`). Used to match literal token strings. The `ShardTree!` macro matches raw string literals (`$val:literal`) directly and evaluates them into `StrShard`s.
* **`CharsetShard`**: Wraps a `Charset` bitset to match any character belonging to a custom set (e.g., via `Boxet!`).

### Binary Nodes (`binshard.rs`)
Nodes that manage two child paths.
* **`BinShard`**: A parameterized binary node that encapsulates the `_Left` and `_Right` child branches. It uses a `BinShardOp` to determine its behavior:
  * `BinShardOp::Choice`: Corresponds to the `|` (Bor) operator, branching to find the first matching sub-grammar.
  * `BinShardOp::Sequence`: Corresponds to the `<` (Less) operator, consecutively matching the left and right branches.

### Unary Nodes
Nodes that wrap a single child grammar path for modified execution semantics.
* **`RepeatShard`**: Encapsulates a child node and allows for unbounded repetitions (e.g., Kleene star `*` or plus `+`), leveraging `USeg` logic.
* **`ActionShard`**: Encapsulates a child node and attaches a semantic action (worker closure) that executes upon a successful match.

## Grammar Parsing

The `IGrammar` trait is the core matching interface implemented by all shards.
```rust
pub trait IGrammar {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32>;
}
```
* **Backtracking**: The `Parser` maintains the state of the stream via markers (`U32`). If a grammar path matches successfully, it returns a new stream marker wrapped in `Some`. If it fails, it returns `None`, allowing choice nodes like `BinShardOp::Choice` to safely backtrack to the original marker and attempt alternative branches.
* **Stream Coupling**: The parser reads tokens directly utilizing the `IXFluxSource` data stream mechanism, seamlessly integrating with Kosh's broader ecosystem.

## ShardTree Macro

The `ShardTree!` macro is the DSL interface for defining grammars. It is implemented as a clean, highly optimized, and modular recursive macro using internal helper sub-rules (`@resolve`, `@bin`, `@action`, `@repeat`):
* Utilizes a generic token tree pattern (`$l:tt`) to resolve leaves (identifiers, literals, charsets, and parenthesized groups) via the `@resolve` helper, eliminating redundant rules.
* Intercepts choices (`|`) and sequences (`<`) to generate `BinShard` structures via the `@bin` helper.
* Handles repetitions (`*` and `+`) and attached closures (`[ |p| body ]`) via the `@repeat` and `@action` helpers.
