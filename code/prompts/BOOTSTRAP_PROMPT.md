# RALPH BOOTSTRAP - Configure Workflow for Repository

## Purpose

This prompt configures the Ralph workflow for your repository by:
1. **Detecting** your language, framework, and build tools
2. **Updating** `AGENTS.md` with your actual commands
3. **Updating** `TECHNICAL_STANDARDS.md` with patterns from your code
4. **Validating** that commands work

**Prerequisites:** The `.agents/code/` template files must already exist.
If they don't, copy them from the Ralph workflow package first.

---

## PHASE 1: VERIFY TEMPLATES EXIST

First, check that template files exist:

```bash
ls -la .agents/code/
```

Required files:
- `.agents/code/AGENTS.md`
- `.agents/code/TECHNICAL_STANDARDS.md`
- `.agents/code/SPEC_PROMPT.md`
- `.agents/code/PLANNING_PROMPT.md`
- `.agents/code/scripts/spec.sh`

If missing, inform user to copy templates first.

---

## PHASE 2: DETECT PROJECT STACK

Use subagents (up to 50 parallel) to scan the repository.

### 2.1 Find Package/Build Files

Search for these files and read them:

| File | Stack | Read For |
|------|-------|----------|
| `package.json` | Node.js | scripts, dependencies, name |
| `tsconfig.json` | TypeScript | compiler options |
| `Cargo.toml` | Rust | package name, dependencies |
| `go.mod` | Go | module name |
| `pyproject.toml` | Python | build system, dependencies |
| `setup.py` | Python | package info |
| `requirements.txt` | Python | dependencies |
| `Gemfile` | Ruby | dependencies |
| `pom.xml` | Java/Maven | build config |
| `build.gradle` | Java/Kotlin/Gradle | build config |
| `*.csproj` | C#/.NET | project config |
| `mix.exs` | Elixir | project config |
| `composer.json` | PHP | dependencies |
| `Makefile` | Various | targets |
| `CMakeLists.txt` | C/C++ | build config |

### 2.2 Find Config Files

| File | Indicates |
|------|-----------|
| `.eslintrc*`, `eslint.config.*` | ESLint |
| `biome.json` | Biome |
| `.prettierrc*` | Prettier |
| `rustfmt.toml` | Rust formatting |
| `clippy.toml` | Rust linting |
| `.golangci.yml` | Go linting |
| `mypy.ini`, `pyrightconfig.json` | Python typing |
| `.rubocop.yml` | Ruby linting |
| `jest.config.*` | Jest |
| `vitest.config.*` | Vitest |
| `pytest.ini`, `pyproject.toml [tool.pytest]` | Pytest |

### 2.3 Check CI/CD for Commands

Read these for actual working commands:
- `.github/workflows/*.yml`
- `.gitlab-ci.yml`
- `Jenkinsfile`
- `.circleci/config.yml`

Extract:
- Build commands
- Test commands
- Lint commands
- Format commands

### 2.4 Analyze Directory Structure

```bash
find . -type d -maxdepth 2 | grep -v node_modules | grep -v .git | grep -v __pycache__
```

Note:
- Source directory (src/, lib/, app/, pkg/, internal/)
- Test directory (test/, tests/, spec/, __tests__/)
- Config location
- Entry point

---

## PHASE 3: DETERMINE COMMANDS

Based on detected stack, determine the actual commands.

### Node.js / TypeScript

```bash
# Read package.json scripts
cat package.json | grep -A 50 '"scripts"'
```

Map to:
| Need | Check For | Default |
|------|-----------|---------|
| Build | `build`, `compile` | `npm run build` |
| Test all | `test` | `npm test` |
| Test one | - | `npm test -- --grep {name}` or `jest {name}` |
| Lint | `lint` | `npm run lint` or `eslint .` |
| Format | `format`, `prettier` | `npm run format` or `prettier --write .` |
| Type check | `typecheck`, `tsc` | `tsc --noEmit` |

### Rust

| Need | Command |
|------|---------|
| Build | `cargo build` |
| Build release | `cargo build --release` |
| Test all | `cargo test` |
| Test one | `cargo test {name} -- --exact` |
| Lint | `cargo clippy -- -D warnings` |
| Format | `cargo fmt` |
| Format check | `cargo fmt --check` |

