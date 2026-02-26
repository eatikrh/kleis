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
Here is a representative sample:

```kleis
// Safety
define check_no_unwrap(source) =
    if contains(source, ".unwrap()") then
        "fail: contains .unwrap() — use ? or .expect() with a message"
    else "pass"

// Error handling
define check_no_result_string(source) =
    if contains(source, "Result<(), String>") then
        "fail: Result<(), String> — use thiserror for typed errors"
    else "pass"

// Clippy patterns
define check_clippy_len_zero(source) =
    if contains(source, ".len() == 0") then
        "fail: clippy::len_zero — use .is_empty() instead"
    else if contains(source, ".len() > 0") then
        "fail: clippy::len_zero — use !.is_empty() instead"
    else "pass"

// Style
define check_no_emoji(source) =
    if contains(source, "rocket_emoji") then
        "fail: color emoji in source — use plain text"
    else "pass"
```

A full policy covers safety (unwrap, unsafe, panic), quality (println, todo,
dbg), security (hardcoded passwords, API keys, secrets), style (wildcard
imports, narrating comments, inline use statements), error handling
(Result<_, String>, clippy suppressions), and clippy `-D warnings` patterns
(ptr_arg, len_zero, bool_comparison, redundant_clone).

### Running a Review

The agent calls `check_file` with a path:

```
check_file: { "path": "src/evaluator.rs" }

-> Code Review: src/evaluator.rs — 15 passed, 10 failed (out of 25 checks)

   FAIL  check_no_unwrap — contains .unwrap() — use ? or .expect()
   FAIL  check_no_panic — contains panic!() — return Result instead
   FAIL  check_no_println — contains println!() — use eprintln! or tracing
   FAIL  check_no_unsafe — contains unsafe block
   FAIL  check_no_result_string — Result<(), String> — use thiserror
   FAIL  check_no_clippy_suppression — fix the lint instead of suppressing
   FAIL  check_clippy_ptr_arg — use &str instead of &String
   FAIL  check_no_wildcard_import — wildcard import (::*) — be explicit
   FAIL  check_no_inline_use — indented use — move imports to top of file
   FAIL  check_no_narrating_comments — code should speak for itself
   PASS  (15 other rules)
```

Or `check_code` with a snippet for in-flight review during development:

```
check_code: { "source": "fn greet(name: &String) { println!(\"hi\"); }" }

-> 2 failures:
   check_clippy_ptr_arg: use &str instead of &String
   check_no_println: use eprintln!, log macros, or tracing
```

### Real Results

We ran the policy against two Rust codebases:

**Kleis** (12 core source files):

| Metric | Value |
|--------|-------|
| Files checked | 12 |
| Passed clean | 1 |
| Total violations | 71 |
| Top violations | `.unwrap()` (8 files), `println!` (9 files), wildcard imports (9 files) |

**A sample codebase** (37 source files):

| Metric | Value |
|--------|-------|
| Files checked | 37 |
| Passed clean | 12 |
| Total violations | 70 |
| Top violations | `.unwrap()` (17 files), inline use (15 files) |
| Emoji violations | 0 |
| Clippy suppressions | 0 |

The tool immediately distinguished the two codebases by maturity. The sample
codebase had zero emoji and zero clippy suppressions — exactly the discipline
expected from a production CLI tool. Its main debt was `.unwrap()` calls,
which is common in CLI applications where panicking on bad input is acceptable.

### Adding New Rules

When a pattern bites you in production, add a rule:

```kleis
define check_no_expect_fun_call(source) =
    if contains(source, ".expect(&format!(") then
        "fail: clippy::expect_fun_call — use .unwrap_or_else or build msg lazily"
    else "pass"
```

No tooling changes needed. The engine discovers all `check_*` functions
automatically on startup. Update the file, restart the MCP, and the new rule
is live.

### Z3 Verification of Formal Properties

Beyond string-level checks, the review policy can define formal properties
using Kleis structures with axioms. These properties are verified by Z3 —
not tested against examples, but proven over all possible inputs.

The standards file can include structures like:

```kleis
structure SafeCode {
    operation is_safe : String -> Bool

    axiom safe_no_unwrap : forall(s : String).
        implies(is_safe(s), not(contains(s, ".unwrap()")))

    axiom safe_no_panic : forall(s : String).
        implies(is_safe(s), not(contains(s, "panic!(")))

    axiom safe_no_unsafe : forall(s : String).
        implies(is_safe(s), not(contains(s, "unsafe {")))
}
```

The `evaluate` tool accepts any Kleis expression, including universal
quantifiers that Z3 can verify:

```
evaluate: contains("hello world", "world")
-> true

evaluate: check_no_unwrap("fn f() { x.unwrap() }")
-> fail: contains .unwrap() — use ? or .expect() with a message

evaluate: forall(s : String). implies(is_safe(s), not(contains(s, ".unwrap()")))
-> VERIFIED
```

The first two calls are concrete evaluation — the Kleis evaluator computes
the result directly. The third is a Z3 proposition: it proves that for
**every** string `s`, if `is_safe(s)` holds, then `s` does not contain
`.unwrap()`. This is not a test case — it is a machine-checked proof.

This gives the review MCP two levels of capability:

| Level | Mechanism | Example |
|-------|-----------|---------|
| **String checks** | `check_*` functions with `contains`/`hasPrefix` | Fast pattern matching, per-file verdicts |
| **Formal properties** | Structures with axioms, verified by Z3 | Prove properties hold over all inputs |

The string checks run in microseconds and catch mechanical issues during
development. The formal properties provide guarantees about the standards
themselves — for example, proving that the safety definition is internally
consistent, or that two properties cannot conflict.

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
| kleis-review | `rust_review_policy.kleis` | `check_*` returns "pass"/"fail: reason" | 5 |
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
