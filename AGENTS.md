## Rust

Before considering work complete, run:

```
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

- Prefer idiomatic Rust and explicit error handling.
- Don't use unwrap() or expect() in normal runtime paths unless failure genuinely represents an invariant violation.
- Before adding a dependency:
    - Check whether the standard library or an existing dependency is sufficient.
    - Prefer focused, well-maintained crates.
    - Avoid large dependencies for trivial functionality.
    - Do not introduce new frameworks or architectural layers without a concrete need.
- Since using the iced library for Rust GUI development, use iced_test to test and refine changes. 

## Iterating changes

- Inspect the relevant existing code first.
- Use ponytail skill for implementing change if available
- Add or update tests for logic-heavy behavior.
- Run formatting, checks, linting, and tests.
- Do not opportunistically refactor unrelated code.

## Project management

- Persist current tasks and features to do in TODO.md
- Delete a task or feature once finished and verified.