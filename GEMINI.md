# Kosh Workspace Instructions

Welcome to the Kosh project! Please adhere to the following project-specific instructions and rules when generating or modifying code in this repository.

## Formatting Rules
This project has strict, non-standard Rust formatting rules defined in `FORMATTING.md`. You **MUST** ensure all code modifications comply with these directives:

1. **Indentation & Braces**: 4 spaces, UNIX (LF) line endings. Opening braces `{` are on a **newline** for `struct`, `impl`, and `fn`. For control flow (`if`, `match`, etc.), keep `{` on the same line.
2. **Spacing in Brackets**:
   - Open parenthesis `(` MUST have a trailing space if not empty (e.g., `( val)`).
   - Open angular bracket `<` MUST have a trailing space if not empty (e.g., `Buff< T>`).
3. **Function declarations**: The `fn` keyword MUST be followed immediately by a tab (`\t`) (e.g., `fn\tSize()`).
4. **Local Variables**: The `let` keyword MUST be followed by two spaces and a tab (`let  \tvar = `). Variables must use `camelCase`.
5. **Separators**: All `//---...` separator lines MUST be padded with one blank line before and after.
6. **Trailing Comments**: Must be aligned to column 72.
7. **Use statements**: The `use` keyword MUST be followed by a tab (`\t`) (e.g., `use\tcrate::silo`). Prefer file-level use statements to full-path and function level use statements. Group file-level use statements.

8. **return**: Must be on separate line

When modifying files, please ensure you don't inadvertently reformat existing code that already complies with these rules. Be extremely careful when using automated formatting tools like `rustfmt`, as they will likely destroy these custom formatting rules.


## Build and Testing
After making changes, always verify them with:
- `cargo build`
- `cargo test`

## Commit
Never commit without explicit directive.

There are heavy macro usages throughout the codebase (e.g. `BiNodeTree!`, `ImplUIntTraits!`), so be careful when editing macros as the custom formatting rules may not apply to DSL tokens inside macros.
