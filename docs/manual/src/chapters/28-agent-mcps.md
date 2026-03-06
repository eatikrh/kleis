# Agent MCP Servers

Kleis ships four Model Context Protocol (MCP) servers that turn Kleis policies
and structures into live tools for AI agents. The previous chapter covered
**kleis-theory** — the interactive theory-building server. This chapter covers
the other three: **kleis-policy** for agent action gating, and
**kleis-review-rust** / **kleis-review-python** for code review against formal
coding standards.

All four share the same architecture: a Kleis `.kleis` file defines the rules,
the Kleis evaluator loads them into memory, and the MCP server exposes them as
JSON-RPC tools over stdio. The rules are declarative, version-controlled, and
readable by both humans and machines.

## kleis-policy: Agent Action Gating

### The Problem

An AI agent in an IDE can run shell commands, edit files, commit code, and push
to remote repositories. Without guardrails, a single hallucinated command —
`rm -rf /`, `git push --force main`, `chmod 777` — can cause real damage.

Static tool-call filters are too coarse. "Never allow shell commands" is
impractical. "Allow shell commands but deny dangerous patterns" requires
nuance that belongs in a policy, not hardcoded in the agent runtime.

### The Solution

The kleis-policy MCP loads a policy file and exposes five tools:

| Tool | Purpose |
|------|---------|
| `check_action` | Check whether an action is allowed, get preconditions |
| `list_rules` | Show all loaded policy rules |
| `explain_rule` | Explain a specific rule in detail |
| `describe_schema` | Show the full policy vocabulary |
| `evaluate` | Evaluate any Kleis expression or verify a proposition via Z3 |

The agent calls `check_action` before performing file edits, deletions,
commands, git operations, and commits. The policy returns "allow" or "deny"
plus any preconditions that must be satisfied first.

### The Policy File

The policy is a standard `.kleis` file using `define`, `if/then/else`,
and built-in string functions (`contains`, `hasPrefix`, `hasSuffix`, `isAscii`):

```kleis
define check_file_delete(path) =
    if hasPrefix(path, "src/") then "deny"
    else if hasPrefix(path, "tests/") then "deny"
    else if path = "Cargo.toml" then "deny"
    else "allow"

define check_run_command(cmd) =
    if contains(cmd, "rm -rf") then "deny"
    else if contains(cmd, "chmod 777") then "deny"
    else if contains(cmd, "pkill") then "deny"
    else "allow"

define check_git_push(branch, force) =
    if force = 1 then "deny"
    else if branch = "production" then "deny"
    else "allow"

define check_git_commit(description) =
    if isAscii(description) then "allow"
    else "deny"
```

These rules are not aspirational documentation — they are enforced at runtime.
When the agent calls `check_action("run_command", "rm -rf ./build")`, the
server evaluates `check_run_command("rm -rf ./build")` and returns `"deny"`.

### Preconditions

Beyond allow/deny, the policy defines preconditions — commands or checks
that must run before an action is permitted:

```kleis
define before_git_push(branch, force) =
    "cargo fmt --all && cargo clippy --all-targets --all-features && cargo test"

define before_file_edit(path) =
    if hasPrefix(path, "src/kleis_parser") then
        "cargo test parser && cat docs/grammar/kleis_grammar_v99.ebnf"
    else if hasPrefix(path, "src/evaluator") then "cargo test evaluator"
    else "none"
```

When the agent asks to edit `src/kleis_parser.rs`, the MCP responds: "allowed,
but first run parser tests and read the grammar." The agent executes the
preconditions, then proceeds with the edit.

### Z3 Verification of the Policy Itself

Because the policy is a Kleis program, its properties can be verified via Z3.
The `evaluate` tool accepts universal quantifiers:

```
evaluate: check_git_commit("fix: resolve parsing bug")
-> "allow"

evaluate: check_git_commit("initial commit")
-> "deny"

evaluate: forall(d : String). implies(check_git_commit(d) = "allow", isAscii(d))
-> VERIFIED
```

The third query proves that **every** commit message the policy allows is
ASCII. This is not a test case — it is a machine-checked proof over all
possible inputs. If someone modifies the policy in a way that breaks this
property, Z3 will catch it.

### Starting the Server

```bash
kleis mcp --policy examples/policies/agent_policy.kleis --verbose
```