### Go

| Need | Command |
|------|---------|
| Build | `go build ./...` |
| Test all | `go test ./...` |
| Test one | `go test -run {name} ./...` |
| Test verbose | `go test -v ./...` |
| Lint | `golangci-lint run` |
| Format | `go fmt ./...` |

### Python

```bash
# Check for pytest vs unittest
ls pytest.ini pyproject.toml 2>/dev/null | head -1
grep -l "pytest" pyproject.toml 2>/dev/null
```

| Need | Pytest | Unittest |
|------|--------|----------|
| Test all | `pytest` | `python -m unittest discover` |
| Test one | `pytest -k {name}` | `python -m unittest {name}` |
| Test file | `pytest {file}` | `python -m unittest {file}` |
| Verbose | `pytest -v` | `python -m unittest -v` |

Linting:
| Tool | Command |
|------|---------|
| ruff | `ruff check .` |
| flake8 | `flake8` |
| pylint | `pylint src/` |

Type checking:
| Tool | Command |
|------|---------|
| mypy | `mypy .` |
| pyright | `pyright` |
| pyrefly | `pyrefly check` |

### Ruby

| Need | Command |
|------|---------|
| Install | `bundle install` |
| Test all | `bundle exec rspec` or `rake test` |
| Test one | `bundle exec rspec {file}:{line}` |
| Lint | `bundle exec rubocop` |
| Format | `bundle exec rubocop -a` |

### Elixir

| Need | Command |
|------|---------|
| Build | `mix compile` |
| Test all | `mix test` |
| Test one | `mix test {file}:{line}` |
| Format | `mix format` |
| Lint | `mix credo` |

---

## PHASE 4: EXTRACT CODE PATTERNS

Use subagents to analyze 5-10 source files.

### 4.1 Naming Conventions

Look at actual code for:
- File names (snake_case, kebab-case, PascalCase)
- Function names (camelCase, snake_case)
- Class/type names (PascalCase)
- Variable names
- Constant names (SCREAMING_SNAKE)

### 4.2 File Organization

Note from existing files:
- Import ordering
- Section organization
- Where types are defined
- Where constants live

### 4.3 Error Handling

Find patterns:
```bash
# Examples of what to search
grep -rn "throw\|raise\|Error\|Exception\|Result<\|unwrap\|expect" src/ | head -20
```

Note the project's error handling style.

### 4.4 Test Patterns

Look at existing tests for:
- Test file naming
- Test function naming
- Setup/teardown patterns
- Assertion style
- Mocking approach

---

## PHASE 5: UPDATE AGENTS.MD

Read current `.agents/code/AGENTS.md` and update placeholders with detected values.

### Template Sections to Update

**Project Info:**
```markdown
| Property | Value |
|----------|-------|
| Name | {detected_name} |
| Language | {detected_language} |
| Version | {detected_version} |
| Framework | {detected_framework} |
| Build Tool | {detected_build_tool} |
| Test Framework | {detected_test_framework} |
| Package Manager | {detected_package_manager} |
```

**Quick Reference:**
```markdown
| Action | Command |
|--------|---------|
| Install deps | `{detected_install}` |
| Build (dev) | `{detected_build}` |
| Build (prod) | `{detected_build_release}` |
| Test all | `{detected_test_all}` |
| Test one | `{detected_test_one}` |
| Test file | `{detected_test_file}` |
| Lint | `{detected_lint}` |
| Lint fix | `{detected_lint_fix}` |
| Format | `{detected_format}` |
| Format check | `{detected_format_check}` |
| Type check | `{detected_type_check}` |
| Clean | `{detected_clean}` |
```

**Directory Structure:**
```markdown
```
{actual_structure}
```
```

**Key Files:**
```markdown
| Purpose | Location |
|---------|----------|
| Main entry | {detected_entry} |
| Config | {detected_config} |
| Dependencies | {detected_deps_file} |
```

Use `str_replace` to update each section.

---

## PHASE 6: UPDATE TECHNICAL_STANDARDS.MD

Read current `.agents/code/TECHNICAL_STANDARDS.md` and update with extracted patterns.

