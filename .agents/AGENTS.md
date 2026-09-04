# Kosh Project Guidelines & Engineering Standards

## 1. Architecture & Performance
- **Dual Memory Lifecycle (`Stash` -> `Buff` -> `Arr`)**:
  - `std::vec::Vec` is strictly eliminated across the entire codebase.
  - Dynamic accumulation uses `Stash< T>` with raw pointer growth and amortized $O(1)$ operations.
  - Once populated, `stash.IntoBuff()` transfers ownership into an immutable, fixed-size `Buff< T>` with zero reallocations or wasted capacity.
  - Non-owning borrowed views use `Arr< 'a, T>` (`buff.Arr()`, `stash.Arr()`).
- **Zero-Allocation AST & Tree Invariants**:
  - Eliminate heap allocation overhead. Use stack-allocated references (`&'a DynINode<'a>`) for tree-based structures instead of owned heap allocations (`Box<DynINode<'a>>`).
  - AST node constructors must be implemented via macro expansions (`NodeTree!`, `ShardTree!`, `TermTree!`, `ChoreTree!`) or inline struct declarations to extend temporary lifetimes in the caller's stack frame.
  - Avoid helper functions returning references to temporaries.
- **Strict Iteration Patterns (Zero-Range, `U32`-Preserving)**:
  - NEVER use native Rust `for ... in ...` loops or integer range conversions (`0..count.0`).
  - Always use `.Arr().Traverse( |item| { ... })`, `arr.Traverse( |item| { ... })`, or `USeg::New( U32::_0, count).Traverse( |i| { ... })`.
  - For reverse iteration, use `arr.TraverseRev( ... )` or `useg.TraverseRev( ... )`.
  - This preserves `U32` as the native index type across all loops and avoids loop index conversions.
- **Subsystem Ownership & Graphics Boundaries**:
  - `silo`: Foundational zero-allocation containers (`Arr`, `Buff`, `Stash`, `Stk`, `USeg`) and transparent custom unsigned numerics (`U8`..`U64`).
  - `stalks`: AST tree primitives, atomic concurrency (`Atm< T>`), stackful coroutines (`Coro`).
  - `shard`: Zero-heap recursive-descent grammar combinators and streaming JSON.
  - `fresco`: Symbolic algebra expressions and term repositories.
  - `flux`: High-performance streaming visitor serialization/deserialization framework (`IFluxExportSource`, `IFluxImportSink`, `ImplFluxSource!`).
  - `swarm` / `symph`: Heterogeneous hardware compute engine across CPU (multithreaded SIMT), WebGPU (`rust-gpu` SPIR-V bytecodes), and CUDA (PTX).
  - `heist`: Work-stealing chore DAG engine, data-parallel `SpawnQuell`, and coroutine fibers.
  - `rube`: Discrete-event digital logic simulation engine (`Layout`, `SimEngine`, `SimContext`, `Reg`, warps, triggers).
  - `fleck`: 3D Point Cloud (.pts) & Wavefront (.obj) mesh parsing, spatial bounding boxes, `Vex`.
  - `fenst`: Graphics orchestrator, camera state, asset loading, multi-GPU session management. Graphics computation must be delegated through `swarm` to `symph` kernels.
  - `wxfrieze` / `frieze`: Pure thin presentation layer (native wxWidgets 3.2+ desktop workspace with direct GPU canvas). It MUST NOT process geometry, compute transforms or camera matrices, cull primitives, or execute graphics shaders.

## 2. Typing, Numerics & Traits
- **Custom Numeric Types**:
  - Always use project-defined types (`U8`, `U16`, `U32`, `U64`, `USz` from `silo::uint`).
  - Never use native Rust primitives (`u8`, `u32`, `usize`, etc.) unless strictly required by external C/system APIs.
  - Always explore alternatives within the framework before resorting to primitives.
  - All public and internal method arguments and return types must use Custom Numeric Types.
  - Function arguments accepting numeric values, sizes, or indices should be parameterized by respective `Into` traits (e.g., `Idx: Into< U32>`, `Sz: Into< U32>`).
  - Use `From` trait implementations and `.into()` for conversions.
- **Native Indexing Standard**:
  - `U32` is the strict standard for indexing, offsets, counts, and sizes.
  - Core containers (`Arr`, `Buff`, `Stash`) implement `Index< I>` and `IndexMut< I>` where `I: Into< U32>`.
  - Slicing sub-ranges must use `.Slice()[start..end]` or `.SliceMut()[start..end]`.
  - Avoid casting to/from `usize` (e.g., `usize::from`, `.AsUsize()`) except when strictly required by external slice indexing boundaries.