In Cursor, add to `.cursor/mcp.json`:

```json
"kleis-policy": {
    "command": "/Users/you/bin/kleis",
    "args": ["mcp", "--verbose", "--policy",
             "/path/to/examples/policies/agent_policy.kleis"]
}
```

---

## kleis-review: Code Review via Formal Standards

> **Note:** Review MCPs are now specialized as `kleis-review-rust` and
> `kleis-review-python`. There may be more specialized review tools in the
> future, or we may combine them into one that is able to review multiple
> languages.

### The Problem

Code reviews catch two categories of issues:

1. **Mechanical** — `.unwrap()` in production code, wildcard imports, emoji
   in source files, `Result<_, String>` instead of typed errors. These are
   team conventions that reviewers enforce inconsistently depending on who
   reviews the MR and how tired they are.

2. **Architectural** — "should this be a middleware or a per-handler check?"
   These require understanding and judgment.

Linters handle syntax. AI agents handle architecture. But the mechanical
layer — team conventions, hard-won lessons, project-specific standards —
falls through the cracks. It lives in people's heads, gets enforced
inconsistently, and slows down every review.

### The Solution

The kleis-review MCP loads a coding standards file and exposes six tools:

| Tool | Purpose |
|------|---------|
| `check_code` | Check a source code snippet against all standards |
| `check_file` | Check a file on disk against all standards |
| `list_rules` | List all loaded coding standard rules |
| `explain_rule` | Explain a specific rule in detail |
| `describe_standards` | Show the full schema of loaded standards |
| `evaluate` | Evaluate a Kleis expression or verify a proposition via Z3 |

Each `check_*` function in the policy receives source code as a string and
returns `"pass"` or `"fail: <reason>"`. The engine runs all rules against
the input and returns a per-rule verdict.

### Three Tiers of Analysis

The review engine operates at three distinct levels of sophistication:

| Tier | Mechanism | What it catches |
|------|-----------|-----------------|
| **1. String checks** | `check_*` with `contains`/`hasPrefix` | Clippy patterns, emoji, `Result<_, String>`, `#[allow(unused)]` |
| **2. Structural checks** | `check_*_structural` via `rust_parser.kleis` AST | `.unwrap()` in production (not test) code, missing docs on pub fns, naming conventions, wildcard imports, function parameter counts |
| **3. Z3 axioms** | `structure` with `axiom`, verified by Z3 | Formal properties over all possible inputs (e.g., SafeCode, SqlSafe) |

Tier 1 runs simple pattern matching — fast but context-blind.
Tier 2 parses the source into a `Crate` AST and queries it for functions,
structs, use declarations, and comments. This enables context-aware analysis:
`.unwrap()` in a `#[test]` function is fine; `.unwrap()` in production code
is flagged. The structural parser was originally written in pure Kleis
(`rust_parser.kleis`) — a Rust parser written in Kleis itself. For performance,
`scan()` now delegates to a native Rust tokenizer and recursive descent parser
(`scan_rust` builtin), while all query helpers and data types remain in Kleis.
Tier 3 uses Z3 to prove properties hold universally.

### The Standards File

The standards file is a `.kleis` file. It imports the structural parser and
defines rules at each tier. Here is a representative sample:

**Tier 1 — String checks** for clippy patterns and style:

```kleis
define check_clippy_len_zero(source) =
    if contains(source, ".len() == 0") then
        "fail: clippy::len_zero — use .is_empty() instead"
    else if contains(source, ".len() > 0") then
        "fail: clippy::len_zero — use !.is_empty() instead"
    else "pass"

define check_no_result_string(source) =
    if contains(source, "Result<(), String>") then
        "fail: Result<(), String> — use thiserror for typed errors"
    else "pass"
```

**Tier 2 — Structural checks** using the Rust parser:

```kleis
import "../meta-programming/rust_parser.kleis"

define check_safe_structural(source) =
    let prod = production_code(source) in
    let c = scan(source) in
    let fns = crate_functions(c) in
    let unwrap_fns = non_test_fns_containing(source, fns, ".unwrap()") in
    let panic_fns  = non_test_fns_containing(source, fns, "panic!(") in
    let unsafe_hits = if contains(prod, "unsafe {")
                      then "unsafe block in production code" else "" in
    let all = append_failure(
        if eq(unwrap_fns, "") then "" else concat(".unwrap() in production fn: ", unwrap_fns),
        append_failure(
            if eq(panic_fns, "") then "" else concat("panic!() in production fn: ", panic_fns),
            unsafe_hits)) in
    if eq(all, "") then "pass"
    else concat("fail: ", all)
```

