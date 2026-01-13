# Technical Standards

> Coding standards and patterns for this project.
> Follow these in ALL implementations.

## Language

| Property | Value |
|----------|-------|
| Language | {LANGUAGE} |
| Version | {LANGUAGE_VERSION} |
| Platform | {PLATFORM} |
| Style Guide | {STYLE_GUIDE} |

---

## Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files | {FILE_CONVENTION} | {FILE_EXAMPLE} |
| Directories | {DIR_CONVENTION} | {DIR_EXAMPLE} |
| Types/Classes | {TYPE_CONVENTION} | {TYPE_EXAMPLE} |
| Functions/Methods | {FUNC_CONVENTION} | {FUNC_EXAMPLE} |
| Variables | {VAR_CONVENTION} | {VAR_EXAMPLE} |
| Constants | {CONST_CONVENTION} | {CONST_EXAMPLE} |
| Private members | {PRIVATE_CONVENTION} | {PRIVATE_EXAMPLE} |
| Boolean | {BOOL_CONVENTION} | {BOOL_EXAMPLE} |

### Naming Guidelines

- Be descriptive and consistent across the codebase.
- Avoid abbreviations (except common ones like `id`, `db`).
- Use domain terms that match the product vocabulary.

---

## File Organization

{FILE_ORGANIZATION}

---

## Error Handling

- Never silently swallow errors.
- Add context when re-throwing or mapping errors.
- Use specific error types, not generic strings.
- Log at appropriate level.
- Fail fast on unrecoverable errors; handle recoverable errors gracefully.

### Error Type Guidelines

| Error Category | When to Use |
|----------------|-------------|
| Validation | Invalid input from user or external sources |
| NotFound | Requested resource does not exist |
| Unauthorized | Missing or invalid credentials |
| Timeout | Operation exceeded time limit |
| Internal | Unexpected system errors |

---

## Testing Standards

{TESTING_STANDARDS}

---

## Documentation

- All public items documented.
- Include examples for non-obvious usage.
- Document errors completely.
- Keep docs updated with code changes.

---

## Anti-Patterns to Avoid

- Placeholders (`todo!`, `unimplemented!`, `fatalError("Not implemented")`).
- Magic numbers; use named constants.
- Silent failures or hidden defaults on error.
- Commented-out code.

---

## Code Review Checklist

### Naming and Style
- Follows naming conventions.
- Consistent with existing code.
- No unclear abbreviations.

### Implementation
- No placeholders or TODOs in shipped paths.
- Explicit error handling with context.
- Edge cases handled; no duplicate implementations.

### Testing
- Tests exist for new logic.
- Tests cover edge cases and error paths.
- Async behavior tested where relevant.

### Documentation
- Public items documented.
- Errors documented; examples added when needed.

### Quality
- Format, lint, and type checks pass.
- Debug logging is gated and filtered for release.

---

## Performance Guidelines

- Measure before optimizing; focus on hot paths.
- Prefer clarity over cleverness.
- Document performance-critical code.
- Add benchmarks for critical paths.

---

## Security Guidelines

- Validate all external input.
- Sanitize output appropriately.
- Use parameterized queries.
- Never log sensitive data.
- Use constant-time comparison for secrets.
- Principle of least privilege.

---

## Async and Concurrency

{ASYNC_PATTERNS}
