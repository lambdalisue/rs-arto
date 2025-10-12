# Project-Specific Rules

## Code Comments

**IMPORTANT: All code comments MUST be written in English.**

- Complex logic should include explanatory comments
- TODO comments should be specific and actionable
- Use English for all comments to maintain consistency with the codebase
- Follow Rust documentation conventions (///, //!)

## Test Code

**Use `indoc` crate for multi-line string literals in tests.**

- Improves readability by preserving proper indentation
- Maintains clean test structure without escaping issues
- Example:
  ```rust
  let input = indoc! {"
      Line 1
      Line 2
      Line 3
  "};
  ```

## Module Organization

**Use modern Rust module system (Rust 2018+) instead of `mod.rs`.**

- Use `module_name.rs` for single-file modules
- Use `module_name/` directory with a file named after the module for multi-file modules
- Declare submodules within the parent module file, not in `mod.rs`

### Examples:

**Single-file module:**
```
src/
  main.rs
  utils.rs        // Declare as `mod utils;` in main.rs
```

**Multi-file module (OLD style - DO NOT USE):**
```
src/
  main.rs
  utils/
    mod.rs        // ❌ Avoid this pattern
    helper.rs
```

**Multi-file module (NEW style - USE THIS):**
```
src/
  main.rs
  utils.rs        // Contains `pub mod helper;`
  utils/
    helper.rs     // ✅ Preferred pattern
```

Or alternatively:
```
src/
  main.rs
  utils/
    helper.rs
    another.rs
  utils.rs        // Contains `pub mod helper; pub mod another;`
```

This modern approach provides:
- Clearer module hierarchy
- Better editor/IDE navigation
- Consistency with Rust 2018+ conventions
