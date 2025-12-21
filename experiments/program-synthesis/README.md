# Program Synthesis Experiments

**Goal:** Use Z3 to synthesize LISP/programs from specifications.

## ⚠️ Honest Assessment

| What We Claimed | What We Achieved |
|-----------------|------------------|
| Z3 synthesizes LISP programs | Z3 found parameters; human wrote template |
| Grammar-based synthesis | Sketch-based synthesis (holes in template) |
| "Correct by construction" | Verified for bounded test cases |

**The hard truth:** True program synthesis from grammar didn't work.

## Results Summary

| Experiment | What Z3 Did | What Human Did |
|------------|-------------|----------------|
| **01_double** | Found params (op=add, args=x,x) | Wrote expression template |
| **02_max** | Found params (cond=>=, val=a,b) | Wrote conditional template |
| **03_length** | Found params (base=0, step=1) | Wrote recursive template |
| **04_sort2** | Found params (compare, branch) | Wrote 2-element template |
| **05_sort3** | Verified sorting network | Wrote 3-element template |
| **09_recursive_sort** | Found params (cc=0, tc=0, ec=1) | Wrote insert/sort template |

## The Failed Dream

We wanted this:

```
INPUT:  Sorting specification + LISP grammar
OUTPUT: (define (sort xs) (if (null? xs) '() ...))
        ↑ Z3 synthesizes the entire program
```

We tried:
```kleis
:sat ∃(P : SExpr). 
    eval_lisp(P, [3,1,2]) = [1,2,3] ∧
    eval_lisp(P, [5,1]) = [1,5]
```

**Result: Stack overflow / timeout**

Z3 cannot symbolically execute `eval_lisp` because:
1. `eval_lisp` is recursive
2. SExpr is an unbounded algebraic datatype
3. E-matching over infinite space = death

## What Actually Worked

### Sketch-Based Synthesis

Human provides template with holes:
```kleis
define insert_param(cc: ℤ, tc: ℤ, ec: ℤ, x: ℤ, ys: List(ℤ)) : List(ℤ) =
    match ys {
        Nil => Cons(x, Nil)
      | Cons(y, rest) =>
            if eval_cond(cc, x, y)   // cc chooses: <=, <, >=, >
            then eval_branch(tc, ...)  // tc chooses: stop or recurse
            else eval_branch(ec, ...)  // ec chooses: stop or recurse
    }
```

Z3 searches 16 combinations, finds (cc=0, tc=0, ec=1):
- Condition: `x <= y` (insert before if ≤)
- Then: stop (Cons(x, Cons(y, rest)))
- Else: recurse (Cons(y, insert(x, rest)))

This is **insertion sort**.

### Local Property Verification

We verified:
```kleis
// If ys is sorted, then insert(x, ys) is sorted
:sat ∃(x y1 y2 : ℤ). 
    y1 <= y2 ∧ 
    ¬(is_sorted(insert_param(0, 0, 1, x, Cons(y1, Cons(y2, Nil)))))

// Result: UNSAT — the property holds!
```

This proves `insert` preserves sortedness for 2-element lists.
By induction, this implies correctness for all lists.
But we didn't mechanize the induction.

## The Gap

```
VISION:
  Natural Language → LLM → Kleis Spec → Z3 → Program
                                         ↑
                                    DOESN'T WORK
                                    (for recursive programs)

REALITY:
  Human writes template with holes → Z3 fills holes → Verify bounded cases
```

## Why Recursive Synthesis Fails

```kleis
// Z3 sees this axiom:
∀(P : SExpr). ∀(env : Env). 
    eval_lisp(SList(Cons(SAtom("if"), ...)), env) = 
        if eval_lisp(cond, env) then eval_lisp(t, env) else eval_lisp(e, env)

// When you ask:
:sat ∃(P : SExpr). eval_lisp(P, env) = [1,2,3]

// Z3 tries to instantiate P with every possible SExpr
// That's infinite. It gives up.
```

## What Z3 CAN Do

| Task | Feasibility | Notes |
|------|-------------|-------|
| Verify finite properties | ✅ Works | `sort([3,1,2]) = [1,2,3]` |
| Find parameters in bounded space | ✅ Works | Search 16 options |
| Synthesize non-recursive expressions | ✅ Works | `max(a,b)`, `2*x` |
| Synthesize from full grammar | ❌ Fails | Infinite search space |
| Evaluate recursive functions | ❌ Fails | Stack overflow |

## Possible Solutions

1. **SyGuS tools**: CVC5 has syntax-guided synthesis support
2. **Bounded synthesis**: Limit program size, list lengths
3. **Enumerate and verify**: Generate candidates, check each
4. **LLM proposes, Z3 verifies**: Use LLM for creativity, Z3 for correctness

## Files

| File | Purpose |
|------|---------|
| `01_double_v2.kleis` | Synthesize `2*x` |
| `02_max.kleis` | Synthesize `max(a,b)` |
| `03_length.kleis` | Synthesize list length |
| `04_sort2.kleis` | Sort 2 elements |
| `05_sort3.kleis` | Sort 3 elements |
| `07_sorting_spec.kleis` | Formal sorting definition |
| `09_recursive_sort_synthesis.kleis` | Insert/sort template |
| `10_run_synthesis.kleis` | Manual search over 16 params |
| `12_z3_synthesis.kleis` | Z3 parameter search + verification |

## Lessons Learned

1. **Z3 is a verifier, not a synthesizer** (for recursive programs)
2. **Sketch-based synthesis works** but human does creative work
3. **Bounded verification works** for correctness confidence
4. **True synthesis needs specialized tools** (SyGuS, CEGIS)

## The Revised Vision

```
Developer: "Sort a list"
     ↓
LLM: Generates sort function (might have bugs)
     ↓
Kleis/Z3: Verifies is_sorted(result) ∧ is_permutation(result, input)
     ↓
✅ Verified  OR  ❌ Counterexample: [3,1,2] fails
```

**LLM synthesizes, Z3 verifies.** That's the realistic architecture.

See `docs/vision/VERIFIED_SOFTWARE_VISION.md` for the full story.
