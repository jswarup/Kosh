# Kosh Project Rules

## Performance Guidelines
- **Avoid Box Allocation in AST Trees**: When designing tree-based data structures (like ASTs, shard trees, term trees), prefer stack-allocated references (`&'a DynINode<'a>`) rather than owned heap allocations (`Box<DynINode<'a>>`) to eliminate heap allocation overhead and maximize CPU cache locality.
- **Inline Shard Construction**: Implement AST node constructor logic via macro expansions or direct inline struct declarations to trigger temporary lifetime extension in the caller's stack frame. Avoid helper functions that return references to local variables/temporaries.

## Typing Guidelines
- **Use Project-Defined Numeric Types**: Throughout the project, use the custom numeric types defined in `uint.rs` (e.g. `U8`, `U16`, `U32`, `USz`) instead of Rust's native primitive types (`u8`, `u16`, `u32`, `usize`) as far as possible.

## Implementation Plan Policy
- **Heap Usage Commentary**: As a strict project policy, EVERY implementation plan MUST include a dedicated section commenting on the anticipated heap usage and allocation impact of the proposed changes.

