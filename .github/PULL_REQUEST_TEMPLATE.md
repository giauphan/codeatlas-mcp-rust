## Description
Please include a summary of the change and which issue is fixed or feature implemented.

Fixes # (issue)

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring / Performance improvement

## Quality Checklist
- [ ] Code follows Clean Architecture layer constraints (pure domain, isolated adapters)
- [ ] All Rust source files comply with the sizing ceiling (< 450 lines per module)
- [ ] `cargo fmt --all -- --check` passes with zero formatting diffs
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with zero warnings
- [ ] `cargo test --workspace` passes all unit and integration tests
- [ ] Documentation has been updated accordingly