The `scan()` function parses source code into a `Crate` AST containing
`Function`, `UseDeclaration`, `Comment`, and `Struct` items. The
`production_code()` function filters out `#[test]` functions and
`#[cfg(test)]` modules using a single `foldLines` pass, so patterns are
only checked against production code.

Structural verdicts include line numbers for precise reporting:

```
fail: .unwrap() in production fn: process_request (line 42), handle_error (line 87)
```

The master `check_structural` check parses once and runs multiple sub-rules:

```kleis
define check_structural(source) =
    let c = scan(source) in
    let f1 = rule_wildcard_imports(c) in
    let f2 = rule_narrating_line_comments(c) in
    let f3 = rule_pub_fn_docs(c) in
    let f4 = rule_param_count(c) in
    let f5 = rule_function_count(c) in
    let f6 = rule_test_presence(c) in
    let f7 = rule_fn_naming(c) in
    let f8 = rule_struct_naming(c) in
    ...
```

Sub-rules are named `rule_*` (not `check_*`) so the engine does not
auto-discover them as top-level checks. They are invoked only through
`check_structural`.

**Tier 3 — Z3 axioms** for formal verification:

```kleis
structure SafeCode {
    operation is_safe : String -> Bool

    axiom safe_no_unwrap : forall(s : String).
        implies(is_safe(s), not(contains(s, ".unwrap()")))

    axiom safe_no_panic : forall(s : String).
        implies(is_safe(s), not(contains(s, "panic!(")))
}
```

The `evaluate` tool can verify these axioms via Z3:

```
evaluate: forall(s : String). implies(is_safe(s), not(contains(s, ".unwrap()")))
-> VERIFIED
```

This is not a test case — it is a machine-checked proof over all possible
inputs.

### The Five Structural Checks

| Check | What it detects (in production code only) |
|-------|------------------------------------------|
| `check_structural` | Wildcard imports, narrating comments, missing pub fn docs, too many params (>5), function/struct naming, function count (>50), missing tests |
| `check_safe_structural` | `.unwrap()`, `panic!()`, `unsafe {}` — skips test functions |
| `check_clean_structural` | `println!()`, `todo!()`, `dbg!()` — skips test functions |
| `check_secure_structural` | Hardcoded credential string literals (`password = "..."`, `secret = "..."`) — skips test code |
| `check_sql_safe_structural` | `format!` with SQL keywords, `.push_str` with `WHERE` — skips test code |

### Running a Review

The agent calls `check_file` with a path:

```
check_file: { "path": "src/server.rs" }

-> Code Review: src/server.rs — 14 passed, 4 failed (out of 18 checks)

   FAIL  check_safe_structural — .unwrap() in production fn: handle_request (line 42)
   FAIL  check_structural — pub fn missing doc comment: process (line 15), serve (line 80)
   FAIL  check_no_result_string — Result<(), String> — use thiserror
   FAIL  check_clippy_ptr_arg — use &str instead of &String
   PASS  (14 other rules)
```

Or `check_code` with a snippet for in-flight review during development:

```
check_code: { "source": "fn greet(name: &String) {\n    println!(\"hi\");\n}" }

-> 2 failures:
   check_clippy_ptr_arg: use &str instead of &String
   check_clean_structural: println! in production fn: greet (line 1)
```

### Adding New Rules

When a pattern bites you in production, add a rule:

```kleis
define check_no_expect_fun_call(source) =
    if contains(source, ".expect(&format!(") then
        "fail: clippy::expect_fun_call — use .unwrap_or_else or build msg lazily"
    else "pass"
```

No Rust recompilation needed. The engine discovers all `check_*` functions
automatically on startup. Update the `.kleis` file, restart the MCP, and
the new rule is live.

### Starting the Server

```bash
kleis review-mcp --policy examples/policies/rust_review_policy.kleis --verbose
```

