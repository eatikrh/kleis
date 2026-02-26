# Agent MCP Servers

Kleis ships three Model Context Protocol (MCP) servers that turn Kleis policies
and structures into live tools for AI agents. The previous chapter covered
**kleis-theory** — the interactive theory-building server. This chapter covers
the other two: **kleis-policy** for agent action gating, and **kleis-review**
for code review against formal coding standards.

All three share the same architecture: a Kleis `.kleis` file defines the rules,
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
Tier 2 parses the source into a `Crate` AST using a structural parser written
in Kleis itself (`rust_parser.kleis`), then queries the AST for functions,
structs, use declarations, and comments. This enables context-aware analysis:
`.unwrap()` in a `#[test]` function is fine; `.unwrap()` in production code
is flagged. Tier 3 uses Z3 to prove properties hold universally.

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
| `check_secure_structural` | Hardcoded `password =`, `secret =`, `api_key =` — skips test code |
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

In Cursor, add to `.cursor/mcp.json`:

```json
"kleis-review": {
    "command": "/Users/you/bin/kleis",
    "args": ["review-mcp", "--verbose", "--policy",
             "/path/to/examples/policies/rust_review_policy.kleis"]
}
```

---

## The Three MCPs Together

The three servers cover different stages of the development workflow:

```
kleis-policy          kleis-review          kleis-theory
     |                     |                     |
     v                     v                     v
"Can I do this?"    "Is this code good?"   "Is this math correct?"
     |                     |                     |
  Action gating       Code standards       Theory building
  before editing       during editing       during research
```

| Server | Policy file | Convention | Tools |
|--------|-------------|------------|-------|
| kleis-policy | `agent_policy.kleis` | `check_*` returns "allow"/"deny" | 5 |
| kleis-review | `rust_review_policy.kleis` | `check_*` returns "pass"/"fail: reason" (3 tiers: string, structural, Z3) | 6 |
| kleis-theory | (none — builds interactively) | `submit_*`, `evaluate`, `save_theory` | 9 |

All three are subcommands of the same `kleis` binary. One build, one install,
three servers:

```bash
./scripts/build-kleis.sh
# Installs to ~/bin/kleis and ~/.cargo/bin/kleis
# All three MCPs are ready
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

---

-> [Previous: Interactive Theory Building](./27-theory-building.md)
