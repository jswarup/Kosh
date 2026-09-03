# Kosh Project Core Rules

## 1. Architecture & Performance
- **Zero-Allocation Data Structures**: Eliminate heap allocation overhead. Use stack-allocated references (&\'a DynINode<\'a>) for tree-based structures instead of owned heap allocations (Box<DynINode<\'a>>). 
- **Inline Construction**: Implement AST node constructors via macro expansions or inline struct declarations to extend temporary lifetimes in the caller's stack frame. Avoid helper functions returning references to temporaries.
- **Strict Iteration Patterns**: Never use native Rust or ... in ... loops. Always use .Arr().Traverse(|item| { ... }), rr.Traverse(|item| { ... }), or USeg::New(U32::_0, count).Traverse(|i| { ... }). This preserves U32 as the native index type.
- **Graphics Ownership (rieze vs enst)**: 
  - rieze is purely a thin presentation layer (DOM state, IPC frames, Canvas 2D painting). It MUST NOT process geometry, transforms, or contexts.
  - enst orchestrates graphics sessions, cameras, and IPC. Graphics computation must be delegated through swarm to symph kernels.

## 2. Typing & Data Layout
- **Custom Numeric Types**: Always use project-defined types (U8, U16, U32, USz from uint.rs). Never use native Rust primitives (u8, u32, usize) unless forced by external APIs.
- **Native Indexing**: U32 is the strict standard for indexing, counts, and sizes. Avoid casting to/from usize (e.g., usize::from, .AsUsize()).
- **Iface Trait Pattern**: 
  - Define a strict 1:1 corresponding trait (IFoo) for public operational methods of concrete structs (Foo).
  - Keep internal plumbing and execution loops out of the trait. 
  - Inherent impl Foo blocks are strictly for constructors and private helpers.
  - Re-export both at module root (pub use foo::{Foo, IFoo};).

## 3. Implementation Plan Policy
- **Heap Usage Commentary**: Every generated implementation plan MUST include a dedicated section evaluating the anticipated heap usage and allocation impact.

## 4. Execution Principles
- **Surgical Precision**: Touch only what you must. Do not "improve" adjacent code, comments, or formatting. Clean up only your own mess (remove unused imports/variables your changes orphaned).
- **Simplicity First**: Write the minimum code needed to solve the problem. Do not build speculative features, "flexible" abstractions, or unnecessary error handling.
- **Goal-Driven Execution**: Define clear success criteria (e.g., "Write test, then make it pass"). Loop and verify independently before declaring completion.