In Cursor, add to `.cursor/mcp.json` (one instance per language):

```json
"kleis-review-rust": {
    "command": "/Users/you/bin/kleis",
    "args": ["review-mcp", "--verbose", "--policy",
             "/path/to/examples/policies/rust_review_policy.kleis"],
    "env": { "KLEIS_ROOT": "/path/to/kleis" }
},
"kleis-review-python": {
    "command": "/Users/you/bin/kleis",
    "args": ["review-mcp", "--verbose", "--policy",
             "/path/to/examples/policies/python_review_policy.kleis"],
    "env": { "KLEIS_ROOT": "/path/to/kleis" }
}
```

The server name is derived from the policy filename (e.g. `python_review_policy.kleis` → `kleis-review-python`). Each instance loads its own rules, advisory prompts, and structural parser.

---

## The MCPs Together

The servers cover different stages of the development workflow:

```
kleis-policy      kleis-review-rust     kleis-review-python    kleis-theory
     |                  |                     |                     |
     v                  v                     v                     v
"Can I do this?"  "Is this Rust good?"  "Is this Python good?"  "Is this math correct?"
     |                  |                     |                     |
  Action gating     Rust standards       Python standards       Theory building
  before editing     during editing       during editing        during research
```

| Server | Policy file | Convention | Tools |
|--------|-------------|------------|-------|
| kleis-policy | `agent_policy.kleis` | `check_*` returns "allow"/"deny" | 5 |
| kleis-review-rust | `rust_review_policy.kleis` | `check_*` returns "pass"/"fail: reason" (3 tiers: string, structural, Z3) | 6 |
| kleis-review-python | `python_review_policy.kleis` | `check_*` returns "pass"/"fail: reason" (string + structural via `scan_python`) | 6 |
| kleis-theory | (none — builds interactively) | `submit_*`, `evaluate`, `save_theory` | 9 |

Each review MCP is a separate process with its own policy, advisory prompt context, and structural parser. The server name is derived from the policy filename. Adding a new language is: write `<lang>_review_policy.kleis`, add an MCP entry.

All servers are subcommands of the same `kleis` binary. One build, one install,
four servers:

```bash
./scripts/build-kleis.sh
# Installs to ~/bin/kleis and ~/.cargo/bin/kleis
# All four MCPs are ready
```

### Shared Architecture

Each MCP server follows the same pattern:

1. **Load** — Parse the `.kleis` file, build an evaluator with all definitions
2. **Serve** — Listen on stdin for JSON-RPC 2.0 messages
3. **Evaluate** — For each tool call, evaluate the corresponding Kleis function
4. **Respond** — Return structured JSON results to the agent

The evaluator is built once at startup. Each tool call is a function evaluation
against the loaded definitions. No compilation, no network calls, no disk I/O
per check. This is why reviews of large files complete in milliseconds.

### CLI Review Mode (CI/CD)

The review engine is also available as a standalone CLI command, without the MCP
protocol overhead. This is suitable for GitLab CI/CD pipelines, GitHub Actions,
or pre-commit hooks:

```bash
kleis review src/**/*.rs --policy examples/policies/rust_review_policy.kleis
```

Use `--failures-only` to suppress passing rules (cleaner CI output):

```bash
kleis review src/**/*.rs -p policy.kleis --failures-only
```

The command exits with code 1 if any file fails, making it a drop-in for CI
pipelines. The kleis binary and the policy file must be available in the CI
environment — either pre-installed in the builder image or downloaded as a
release artifact.

#### GitLab CI/CD

Add a `kleis_review` job to your existing pipeline. It uses the same builder
image and resource configuration as your other Rust jobs:

```yaml
kleis_review:
  stage: mr_build
  image: $YOUR_RUST_BUILDER_IMAGE
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
      changes:
        - "src/**/*.rs"
  script:
    - |
      FAILED=0
      for f in $(find src -name '*.rs'); do
        kleis review "$f" -p policy.kleis --failures-only || FAILED=1
      done
      exit $FAILED
```

#### GitHub Actions

```yaml
name: kleis-review
on:
  pull_request:
    paths: ['src/**/*.rs']

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Download kleis
        run: |
          curl -L -o kleis https://your-artifact-store/kleis-linux
          chmod +x kleis
      - name: Run kleis review
        run: |
          FAILED=0
          for f in $(find src -name '*.rs'); do
            ./kleis review "$f" -p policy.kleis --failures-only || FAILED=1
          done
          exit $FAILED
```

