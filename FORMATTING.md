# Formatting Rules for Kosh

All Rust source files (`.rs`) in this project must adhere to the following formatting directives:

## 1. File & Block Structure
- **Indentation**: 4 spaces (`tab_spaces = 4` in `.rustfmt.toml`).
- **Line Endings**: Unix line endings (`newline_style = "Unix"` / LF).

## 2. Spacing in Parentheses & Brackets
- **Open Parenthesis**: An open parenthesis `(` must always be followed by a space, unless it encloses nothing (e.g., `( value)` or `( value )`, but `()` remains `()`).
- **Open Angular Bracket**: An open angular bracket `<` (used for generic parameters) must always be followed by a space, unless it encloses nothing (e.g., `Buff< T>` or `Result< ()>`, but `<>` remains `<>`). Less-than operators (`<`) are unaffected.

## 3. Function Declarations
- **Fn Keyword**: The `fn` keyword in all function declarations must be immediately followed by a tab character (`\t`) instead of a space (e.g., `fn\tfoo(...)` or `pub fn\tbar(...)`).

## 4. Local Variable Declarations
- **Indentation**: All local variable declarations (`let` statements) must be preceded by a tab character (`\t`) for indentation, rather than spaces (e.g., `\tlet foo = ...`).

