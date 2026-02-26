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

### The Standards File

The standards file is a `.kleis` file where each rule is a `define` function.
The current policy includes 36 rules across nine categories:

| Category | Rules | Examples |
|----------|-------|---------|
| **Safety** | 4 | `unwrap`, `unsafe`, `panic`, `transmute` |
| **Quality** | 3 | `println`, `todo`, `dbg` |
| **Security** | 4 | hardcoded passwords, API keys, secrets, SQL injection |
| **Style** | 6 | wildcard imports, inline `use`, narrating comments, emoji, separator comments |
| **Error handling** | 2 | `Result<_, String>`, clippy suppressions |
| **Clippy `-D warnings`** | 8 | `ptr_arg`, `len_zero`, `bool_comparison`, `redundant_clone`, `single_char_pattern`, `expect_fun_call`, `manual_is_ascii`, `too_many_arguments` |
| **Deprecated patterns** | 3 | `try!()` macro, `extern crate`, hardcoded URLs |
| **Test hygiene** | 2 | `#[ignore]` tests, needless `.collect()` |
| **STRIDE threat model** | 4 | TLS bypass, credential logging, unbounded reads, command injection |

Here is a representative sample:

```kleis
// Safety
define check_no_unwrap(source) =
    if contains(source, ".unwrap()") then
        "fail: contains .unwrap() — use ? or .expect() with a message"
    else "pass"

// SQL injection detection
define check_no_sql_injection(source) =
    if and(contains(source, "format!("), contains(source, "SELECT")) then
        "fail: SQL query built with format! — use parameterized queries (sqlx::query!)"
    else if and(contains(source, ".push_str("), contains(source, "WHERE")) then
        "fail: SQL string built via push_str — use parameterized queries"
    else "pass"

// Clippy -D warnings patterns
define check_clippy_len_zero(source) =
    if contains(source, ".len() == 0") then
        "fail: clippy::len_zero — use .is_empty() instead"
    else if contains(source, ".len() > 0") then
        "fail: clippy::len_zero — use !.is_empty() instead"
    else "pass"

// Deprecated syntax
define check_no_deprecated_syntax(source) =
    if contains(source, "try!(") then
        "fail: try!() macro is deprecated — use ? operator"
    else if contains(source, "extern crate ") then
        "fail: extern crate is unnecessary since Rust 2018 — use 'use' imports"
    else "pass"

// STRIDE/DoS: unbounded reads
define check_no_unbounded_read(source) =
    if contains(source, "read_to_end(") then
        "fail: STRIDE/DoS — unbounded read_to_end, use take() or read with a size limit"
    else if contains(source, "read_to_string(") then
        "fail: STRIDE/DoS — unbounded read_to_string, use take() to limit input size"
    else "pass"
```

Rules are easy to add — the engine discovers all `check_*` functions
automatically on startup. No tooling changes needed.

### Running a Review

The agent calls `check_file` with a path:

```
check_file: { "path": "src/evaluator.rs" }

-> Code Review: src/evaluator.rs — 16 passed, 10 failed (out of 26 checks)

   FAIL  check_no_unwrap — contains .unwrap() — use ? or .expect()
   FAIL  check_no_panic — contains panic!() — return Result instead
   FAIL  check_no_println — contains println!() — use eprintln! or tracing
   FAIL  check_no_unsafe — contains unsafe block
   FAIL  check_no_result_string — Result<_, String> — use thiserror
   FAIL  check_no_clippy_suppression — fix the lint instead of suppressing
   FAIL  check_clippy_ptr_arg — use &str instead of &String
   FAIL  check_no_wildcard_import — wildcard import (::*) — be explicit
   FAIL  check_no_inline_use — indented use — move imports to top of file
   FAIL  check_no_narrating_comments — code should speak for itself
   PASS  (16 other rules)
```

Or `check_code` with a snippet for in-flight review during development:

```
check_code: { "source": "fn greet(name: &String) { println!(\"hi\"); }" }

-> 2 failures:
   check_clippy_ptr_arg: use &str instead of &String
   check_no_println: use eprintln!, log macros, or tracing
```

### Dogfooding: Reviewing Our Own Code

We ran kleis-review against the Kleis codebase itself — the AI agent
reviewing code it wrote. Results across 11 core source files:

| File | Pass | Fail | Score |
|------|------|------|-------|
| `review_mcp/protocol.rs` | 25 | 1 | 96% |
| `review_mcp/engine.rs` | 24 | 2 | 92% |
| `review_mcp/server.rs` | 21 | 5 | 81% |
| `kleis_parser.rs` | 21 | 5 | 81% |
| `axiom_verifier.rs` | 19 | 7 | 73% |
| `type_checker.rs` | 19 | 7 | 73% |
| `bin/kleis.rs` | 18 | 8 | 69% |
| `type_inference.rs` | 17 | 9 | 65% |
| `render.rs` | 17 | 9 | 65% |
| `evaluator.rs` | 16 | 10 | 62% |
| `solvers/z3/backend.rs` | 16 | 10 | 62% |

Most common violations across the codebase:

| Violation | Files Hit | Severity |
|-----------|----------|----------|
| wildcard imports `::*` | 10/11 | Medium |
| narrating comments | 9/11 | Low |
| `.unwrap()` | 8/11 | Medium |
| `println!()` | 8/11 | Medium |
| `Result<_, String>` | 8/11 | Medium |
| inline `use` | 7/11 | Low |
| color emoji | 5/11 | Low |
| `panic!()` | 3/11 | High |