### Diff-Aware Rules (`--base-branch`)

Some rules need to compare a file on the current branch against the same file
on a base branch. Version bump enforcement, changelog entry checks, and API
compatibility validation all follow this pattern.

```bash
kleis review builders/runner/config -p policy.kleis --base-branch main
```

The `--base-branch` flag accepts any git ref: branch name, remote ref (`origin/main`),
tag, or SHA. When provided, the CLI runs `diff_check_*` functions in addition to the
standard `check_*` rules.

#### Convention

| Prefix | Arguments | When it runs |
|--------|-----------|-------------|
| `check_*` | `(source)` | Always |
| `diff_check_*` | `(current, base, path)` | Only with `--base-branch` |
| `diff_file_filter` | `(path)` | Controls which files trigger `git show` |

The `diff_file_filter` function prevents unnecessary `git show` calls. Only files
where the filter returns `true` trigger a base branch fetch:

```kleis
define diff_file_filter(path) =
    if contains(path, "config") then true
    else if hasSuffix(path, "Cargo.toml") then true
    else false
```

If `diff_file_filter` is not defined, diff rules run for all files.
If `--base-branch` is absent, `diff_check_*` functions are silently skipped.
If the ref is invalid or the file is new (not on the base branch), diff rules
are skipped with a warning — the pipeline is never blocked by missing refs.

#### Example: version bump enforcement

For CI builder config files (`IMAGE_TAG=3.0.99`):

```kleis
define pick_image_tag(line, acc) =
    if hasPrefix(line, "IMAGE_TAG=") then substring(line, 10, 100)
    else acc

define diff_check_version_bump(current, base, path) =
    if not(contains(path, "config")) then "pass"
    else
        let cur_ver = foldLines(pick_image_tag, "", current) in
        let base_ver = foldLines(pick_image_tag, "", base) in
        if eq(cur_ver, base_ver) then
            concat("fail: IMAGE_TAG not bumped (still ", concat(cur_ver, ")"))
        else "pass"
```

For Rust projects (`Cargo.toml` with `version = "0.0.42"`):

```kleis
define pick_cargo_version_line(line, acc) =
    if hasPrefix(trim(line), "version = ") then trim(line) else acc

define diff_check_cargo_version(current, base, path) =
    if not(hasSuffix(path, "Cargo.toml")) then "pass"
    else
        let cur = foldLines(pick_cargo_version_line, "", current) in
        let base_line = foldLines(pick_cargo_version_line, "", base) in
        if eq(cur, base_line) then
            concat("fail: Cargo.toml version not bumped (", concat(cur, ")"))
        else "pass"
```

#### CI/CD with diff rules

In GitLab, the target branch is available as `$CI_MERGE_REQUEST_TARGET_BRANCH_NAME`:

```yaml
kleis_review:
  stage: mr_build
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
  script:
    - kleis review builders/runner/config
        -p policy.kleis
        --base-branch origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME
        --failures-only
```

In GitHub Actions, use `origin/${{ github.base_ref }}`.

For local development, run `--base-branch main` to get early warnings before
creating a merge request.

#### MCP compatibility

The `diff_check_*` prefix does not match the MCP's `check_*` discovery filter,
so diff rules are invisible to the kleis-review MCP server. They live in the
same policy file but only activate via the CLI `--base-branch` flag.

### Extra Review Files (`review_extra_files`)

Code reviews often need to inspect non-source files: `Cargo.toml` for version
bumps, `requirements.txt` for pinned dependencies, `builders/runner/config` for
image tags. These files don't match the usual `*.rs` or `*.py` globs, so they
get silently excluded.

The `review_extra_files()` convention solves this. The policy declares which
non-source files should be included:

```kleis
// Rust: version and lock files
define review_extra_files() =
    "Cargo.toml\nCargo.lock"

// Python: deps, config, CI builder
define review_extra_files() =
    "requirements.txt\nrequirements-dev.txt\nbuilders/runner/config"
```

The function returns a newline-separated string of file patterns, resolved
relative to the git repo root of the target files.

#### Discovery via MCP

