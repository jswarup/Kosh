# Shard Architecture

The Shard framework is a robust, zero-heap-allocation Abstract Syntax Tree (AST) designed for high-performance grammar parsing and backtracking recursive descent execution. The architecture focuses on CPU cache locality, eliminating dynamic memory allocation during tree construction, and maintaining a strict, generic interface via the `IGrammar` trait.

## Core Design Principles

1. **Zero-Heap Allocation**: The Shard AST nodes prioritize stack-allocated references rather than owned heap allocations (like `Box`). This design significantly reduces overhead during parser initialization and ensures that tree traversal is highly cache-efficient.
2. **Rust Temporary Lifetime Extension**: The node construction logic relies heavily on inline struct instantiation directly within the `ShardTree!` macro. By expanding structs like `BinShard` and `StrShard` behind borrow references (`&`) inline, Rust's temporary lifetime extension guarantees that these stack-allocated trees live as long as the enclosing block.
3. **Trait-driven Modularity**: Nodes implement `IGrammar` (to define the recursive parser matching semantics) and `IFluxOutSource` (to enable serialization).

## Node Types

The Shard AST consists of consolidated, lightweight structs, separated broadly into leaves, binary nodes, and unary modifiers.

### Leaves (`leaves.rs` and `charset.rs`)
Terminal nodes in the AST represent basic consumable elements from the input stream.
* **`StrShard`**: Wraps a static or temporary string slice (`&str`). Used to match literal token strings. The `ShardTree!` macro matches raw string literals (`$val:literal`) directly and evaluates them into `StrShard`s.
* **`Charset`**: A 256-bit set representing character classes, matching any character in the set (e.g., constructed via `Boxet!`).

### Binary Nodes (`binshard.rs`)
Nodes that manage two child paths.
* **`BinShard`**: A type alias of Kosh's generic `BinNode<L, R>`. It encapsulates the `_Left` and `_Right` child branches. It uses the unified `BinOp` enum to determine its behavior:
  - `BinOp::Bor`: Corresponds to the `|` (Bor) operator, branching to find the first matching sub-grammar.
  - `BinOp::Less`: Corresponds to the `<` (Less) operator, consecutively matching the left and right branches.

### Unary Nodes
Nodes that wrap a single child grammar path for modified execution semantics.
* **`RepeatShard`**: A type alias of Kosh's generic `UniNode<C, crate::silo::USeg>`. It encapsulates a child node and allows for unbounded repetitions (e.g., Kleene star `*` or plus `+`), leveraging `USeg` logic.
* **`ActionShard`**: A type alias of Kosh's generic `UniNode<C, ActionOp<W>>`. It encapsulates a child node and attaches a semantic action (worker closure) that executes upon a successful match.

## Grammar Parsing and Detached Processing

The `IGrammar` and `IForge` traits form the core matching and detached processing architecture of the Shard framework.
```rust
pub trait IGrammar: INode {
    fn  Match( &self, parser: &mut Parser, sink: FieldIn< '_>);
    fn  Parse( &self, parser: &mut Parser, mark: U32, sink: FieldIn< '_>) -> Option< U32>;
}
```
* **Detached Match & Processing**: The match phase (`IGrammar::Match`) is decoupled from processing semantics. When a shard's grammar matches, it deposits its results (such as matching offsets or markers) onto the local `IForge` instance created for that grammar.
* **Drop-Based Backtracking**: If matching fails, the `Drop` implementation of the local forge automatically rolls back the parent's marker to its original starting state. If matching succeeds, the local forge propagates its outcome (and triggers any related semantic actions, as in `ActionForge`) by calling `self._Parent.Deposit(Some(mark))` upon destruction.
* **Stream Coupling**: The parser reads tokens directly utilizing the `IFluxOutSource` data stream mechanism, seamlessly integrating with Kosh's broader ecosystem.

## ShardTree Macro

The `ShardTree!` macro is the DSL interface for defining grammars:
* It keeps only the leaf resolution rules (`Charset`, `StrShard`) and UniNode constructor helpers (`@action`, `@repeat`).
* Delegates all recursive operator precedence parsing (infix choice `|`, sequencing `<`) and modifiers (`*`, `+`, `[ |p| body ]`) directly to Kosh's centralized `NodeTree!` macro.
