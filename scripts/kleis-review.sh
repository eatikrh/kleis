#!/bin/bash
#
# Kleis Code Review — 5-angle Claude Code review customized for the Kleis
# codebase (Rust CLI / language runtime / formal verification platform).
#
# Adapted from user-service/scripts/claude-review.sh (hackathon 5-angle
# approach) with review criteria rewritten for Kleis's architecture.
#
# Usage:
#   ./scripts/kleis-review.sh                  # review current branch diff against main
#   ./scripts/kleis-review.sh --file path.rs   # review a single file
#   ./scripts/kleis-review.sh --base develop   # override base branch
#
# Requires: claude CLI (Claude Code) in PATH.

set -euo pipefail

BASE_BRANCH="${BASE_BRANCH:-main}"

DIFF_PROMPT='Review this diff for the Kleis project — a Rust CLI and language runtime for formal verification (Z3 backend, Typst rendering, LSP, DAP debugger, Jupyter kernel).

Check from these 5 angles:

1. **Grammar–Parser consistency** — Does the change match the formal EBNF grammar (docs/grammar/kleis_grammar_v098.ebnf)? Parser changes must align with the grammar spec. New syntax must be documented in the grammar. AST variants must faithfully represent the parsed input.

2. **Span and debug integrity** — Does the change preserve Expression spans correctly? The span is the source of truth for file/line/column. Substitution must NOT modify spans. New Expression constructors must carry spans. Debug hooks (DAP) rely on spans being accurate.

3. **Z3 soundness** — Does the change maintain correct translation between Kleis expressions and Z3 sorts/values? Type inference (Hindley-Milner, ADR-014) must use constraint-based unification, not hardcoded results. Operations must be defined in stdlib structures, not hardcoded in Rust (ADR-016). Rational number conversion must be exact (no floating-point truncation in Z3 Real).

4. **Evaluation correctness** — Does eval_concrete produce correct results? Does eval_numeric handle the new code path? Are edge cases covered (zero, negative, very large/small values)? Does the pretty printer round-trip correctly?

5. **Security and quality** — Hardcoded secrets, unsafe blocks without SAFETY comments, command injection via Command::new + format!, credential logging. For Kleis specifically: eprintln! behind KLEIS_Z3_DEBUG is acceptable (CLI diagnostic output, not a bug). #[allow(dead_code)] on complex number scaffolding is acceptable (planned infrastructure).

For each issue found, rate your confidence 0-100. Only report issues you rate 80+.

Ignore these (treat as false positives):
- Pre-existing issues (not introduced by this change)
- Pedantic nitpicks a senior engineer would not flag
- Things cargo clippy, rustfmt, or the compiler would catch
- eprintln! in production code guarded by if z3_debug {} (intentional CLI diagnostics)
- #[allow(dead_code)] on Complex number Z3 ADT methods (planned infrastructure)
- Decorative separator comments in test modules (organizational, not narrating)
- Result<_, String> error types (known tech debt, codebase-wide refactor needed)

Output format:
- If no issues found: "PASS: No critical issues found."
- Otherwise, list each issue briefly with file, line context, confidence score, and which of the 5 angles flagged it.

Be brief and precise.'

FILE_PROMPT='Review this Rust source file from the Kleis project — a Rust CLI and language runtime for formal verification (Z3 backend, Typst rendering, LSP, DAP debugger, Jupyter kernel).

Check from these 5 angles:

1. **Grammar–Parser consistency** — Does the code match the formal EBNF grammar? Parser functions must implement what the grammar specifies. AST variants must faithfully represent the parsed input.

2. **Span and debug integrity** — Are Expression spans preserved correctly? New Expression constructors must carry spans. The span is the sole source of truth for DAP file/line/column.

3. **Z3 soundness** — Is Kleis-to-Z3 translation correct? Type inference follows HM (ADR-014)? Operations defined in stdlib, not hardcoded (ADR-016)? Rational conversion exact?

4. **Evaluation correctness** — Does eval_concrete produce correct results? Edge cases handled? Pretty printer round-trips correctly?

5. **Security and quality** — Hardcoded secrets, undocumented unsafe, command injection, credential logging. eprintln! behind z3_debug is acceptable. Complex number scaffolding #[allow(dead_code)] is acceptable.

For each issue found, rate your confidence 0-100. Only report issues you rate 80+.

Ignore these (treat as false positives):
- Pedantic nitpicks a senior engineer would not flag
- Things cargo clippy, rustfmt, or the compiler would catch
- eprintln! guarded by z3_debug (intentional CLI diagnostics)
- #[allow(dead_code)] on Complex Z3 ADT methods (planned)
- Separator comments in test modules
- Result<_, String> error types (known tech debt)

Output format:
- If no issues found: "PASS: No critical issues found."
- Otherwise, list each issue briefly with file, line context, confidence score, and which of the 5 angles flagged it.

Be brief and precise.'

usage() {
    echo "Usage: $0 [--file <path>] [--base <branch>]"
    echo ""
    echo "  (no args)       Review current branch diff against $BASE_BRANCH"
    echo "  --file <path>   Review a single file"
    echo "  --base <branch> Override base branch (default: main)"
    exit 1
}

MODE="diff"
FILE_PATH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --file)  MODE="file"; FILE_PATH="$2";  shift 2 ;;
        --base)  BASE_BRANCH="$2";             shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

echo "=========================================="
echo "  Kleis Code Review (5-angle Claude)"
echo "=========================================="
echo ""

case $MODE in
    diff)
        CHANGED_FILES=$(git diff --name-only "origin/$BASE_BRANCH...HEAD" 2>/dev/null)
        if [ -z "$CHANGED_FILES" ]; then
            echo "No changes to review against $BASE_BRANCH."
            exit 0
        fi
        echo "Reviewing branch diff against $BASE_BRANCH..."
        echo "Files changed:"
        echo "$CHANGED_FILES" | sed 's/^/  - /'
        echo ""
        INPUT=$(git diff "origin/$BASE_BRANCH...HEAD")
        PROMPT="$DIFF_PROMPT"
        ;;
    file)
        echo "Reviewing file: $FILE_PATH"
        echo ""
        INPUT=$(cat "$FILE_PATH")
        PROMPT="$FILE_PROMPT"
        ;;
esac

CLAUDE_CMD="claude"
if command -v claude.sh &>/dev/null; then
    CLAUDE_CMD="claude.sh"
fi

REVIEW=$(echo "$INPUT" | $CLAUDE_CMD -p "$PROMPT" 2>&1)

echo "$REVIEW"
echo ""
echo "=========================================="

if echo "$REVIEW" | grep -qi "PASS:.*No critical issues"; then
    echo "Review passed."
else
    echo "Review found potential issues."
fi

echo ""