When an AI agent calls `describe_standards`, the response includes the extra
files list:

```
## Extra Review Files

These non-source files should also be included in reviews:

  - Cargo.toml
  - Cargo.lock
```

The `"extra_review_files"` key also appears in the JSON schema, so agents can
programmatically discover which files to feed alongside source code.

#### CLI usage

The CLI auto-discovers extra files from the policy by default. When you run:

```bash
kleis review src/**/*.rs -p rust_review_policy.kleis --failures-only
```

the engine evaluates `review_extra_files()`, resolves each pattern relative to
the git repo root, and appends any existing files to the review list. No extra
flags needed.

To add files manually (overriding or supplementing the policy):

```bash
kleis review src/**/*.rs --include Cargo.toml --include README.md -p policy.kleis
```

To disable auto-discovery:

```bash
kleis review src/**/*.rs -p policy.kleis --include-from-policy false
```

#### Convention

| Function | Arguments | Returns | When it runs |
| --- | --- | --- | --- |
| review\_extra\_files | (none) | Newline-separated file patterns | Auto on CLI; via `describe_standards` on MCP |

If `review_extra_files` is not defined, no extra files are added. The engine
checks for its presence the same way it checks for `diff_file_filter` — by
scanning the policy's function list at startup.

### Advisory Mode (LLM-Assisted Review)

Formal checks are deterministic and machine-checked — they **block** the pipeline
on failure. But some code quality concerns are inherently ambiguous: naming
quality, architectural smell, "this function does two things." For these, Kleis
can optionally call an LLM after formal checks complete.

```bash
kleis review src/**/*.rs -p policy.kleis --advise
```

Advisory findings are printed as warnings but **never affect the exit code**.
The two-tier model:

| Tier | Engine | Verdict | Blocks CI? |
|------|--------|---------|------------|
| Formal | Kleis rules (Z3-backed) | pass/fail | Yes |
| Advisory | LLM (OpenAI-compatible) | warning/info | No |

#### Configuration

Endpoint and model are configured in `config.toml` (or via env overrides).
The API key is always an environment variable — never stored in files:

```toml
# ~/.config/kleis/config.toml
[llm]
endpoint = "https://api.openai.com/v1/chat/completions"
model = "gpt-4o-mini"
```

| Setting | config.toml | Env override | Default |
|---------|-------------|-------------|---------|
| API key | — | `KLEIS_LLM_API_KEY` (required) | — |
| Endpoint | `[llm] endpoint` | `KLEIS_LLM_ENDPOINT` | OpenAI |
| Model | `[llm] model` | `KLEIS_LLM_MODEL` | gpt-4o-mini |

Any OpenAI-compatible endpoint works: OpenAI, Azure OpenAI, Ollama
(`http://localhost:11434/v1/chat/completions`), vLLM, etc.

Example output (`kleis review src/config.rs -p policy.kleis --failures-only --advise`):

```
❌ src/config.rs
  ❌ check_structural — fail: 6 functions but no tests — consider adding test coverage
  ❌ check_no_hardcoded_urls — fail: hardcoded HTTPS URL — use configuration or constants
  ❌ check_no_unbounded_read — fail: STRIDE/DoS — unbounded read_to_string, use take() to limit input size
  ℹ️  [advisory] unnecessary-clone — Using `to_string()` can be avoided with string literals directly assigned
  ⚠️  [advisory] error-handling — Better error handling instead of silently ignoring invalid env variables
  ℹ️  [advisory] redundant-methods — Consolidating environment variable parsing methods could reduce redundancy
```

Formal verdicts (❌/✅) block the pipeline. Advisory findings (⚠️/ℹ️) inform but
never affect the exit code.

#### CI/CD with Advisory

In CI, use `--advise` alongside `--failures-only` so formal failures block the
pipeline while advisory findings appear as informational annotations:

```yaml
kleis_review:
  script:
    - kleis review src/**/*.rs -p policy.kleis --failures-only --advise
  variables:
    KLEIS_LLM_API_KEY: $OPENAI_API_KEY
```

The `--advise` flag is compiled behind the `llm-advisory` feature (enabled by
default). To build without LLM support: `cargo build --no-default-features
--features axiom-verification`.

---

-> [Previous: Interactive Theory Building](./27-theory-building.md)
