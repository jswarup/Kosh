# Kosh Workspace & Formatting Rules

Welcome to Kosh! You **MUST** strictly adhere to these non-standard formatting and workflow rules. Do not use automated formatters like `rustfmt`, as they will destroy project macros and styling.

## 1. Strict Formatting & Syntax
- **Indentation & Braces**: 4 spaces, UNIX (LF) line endings. Opening braces `{` MUST be on a **newline** for `struct`, `impl`, and `fn`. For control flow (`if`, `match`), keep `{` on the same line.
- **Spacing in Brackets**: Open parenthesis `(` and angular bracket `<` MUST have a trailing space if not empty (e.g., `( val)`, `Buff< T>`).
- **Function/Keyword Spacing**: `fn` and `use` MUST be followed immediately by a tab (`\t`) (e.g., `fn\tSize()`, `use\tcrate::silo`).
- **Local Variables**: `let` MUST be followed by two spaces and a tab (`let  \tvarStr = ...`).
- **Return Statements**: `return` MUST always be on its own line, not inline.
- **Comments & Separators**: Trailing comments must align to column 72. Separator lines (`//---...`) MUST be padded with one blank line before and after.

## 2. Naming Conventions
- **PascalCase**: Structs, Enums, Types (e.g., `SimContext`), Methods/Functions (e.g., `fn\tAdvance()`).
- **Traits**: PascalCase with an `I` prefix (e.g., `IAccess`).
- **camelCase**: Local variables, function arguments (e.g., `initialVal`).
- **Struct Fields**: PascalCase preceded by an underscore `_` (e.g., `_Data`, `_Size`).
- **Alignment**: Struct field types and right-hand side initializations MUST be vertically aligned into consistent columns.

## 3. Code Organization
- **Imports**: All `use` statements must be placed strictly at the file header, logically grouped. NEVER use inline full-path qualifications (e.g., `crate::silo::Stash`). Import short names and use them exclusively.
- **Macros**: Be highly careful modifying macros (e.g., `ImplUIntTraits!`). Formatting rules might differ inside macro DSL tokens.

## 4. Agent Workflow Rules
- **Verify**: After modifications, always verify with `cargo build` and `cargo test`. Ensure tests pass before and after your edits.
- **Commit**: Never commit without an explicit directive from the user.
- **Think Before Coding**: State assumptions explicitly. Surface trade-offs. If a simpler approach exists, push back. If unclear, stop and ask.
