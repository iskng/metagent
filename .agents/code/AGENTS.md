# Agent Instructions

> Build/test commands and learnings for this project.
> **Keep brief. NO status updates.**

## Project Info

| Property | Value |
|----------|-------|
| Name | {PROJECT_NAME} |
| Language | {LANGUAGE} |
| Platform | {PLATFORM} |
| Framework | {FRAMEWORK} |
| Build Tool | {BUILD_TOOL} |
| Test Framework | {TEST_FRAMEWORK} |
| Package Manager | {PACKAGE_MANAGER} |

---

## Quick Reference

| Action | Command |
|--------|---------|
| Install deps | `{INSTALL_COMMAND}` |
| Build (dev) | `{BUILD_DEV_COMMAND}` |
| Build (prod) | `{BUILD_PROD_COMMAND}` |
| Test all | `{TEST_ALL_COMMAND}` |
| Test one | `{TEST_ONE_COMMAND}` |
| Test file | `{TEST_FILE_COMMAND}` |
| Lint | `{LINT_COMMAND}` |
| Lint fix | `{LINT_FIX_COMMAND}` |
| Format | `{FORMAT_COMMAND}` |
| Format check | `{FORMAT_CHECK_COMMAND}` |
| Type check | `{TYPE_CHECK_COMMAND}` |
| Clean | `{CLEAN_COMMAND}` |

---

## Build Commands

```bash
# Development build
{BUILD_DEV_COMMAND}

# Production build
{BUILD_PROD_COMMAND}

# Clean artifacts
{CLEAN_COMMAND}
```

---

## Test Commands

```bash
# Run all tests
{TEST_ALL_COMMAND}

# Run single test by name
{TEST_ONE_COMMAND}

# Run tests in a specific file/module
{TEST_FILE_COMMAND}

# Run tests with verbose output
{TEST_VERBOSE_COMMAND}
```

### Test Execution Notes

- **Single test execution is CRITICAL** - Ralph needs fast feedback
- **Pattern matching** - Use to run related tests quickly

---

## Lint & Format

```bash
# Check linting
{LINT_COMMAND}

# Fix linting issues
{LINT_FIX_COMMAND}

# Fix formatting
{FORMAT_COMMAND}

# Check formatting (no changes)
{FORMAT_CHECK_COMMAND}
```

### Validation Chain Order

Run in this order for fast feedback:
1. Format check (fastest)
2. Lint
3. Type check
4. Build
5. Test (specific)
6. Test (all)

---

## Environment

```bash
# Environment variables needed for development
{ENV_VARS}
```

---

## Directory Structure

```
{PROJECT_STRUCTURE}
```

---

## Key Files

| Purpose | Location |
|---------|----------|
| Main entry | {ENTRY_POINT} |
| Config | {CONFIG_FILE} |
| Dependencies | {DEPS_FILE} |

---

## Key Patterns

{KEY_PATTERNS}

---

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| {ISSUE_1} | {SOLUTION_1} |

---

## Debugging Tips

- {DEBUG_TIP_1}

---

## Update Guidelines

**DO add:**
- New commands discovered
- Working solutions to issues
- Debugging techniques
- Important patterns
- Environment requirements

**DO NOT add:**
- Status updates ("working on X")
- Progress reports ("50% complete")
- Task tracking (use plan.md)
- Temporary notes
- Personal comments


---

## Ralph Integration

**Subagent limits:**
- Search/read: up to 100 parallel
- Write: up to 50 parallel
- Build/test: **1 ONLY**

**Back pressure chain:**
1. Format check - < 1 second
2. Lint - 1-10 seconds
3. Type check - 5-30 seconds
4. Build - 10-120 seconds
5. Unit tests - 5-60 seconds
6. Full suite - 1-10 minutes