### Sections to Update

**Language Info:**
```markdown
| Property | Value |
|----------|-------|
| Language | {detected_language} |
| Version | {detected_version} |
| Style Guide | {style_guide_url} |
```

**Naming Conventions:**
Update with actual conventions observed in code.

**File Organization:**
Update with actual patterns from existing files.

**Error Handling:**
Update with project's actual error handling pattern, including code example.

**Test Structure:**
Update with actual test pattern from existing tests.

Use `str_replace` to update each section.

---

## PHASE 7: VALIDATE COMMANDS

Test that detected commands work:

```bash
# Test build (should work or show meaningful error)
{BUILD_COMMAND} 2>&1 | head -20

# Test that test framework is available
{TEST_COMMAND} --help 2>&1 | head -5

# Test lint
{LINT_COMMAND} --help 2>&1 | head -5
```

### Handle Failures

If a command fails:

1. **Missing tool:** Note in AGENTS.md
   ```markdown
   ## Setup Required
   
   ⚠️ Install required tools:
   - `{tool}`: {install_command}
   ```

2. **Wrong command:** Try alternatives, update if found

3. **Config issue:** Note what config is needed

---

## PHASE 8: VERIFY SCRIPT

Test that spec.sh works:

```bash
# Test the script (don't actually create)
.agents/code/scripts/spec.sh --help 2>&1 || .agents/code/scripts/spec.sh 2>&1 | head -10
```

Ensure it's executable:
```bash
chmod +x scripts/spec.sh
```

---

## PHASE 9: VERIFY SCRIPT

Review the .agents/code/PLANNING_PROMPT.md

---

## PHASE 10: VERIFY SCRIPT

Test that spec.sh works:

```bash
# Test the script (don't actually create)
.agents/code/scripts/spec.sh --help 2>&1 || .agents/code/scripts/spec.sh 2>&1 | head -10
```

Ensure it's executable:
```bash
chmod +x scripts/spec.sh
```

---

## PHASE 11: COMPLETION REPORT

Output summary:

```
╔══════════════════════════════════════════════════════════════╗
║              RALPH WORKFLOW CONFIGURED                       ║
╠══════════════════════════════════════════════════════════════╣
║ Detected Stack:                                              ║
║   Language:     {LANGUAGE} {VERSION}                         ║
║   Framework:    {FRAMEWORK}                                  ║
║   Build Tool:   {BUILD_TOOL}                                 ║
║   Test:         {TEST_FRAMEWORK}                             ║
║   Lint:         {LINT_TOOL}                                  ║
╠══════════════════════════════════════════════════════════════╣
║ Updated Files:                                               ║
║   .agents/code/AGENTS.md           ✓                         ║
║   .agents/code/TECHNICAL_STANDARDS.md  ✓                     ║
╠══════════════════════════════════════════════════════════════╣
║ Commands Verified:                                           ║
║   Build:     {✓/⚠️} {command}                                ║
║   Test:      {✓/⚠️} {command}                                ║
║   Lint:      {✓/⚠️} {command}                                ║
║   Format:    {✓/⚠️} {command}                                ║
╠══════════════════════════════════════════════════════════════╣
║ Ready to use!                                                ║
║                                                              ║
║ Start a task:                                                ║
║   .agents/code/scripts/spec.sh my-feature                               ║
║   cat .agents/code/SPEC_PROMPT.md | claude-code              ║
╚══════════════════════════════════════════════════════════════╝
```

If issues found:
```
⚠️ Manual setup needed:
- {issue}: {what to do}
```

---

## RULES

1. **Templates must exist** - Don't create from scratch
2. **Detect, don't assume** - Use actual file contents
3. **Validate commands** - Test before claiming success
4. **Preserve structure** - Update sections, don't rewrite files
5. **Report gaps** - Note what needs manual attention

## PRIORITY

9. Verify templates exist first
99. Detect actual stack from files
999. Extract patterns from real code
9999. Update both AGENTS.md and TECHNICAL_STANDARDS.md
99999. Validate commands work

999999. **USE STR_REPLACE TO UPDATE - DON'T REWRITE FILES**
9999999. **REPORT WHAT NEEDS MANUAL SETUP**
