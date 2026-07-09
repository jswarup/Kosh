# Kosh Project Rules

## Performance Guidelines
- **Avoid Box Allocation in AST Trees**: When designing tree-based data structures (like ASTs, shard trees, term trees), prefer stack-allocated references (`&'a DynINode<'a>`) rather than owned heap allocations (`Box<DynINode<'a>>`) to eliminate heap allocation overhead and maximize CPU cache locality.
- **Inline Shard Construction**: Implement AST node constructor logic via macro expansions or direct inline struct declarations to trigger temporary lifetime extension in the caller's stack frame. Avoid helper functions that return references to local variables/temporaries.
