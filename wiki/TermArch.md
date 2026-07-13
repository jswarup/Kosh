# TermTree Architecture

The TermTree framework is a high-performance, stack-allocated Abstract Syntax Tree (AST) designed for expressing and compiling symbolic algebraic expressions (terms) in Kosh. It is completely standalone and decoupled from other components, focusing on zero-heap construction, dynamic traversal, and compilation into Kosh's expressions repository (`ExprRepos`).

## Core Design Principles

1. **Zero-Heap Allocation**: Symbolic math expressions built using `TermTree!` reside entirely on the caller's stack frame via Rust's temporary lifetime extension of referenced structs (`&TermBinNode` and `&Term`), eliminating dynamic memory allocation overhead.
2. **Decoupled AST and Compiler**: The AST structure ([termtree.rs](../src/fresco/termtree.rs)) is strictly separated from the compilation target ([exprrepos.rs](../src/fresco/exprrepos.rs)). Traversal is defined generically via the `ITermNode` trait.
3. **Cache-Friendly Traversal**: Dynamic dispatch is restricted to traversal time, keeping node construction instant and memory-efficient.

---

## Core Components

The framework is built around three elements in [termtree.rs](../src/fresco/termtree.rs):

### 1. Term (Leaves)
A leaf node representing either a variable or a real numeric literal:
* `Term::String(String)`: Symbolic variable name.
* `Term::Real(f64)`: Floating-point numeric literal.

### 2. TermBinNode (Binary Nodes)
A type alias of Kosh's generic `BinNode<L, R>`. It is parameterized over left (`L`) and right (`R`) branches, allowing nested trees of diverse types to compile down to nested stack-allocated structures.

### 3. BinOp Enum (Unified Binary Operators)
Represents supported arithmetic and structural operations across all modules in Kosh:
* `Sum = 0`: Addition (`+`)
* `Prod = 1`: Multiplication (`*`)
* `Sub = 2`: Subtraction (`-`)
* `Div = 3`: Division (`/`)
* `Pow = 4`: Exponentiation (`^`)
* `None = 5`: Terminal leaf sentinel
* `Less = 6`: Sequential sequence
* `Bor = 7`: Parallel choice

---

## Traversal and Serialization

Nodes implement the following traits:

### 1. `ITermNode`
Provides generic tree-walking operations:
```rust
pub trait ITermNode: INode {
    fn ChildrenCount(&self) -> usize;
    fn Child(&self, idx: usize) -> &dyn ITermNode;
    fn Op(&self) -> BinOp;
    fn AsLeaf(&self) -> Option<&Term>;
}
```
Implementations are provided for `Term`, `BinNode` (aliased as `TermBinNode`), and references `&T`.

### 2. `IXFluxSource`
Enables serializing TermTree nodes into JSON or other stream formats (e.g., for logging or tracing expressions) with standard `"Op"`, `"Left"`, and `"Right"` keys.

---

## DSL Macro (`TermTree!`)

The `TermTree!` macro builds right-associative algebraic trees on the stack:
* Delegates all recursive operator precedence parsing (infix `+`, `*`, `-`, `/`, `^`) directly to Kosh's centralized `NodeTree!` macro.
* Defines a custom `@leaf` mapping that automatically wraps literals/identifiers using `.AsTermNode()`.

```rust
let tree = TermTree!( a + b * c );
```
This expands inline to a stack-allocated tree structure representing the operations, where operators are parsed with right-associativity.

---

## Compilation & Flattening

When a `TermTree` is compiled using `PostTermTree` inside [`exprrepos.rs`](../src/fresco/exprrepos.rs), it uses a recursive traversal to flatten associative chains (like consecutive additions `a + b + c`):

1. **Flattening**: If a parent operation matches a child operation (e.g., `Sum` parent with `Sum` children), the traversal flattens the child nodes into a flat array of operands, rather than creating nested binary nodes in the expression repository.
2. **Expression Repository Posting**: Operands are posted to the `ExprRepos` instance, resulting in efficient multi-operand expressions (like `PolyExpr` / `SumExpr`) that minimize compilation depth and execution overhead.
