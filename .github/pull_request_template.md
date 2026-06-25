## Description

Please include a summary of the changes and why they're being made.

## Type of Change

- [ ] Bug fix (fixes an issue without changing existing functionality)
- [ ] New feature (adds new functionality)
- [ ] Breaking change (causes existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring
- [ ] Tests

## Checklist

### Library Changes (`crates/core`)

- [ ] Created processor file in `crates/core/src/processors/`
- [ ] Processor implements the `Processor` trait correctly
- [ ] Processor is registered in `crates/core/src/pipeline/registry.rs`
- [ ] Processor is exported in `crates/core/src/processors.rs`
- [ ] Added comprehensive documentation comments
- [ ] Added tests covering happy path and error cases

### UI Changes (`crates/ui`)

- [ ] Created new type in `InputOutputType` (if needed)
- [ ] Created node file in `crates/ui/src/nodes/`
- [ ] Node implements `BaseNode` trait correctly
- [ ] Input/output types match processor expectations
- [ ] Node exported in `crates/ui/src/nodes.rs`
- [ ] Node registered in `crates/ui/src/nodes/node_registry.rs`
- [ ] UI pins render correctly (tested manually)

### Code Quality

- [ ] Ran `cargo fmt` to format code
- [ ] Ran `cargo test` to verify everything works
- [ ] No compiler warnings or errors
- [ ] Code follows project conventions

## Testing

Describe how you tested your changes:

- [ ] Unit tests pass
- [ ] Manual testing completed
- [ ] Tested happy path scenario
- [ ] Tested error cases

## Related Issues

Closes #(issue number)

## Screenshots (if applicable)

If you made UI changes, please include screenshots showing the new functionality.

## Additional Notes

Add any other context or notes about your changes here.

---