The cleanest file (`protocol.rs` at 96%) was the most recently written.
The oldest, most complex files (`evaluator.rs`, `z3/backend.rs` at 62%)
carry the most debt. The tool measures what discipline cannot sustain
under pressure — and the quality trend from old to new code is visible.

### Adding New Rules

When a pattern bites you in production, add a rule:

```kleis
define check_no_transmute(source) =
    if contains(source, "transmute(") then
        "fail: std::mem::transmute — use safe conversions (From/Into, as, bytemuck)"
    else "pass"
```

Update the file, restart the MCP, and the new rule is live.

### Z3 Verification of Formal Properties

Beyond string-level checks, the review policy defines formal properties
using Kleis structures with axioms. These properties are verified by Z3 —
not tested against examples, but proven over all possible inputs.

**Structures define what "safe" or "clean" means formally:**

```kleis
structure SafeCode {
    operation is_safe : String -> Bool

    axiom safe_no_unwrap : ∀(s : String).
        implies(is_safe(s), not(contains(s, ".unwrap()")))

    axiom safe_no_panic : ∀(s : String).
        implies(is_safe(s), not(contains(s, "panic!(")))

    axiom safe_no_unsafe : ∀(s : String).
        implies(is_safe(s), not(contains(s, "unsafe {")))

    axiom safe_complete : ∀(s : String).
        implies(
            and(not(contains(s, ".unwrap()")),
                and(not(contains(s, "panic!(")),
                    not(contains(s, "unsafe {")))),
            is_safe(s))
}
```

Each structure has both *necessary conditions* (if safe, then no unwrap) and
a *completeness axiom* (if no unwrap and no panic and no unsafe, then safe).
The completeness axiom lets Z3 prove positive properties — that a piece of
code *is* safe — not just disprove negative ones.

**The `evaluate` tool operates at three levels:**

```
// Level 1: Concrete evaluation
evaluate: check_no_unwrap("fn f() { x.unwrap() }")
-> fail: contains .unwrap() — use ? or .expect() with a message

// Level 2: Universal proof
evaluate: ∀(s : String). implies(is_safe(s), not(contains(s, ".unwrap()")))
-> VERIFIED

// Level 3: Concrete Z3 verification on actual code
evaluate: is_safe("fn process(x: Option<i32>) -> Result<i32> { Ok(x?) }")
-> Verified: true
```

Level 1 is fast string matching. Level 2 proves properties hold over all
possible inputs — a machine-checked proof, not a test case. Level 3 is
where the tool becomes a genuine code reviewer: Z3 checks whether a
*specific* code snippet satisfies the formal definition of safety.

Level 3 uses **ground instantiation** — instead of asking Z3 to reason about
universal quantifiers over the infinite space of strings, the evaluator
substitutes the concrete code snippet directly into each axiom. This turns
quantified formulas into quantifier-free assertions that Z3 solves in
milliseconds.

### SQL Injection Analysis via Z3

The policy includes a `SqlSafe` structure that models taint analysis
axiomatically:

```kleis
structure SqlSafe {
    operation is_tainted : String -> Bool
    operation is_sanitized : String -> Bool
    operation reaches_query : String -> Bool

    axiom no_tainted_query : ∀(s : String).
        implies(and(is_tainted(s), reaches_query(s)), is_sanitized(s))

    axiom format_select_is_tainted : ∀(s : String).
        implies(
            and(contains(s, "format!("), contains(s, "SELECT")),
            and(is_tainted(s), not(is_sanitized(s))))

    axiom parameterized_is_sanitized : ∀(s : String).
        implies(contains(s, "sqlx::query!"), is_sanitized(s))

    axiom placeholder_is_sanitized : ∀(s : String).
        implies(contains(s, "$1"), is_sanitized(s))

    // ... additional axioms for INSERT, UPDATE, DELETE, DROP, TRUNCATE
}
```

The axioms encode domain knowledge: `format!` with SQL keywords is tainted,
parameterized queries (`sqlx::query!`, `$1`, `?`) are sanitized, and
destructive operations (`DROP`, `TRUNCATE`) are always tainted. Z3 can then
verify concrete code:

```
evaluate: is_tainted("let q = format!(\"SELECT * FROM users WHERE id = {}\", input);")
-> Verified: true

evaluate: is_sanitized("sqlx::query!(\"SELECT * FROM users WHERE id = $1\", id)")
-> Verified: true

evaluate: is_tainted("let q = format!(\"DROP TABLE {}\", name);")
-> Verified: true
```

This is not string matching — it is logical deduction from axioms. If a code
snippet contains `format!` with `SELECT` but no parameterized placeholder,
Z3 deduces it is tainted and not sanitized. The axioms compose: new SQL
patterns can be added without changing the engine.

### Three Levels of Review

| Level | Mechanism | Speed | Example |
|-------|-----------|-------|---------|
| **String checks** | `check_*` functions with `contains`/`hasPrefix` | Microseconds | Fast pattern matching, per-file verdicts |
| **Universal proofs** | Structures with axioms, verified by Z3 | Milliseconds | Prove properties hold over all inputs |
| **Concrete Z3** | Ground instantiation of axioms on real code | Milliseconds | Verify `is_safe(actual_code)` or `is_tainted(sql_snippet)` |

The string checks catch mechanical issues during development. The universal
proofs guarantee the standards themselves are consistent. The concrete Z3
checks apply formal definitions to actual code — the reviewer moves from
"does this string contain `.unwrap()`" to "does this code satisfy the formal
definition of safety."

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
| kleis-review | `rust_review_policy.kleis` | `check_*` returns "pass"/"fail: reason" | 6 |
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
