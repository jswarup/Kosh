# Kosh Project Rules

## Performance Guidelines
- **Avoid Box Allocation in AST Trees**: When designing tree-based data structures (like ASTs, shard trees, term trees), prefer stack-allocated references (`&'a DynINode<'a>`) rather than owned heap allocations (`Box<DynINode<'a>>`) to eliminate heap allocation overhead and maximize CPU cache locality.
- **Inline Shard Construction**: Implement AST node constructor logic via macro expansions or direct inline struct declarations to trigger temporary lifetime extension in the caller's stack frame. Avoid helper functions that return references to local variables/temporaries.

## Typing Guidelines
- **Use Project-Defined Numeric Types**: Throughout the project, use the custom numeric types defined in `uint.rs` (e.g. `U8`, `U16`, `U32`, `USz`) instead of Rust's native primitive types (`u8`, `u16`, `u32`, `usize`) as far as possible.
- **Struct Data Members**: All struct fields (data members) must be named in PascalCase preceded by an underscore `_` (e.g. `_Data`, `_Size`, `_Points`, `_BBoxMin`).

## Implementation Plan Policy
- **Heap Usage Commentary**: As a strict project policy, EVERY implementation plan MUST include a dedicated section commenting on the anticipated heap usage and allocation impact of the proposed changes.

## Graphics Ownership Rule
- **Frieze Is Presentation Only**: `frieze` is a thin presentation layer. It may manage DOM state, forward input, decode IPC frames, and paint backend-projected primitives through Canvas 2D. It MUST NOT parse geometry, calculate camera transforms, project vertices, cull, shade, sort geometry, or create WebGL/WebGPU contexts.
- **Fenst Orchestrates Graphics**: `fenst` owns graphics sessions, asset loading, camera state, frame serialization, and IPC. All graphics computation and render preparation MUST be delegated through `swarm` to `symph` kernels, with a CPU fallback only inside that backend pipeline.

## Architectural Guidelines
- **Iface Pattern**:
  - **1:1 Trait-per-Struct**: For a concrete struct `Foo`, define a corresponding trait `IFoo` containing all public non-constructor methods.
  - **Inherent Impl Limitation**: Inherent `impl Foo` blocks are reserved exclusively for constructors (e.g. `New`, `Create`, `From...`).
  - **Trait Implementation**: All functional/operational methods are implemented under `impl IFoo for Foo`.
  - **Module Re-exporting**: Re-export both `Foo` and `IFoo` at the module root (`pub use foo::{Foo, IFoo};`) so callers importing the module automatically bring the trait methods into scope.
  - **Object Safety & Generics**: Use `where Self: Sized` on generic trait methods to preserve object safety (`dyn IFoo`) for non-generic methods. Default implementations should be defined directly in `IFoo` when implemented via other trait methods.

## Code Organization & Naming Guidelines
- **Always Use Short Names & File-Level `use` Statements**:
  - All `use` statements must be placed strictly at the file header (grouped logically by module).
  - Do NOT use inline full-path qualifications (e.g. `crate::silo::Stash`, `crate::heist::choretree::IChoreNode`, `crate::swarm::SwarmEngine`, `std::sync::atomic::Ordering`).
  - Import traits, structs, enums, and functions at the file header and refer to them exclusively by their short names throughout the file body.
- **Casing & Naming Standards**:
  - **Structs, Enums, & Types**: `PascalCase` (e.g. `TriggerWad`, `SimContext`, `PortLayout`, `Reg`).
  - **Traits**: `PascalCase` with `I` prefix (e.g. `ITriggerWad`, `IPortLayout`, `IReg`, `IAccess`).
  - **Methods & Functions**: `PascalCase` (e.g. `fn\tNew()`, `fn\tAdd()`, `fn\tGet()`, `fn\tAdvance()`, `fn\tIsEdge()`, `fn\tSize()`).
  - **Function / Method Arguments**: `camelCase` (e.g. `val`, `initialVal`, `mxVert`, `biDirFlg`, `inSig`).
  - **Local Variables**: `camelCase` preceded by `let  \t` (two spaces + tab, e.g. `let  \tnameStr = `, `let  \tnewSize = `).
  - **Struct Data Members (Fields)**: Preceded by an underscore `_` followed by `PascalCase` (e.g. `_Data`, `_Size`, `_Names`, `_PastVal`, `_CurrentVal`, `_FutureVal`, `_X`).