- **Iface Trait Pattern**:
  - Define a strict 1:1 corresponding trait (`IFoo`) for public operational methods of concrete structs (`Foo`).
  - Keep internal plumbing, mutable state buffers, and execution loops out of the trait.
  - Inherent `impl Foo` blocks are strictly for constructors (`New`, `WithCapacity`) and private helpers.
  - Keep traits minimal, concise, and sufficient.
  - Re-export both at module root: `pub use foo::{Foo, IFoo};`.

## 3. Strict Formatting & Syntax Standards
- **Automated Formatters**:
  - DO NOT use automated formatters like `rustfmt`, as they will destroy project macros, bracket spacing, and column alignment.
- **Indentation, Line Endings & Braces**:
  - 4 spaces indentation (`tab_spaces = 4`).
  - Unix line endings (LF).
  - Opening brace `{` MUST be placed on a **newline** *only* for `struct`, `impl`, and `fn` declarations.
  - Opening brace `{` MUST remain on the **same line** for all control flow (`if`, `else`, `match`, `while`, `loop`) and closures.
- **Spacing in Parentheses & Brackets**:
  - Open parenthesis `(` MUST be followed by a space if not empty (e.g., `( val)`, `( a, b)`). Empty parentheses remain `()`.
  - Open angular bracket `<` (generics) MUST be followed by a space if not empty (e.g., `Buff< T>`, `Result< ()>`). Empty brackets remain `<>`. Less-than comparison operators (`<`) are unaffected.
- **Keyword & Variable Spacing**:
  - `fn` in all function declarations must be immediately followed by a tab character (`\t`) (e.g., `fn\tSize()`, `pub fn\tNew()`).
  - `let` in all local variable declarations must be immediately followed by exactly two spaces and a tab character (`\t`) (e.g., `let  \tmyVal = ...`).
  - `use` in all import statements must be immediately followed by a tab character (`\t`) (e.g., `use\tcrate::silo;`).
  - The outer-most opening brace `{` following `use` must remain on the same line as the `use` path/keyword (e.g., `use\tcrate::{`).
- **Return Statements**:
  - `return` MUST always be on its own line, never inline.
- **In-Line Comments**:
  - All trailing/in-line comments (sharing a line with code, excluding full-line comments and separator lines) must align to column 72 onwards.
- **Separator Lines**:
  - Separator lines (`//---------------------------------------------------------------------------------------------------------------------------------`) must have an empty line preceding and succeeding them.
  - Exception: when the line above starts with a comment itself, no empty line is required. If the line below is a comment and the line above is a separator line, no empty line is required.
- **Naming Conventions**:
  - Types / Structs / Enums: `PascalCase` (e.g., `SimEngine`, `TriggerWad`).
  - Traits: `PascalCase` with an `I` prefix (e.g., `IModule`, `IFluxExportSource`, `IAccess`).
  - Functions / Methods: `PascalCase` (e.g., `fn\tAdvance()`, `fn\tNew()`).
  - Local Variables & Arguments: `camelCase` (e.g., `modId`, `initialVal`).
  - Struct Fields: `PascalCase` preceded by an underscore `_` (e.g., `_Data`, `_Size`, `_InPorts`).
- **Vertical Column Alignment**:
  - All type definitions for struct data members must be vertically column-aligned across field declarations.
  - The right-hand side (RHS) in struct field initializations must also be vertically column-aligned.

## 4. Code Organization & Imports
- **Imports**:
  - All `use` statements must be placed strictly at the file header, logically grouped.
  - NEVER use inline full-path qualifications (e.g., `crate::silo::Stash`). Import short names at the file header and use them exclusively.
- **Macros**:
  - Exercise extreme care when modifying macros (e.g., `ImplUIntTraits!`, `ImplFluxSource!`, `NodeTree!`, `Stash!`). Formatting rules inside macro DSL tokens might differ.

## 5. Implementation Plan Policy
- **Heap Usage Commentary**:
  - Every generated implementation plan MUST include a dedicated section evaluating the anticipated heap usage and allocation impact.

## 6. Execution Principles & Agent Workflow
- **Think Before Coding**:
  - State assumptions explicitly.
  - Surface trade-offs before implementing changes.
  - If a simpler approach exists, push back. If unclear, stop and ask.
- **Surgical Precision**:
  - Touch only what you must. Do not "improve" adjacent code, comments, or formatting.
  - Clean up only your own mess (remove unused imports/variables your changes orphaned).
- **Simplicity First**:
  - Write the minimum code needed to solve the problem. Do not build speculative features, "flexible" abstractions, or unnecessary error handling.
- **Goal-Driven Execution**:
  - Define clear success criteria (e.g., "Write test, then make it pass").
  - Loop and verify independently before declaring completion.
- **Verification**:
  - Always verify modifications with `cargo build` and `cargo test`.
  - Ensure all tests pass with zero warnings before declaring completion.
- **Commit Directive**:
  - Never commit without an explicit directive from the user.
- **Always Review**:
  - Review the final diff for minimal footprint and strict compliance with project invariants before declaring completion.
