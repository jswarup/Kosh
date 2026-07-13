# Kosh

Rust CLI development project. The intention is to develop a fast geometry, Constraints and BRep kernel, using modern programing paradigms

## Unified AST Architecture

Kosh features a unified Abstract Syntax Tree (AST) design defined in [stalks/node.rs](src/stalks/node.rs) that consolidates the binary and unary node structures across all modules:

* **Unified Nodes**: Generics `BinNode<L, R, Op>` and `UniNode<C, Op>` represent binary and unary nodes across the entire AST framework.
* **Unified Binary Operators**: The `BinOp` enum defines all binary operators:
  - Arithmetic operators: `Sum = 0`, `Prod = 1`, `Sub = 2`, `Div = 3`, `Pow = 4`.
  - Traversal/Structural operators: `Less = 6` (`<`), and `Bor = 7` (`|`).
* **Centralized Parser (`NodeTree!`)**: A generic, highly optimized recursive macro `NodeTree!` handles infix operator precedence and prefix rule parsing. Domain-specific macros (`ChoreTree!`, `TermTree!`, `ShardTree!`) delegate parsing to `NodeTree!` and only contain leaf/node construction calls.
* **Readable AST Output**: Implementations of `std::fmt::Display` and `std::fmt::Debug` format the unified node trees into readable symbolic infix expressions (e.g. `(a < (b | c))`).

## Documentation

* [Heist Architecture](wiki/HeistArch.md) - Work-stealing scheduling & dependency resolution framework. 
* [Shard Architecture](wiki/ShardArch.md) - Zero-heap allocation AST framework for recursive grammar parsing.
* [TermTree Architecture](wiki/TermArch.md) - Standalone algebraic term AST and compilation framework.
