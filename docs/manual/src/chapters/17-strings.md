# Strings

Kleis provides comprehensive support for **string operations** via Z3's QF_SLIA (Quantifier-Free Strings and Linear Integer Arithmetic) theory. This enables formal verification of string-manipulating programs.

## The String Type

```kleis
define greeting : String = "Hello, World!"
define empty : String = ""
```

## Basic Operations

### Concatenation

```kleis
structure StringConcat {
    // Concatenation
    axiom concat_ex : concat("Hello", " World") = "Hello World"
    
    // Empty string is identity
    axiom concat_empty_left : ∀(s : String). concat("", s) = s
    axiom concat_empty_right : ∀(s : String). concat(s, "") = s
    
    // Associativity
    axiom concat_assoc : ∀(a b c : String). 
        concat(concat(a, b), c) = concat(a, concat(b, c))
}
```

### Length

```kleis
structure StringLength {
    axiom len_hello : strlen("Hello") = 5
    axiom len_empty : strlen("") = 0
    
    // Length of concatenation
    axiom len_concat : ∀(a b : String). 
        strlen(concat(a, b)) = strlen(a) + strlen(b)
    
    // Length is non-negative
    axiom len_nonneg : ∀(s : String). strlen(s) ≥ 0
}
```

## Substring Operations

### Contains

Check if one string contains another:

```kleis
structure StringContains {
    axiom contains_ex : contains("Hello World", "World") = true
    axiom contains_empty : ∀(s : String). contains(s, "") = true
    axiom contains_self : ∀(s : String). contains(s, s) = true
}
```

### Prefix and Suffix

```kleis
structure PrefixSuffix {
    // Prefix check
    axiom prefix_ex : hasPrefix("Hello World", "Hello") = true
    axiom prefix_empty : ∀(s : String). hasPrefix(s, "") = true
    
    // Suffix check
    axiom suffix_ex : hasSuffix("Hello World", "World") = true
    axiom suffix_empty : ∀(s : String). hasSuffix(s, "") = true
}
```

### Substring Extraction

```kleis
structure Substring {
    // substr(s, start, length) extracts substring
    axiom substr_ex : substr("Hello World", 0, 5) = "Hello"
    axiom substr_middle : substr("Hello World", 6, 5) = "World"
    
    // Empty substring
    axiom substr_zero : ∀(s : String)(i : ℕ). substr(s, i, 0) = ""
}
```

### Character Access

```kleis
structure CharAt {
    // charAt(s, i) returns single character at index i
    axiom charAt_ex : charAt("Hello", 0) = "H"
    axiom charAt_last : charAt("Hello", 4) = "o"
}
```

### Index Of

```kleis
structure IndexOf {
    // indexOf(s, pattern, start) returns first index of pattern
    axiom indexOf_ex : indexOf("Hello World", "o", 0) = 4
    axiom indexOf_second : indexOf("Hello World", "o", 5) = 7
    
    // Not found returns -1
    axiom indexOf_notfound : indexOf("Hello", "z", 0) = 0 - 1
}
```

### Replace

```kleis
structure StringReplace {
    // replace(s, old, new) replaces first occurrence
    axiom replace_ex : replace("Hello World", "World", "Kleis") = "Hello Kleis"
    
    // No match means no change
    axiom replace_nomatch : ∀(s : String). 
        ¬contains(s, "xyz") → replace(s, "xyz", "abc") = s
}
```

## String-Integer Conversion

### String to Integer

```kleis
structure StrToInt {
    axiom str_to_int_ex : strToInt("42") = 42
    axiom str_to_int_neg : strToInt("-17") = 0 - 17
    axiom str_to_int_zero : strToInt("0") = 0
}
```

### Integer to String

```kleis
structure IntToStr {
    axiom int_to_str_ex : intToStr(42) = "42"
    axiom int_to_str_neg : intToStr(0 - 17) = "-17"
    axiom int_to_str_zero : intToStr(0) = "0"
}
```

### Round-trip Property

```kleis
structure Roundtrip {
    // Converting back and forth preserves value
    axiom roundtrip_int : ∀(n : ℤ). strToInt(intToStr(n)) = n
    
    // For valid numeric strings
    axiom roundtrip_str : ∀(s : String). 
        isDigits(s) → intToStr(strToInt(s)) = s
}
```

## Regular Expressions

Kleis exposes Z3's native regular expression theory, giving you **composable regex constructors** that Z3 can reason about formally. Unlike traditional regex engines that just match strings, Kleis regexes are first-class Z3 objects — you can build them, combine them, and then **prove properties** about strings that match (or don't match) them.

### Regex Constructors

Build regular expressions compositionally using these operations:

| Constructor | Description | Equivalent Regex |
|-------------|-------------|-----------------|
| `re_literal("foo")` | Match exact string | `foo` |
| `re_range("a", "z")` | Character class | `[a-z]` |
| `re_star(re)` | Zero or more | `re*` |
| `re_plus(re)` | One or more | `re+` |
| `re_option(re)` | Zero or one | `re?` |
| `re_concat(re1, re2)` | Sequence | `re1re2` |
| `re_union(re1, re2)` | Alternation | `re1\|re2` |
| `re_intersect(re1, re2)` | Intersection | Both must match |
| `re_complement(re)` | Negation | Anything `re` doesn't match |
| `re_full()` | Match any string | `.*` |
| `re_empty()` | Match nothing | (empty language) |
| `re_allchar()` | Any single character | `.` |
| `re_loop(re, lo, hi)` | Bounded repetition | `re{lo,hi}` |

### Matching

Use `matches(s, re)` to test whether string `s` matches regex `re`:

```kleis
structure RegexExamples {
    // Composable regex: one or more lowercase letters
    axiom lower_word : matches("hello", re_plus(re_range("a", "z"))) = true
    
    // Sequence: "foo" followed by digits
    axiom foo_digits : matches("foo42", re_concat(
        re_literal("foo"),
        re_plus(re_range("0", "9"))
    )) = true
    
    // Alternation: "yes" or "no"
    axiom yes_no : matches("yes", re_union(
        re_literal("yes"),
        re_literal("no")
    )) = true
    
    // Complement: anything that's NOT all digits
    axiom not_digits : matches("abc", re_complement(
        re_plus(re_range("0", "9"))
    )) = true
}
```

### Convenience Predicates

For common patterns, Kleis provides built-in predicates that combine regex constructors internally:

```kleis
structure RegexPredicates {
    // Character class checks
    axiom digits : isDigits("12345") = true
    axiom alpha : isAlpha("Hello") = true
    axiom alphanum : isAlphaNum("Test123") = true
    
    // ASCII printable: every character in range ' ' (0x20) to '~' (0x7E)
    axiom ascii_ok : isAscii("Hello, World! 42 + 7 = 49") = true
    axiom ascii_fail : isAscii("Hello 🌍") = false
}
```

| Predicate | Pattern | Description |
|-----------|---------|-------------|
| `isDigits(s)` | `[0-9]*` | Only digits |
| `isAlpha(s)` | `[a-zA-Z]*` | Only letters |
| `isAlphaNum(s)` | `[a-zA-Z0-9]*` | Letters and digits |
| `isAscii(s)` | `[ -~]*` | ASCII printable characters |

### Formal Verification with Regexes

Because regexes are Z3 objects, you can **prove** properties about them:

```kleis
structure RegexProofs {
    // If a string is all digits, it's also alphanumeric
    axiom digits_are_alphanum : ∀(s : String).
        isDigits(s) → isAlphaNum(s)
    
    // If a string is ASCII, it matches the printable range
    axiom ascii_is_printable : ∀(s : String).
        isAscii(s) → matches(s, re_star(re_range(" ", "~")))
    
    // Intersection of [a-z]+ and [A-Z]+ is empty (no string matches both)
    axiom disjoint_case : ∀(s : String). ¬matches(s, re_intersect(
        re_plus(re_range("a", "z")),
        re_plus(re_range("A", "Z"))
    ))
}
```

### Practical Example: Policy Enforcement

The Kleis MCP agent policy uses `isAscii` to enforce clean commit messages:

```kleis
// Commit messages must be ASCII-only (no emojis)
define check_git_commit(description) =
    if isAscii(description) then "allow"
    else "deny"
```

Z3 can then verify that **no emoji can ever slip through**:

```kleis
// This proposition is VERIFIED by Z3:
// ∀(d : String). implies(check_git_commit(d) = "allow", isAscii(d))
```

## Z3 Verification

String properties are verified using Z3's native string theory:

```kleis
structure Z3StringProofs {
    // Concatenation properties
    axiom concat_length : ∀(a b : String). 
        strlen(concat(a, b)) = strlen(a) + strlen(b)
    
    // Contains implies length relationship
    axiom contains_length : ∀(s t : String). 
        contains(s, t) → strlen(s) ≥ strlen(t)
    
    // Prefix implies contains
    axiom prefix_contains : ∀(s t : String). 
        hasPrefix(s, t) → contains(s, t)
}
```

## Monoid Structure

Strings form a **monoid** under concatenation:

```kleis
implements Monoid(String) {
    operation identity = ""
    operation mul = concat
}

// Monoid laws hold:
// 1. concat("", s) = s           (left identity)
// 2. concat(s, "") = s           (right identity)
// 3. concat(a, concat(b, c)) 
//    = concat(concat(a, b), c)   (associativity)
```

## Practical Examples

### Email Validation

```kleis
structure EmailValidation {
    define isValidEmail(email : String) : Bool =
        contains(email, "@") ∧ 
        contains(email, ".") ∧
        indexOf(email, "@", 0) < indexOf(email, ".", 0)
    
    axiom valid_ex : isValidEmail("user@example.com") = true
    axiom invalid_ex : isValidEmail("invalid") = false
}
```

### URL Parsing

```kleis
structure URLParsing {
    define getProtocol(url : String) : String =
        substr(url, 0, indexOf(url, "://", 0))
    
    axiom http_ex : getProtocol("https://kleis.io") = "https"
}
```

### String Builder Pattern

```kleis
structure StringBuilder {
    define join(sep : String, a : String, b : String) : String =
        concat(concat(a, sep), b)
    
    axiom join_ex : join(", ", "Hello", "World") = "Hello, World"
}
```

## Operation Reference

### String Operations

| Operation | Syntax | Description |
|-----------|--------|-------------|
| Concatenate | `concat(a, b)` | Join two strings |
| Length | `strlen(s)` | Character count |
| Contains | `contains(s, t)` | Check substring |
| Prefix | `hasPrefix(s, t)` | Check prefix |
| Suffix | `hasSuffix(s, t)` | Check suffix |
| Substring | `substr(s, i, n)` | Extract n chars from i |
| Character | `charAt(s, i)` | Get char at index |
| Index | `indexOf(s, t, i)` | Find substring from i |
| Replace | `replace(s, old, new)` | Replace first match |
| To Int | `strToInt(s)` | Parse integer |
| From Int | `intToStr(n)` | Format integer |

### Regex Constructors

| Operation | Syntax | Description |
|-----------|--------|-------------|
| Literal | `re_literal(s)` | Exact string match |
| Range | `re_range(lo, hi)` | Character class `[lo-hi]` |
| Star | `re_star(re)` | Zero or more |
| Plus | `re_plus(re)` | One or more |
| Option | `re_option(re)` | Zero or one |
| Concat | `re_concat(re1, re2)` | Sequence |
| Union | `re_union(re1, re2)` | Alternation |
| Intersect | `re_intersect(re1, re2)` | Both must match |
| Complement | `re_complement(re)` | Negation |
| Full | `re_full()` | Any string |
| Empty | `re_empty()` | No string |
| Any char | `re_allchar()` | Single character |
| Loop | `re_loop(re, lo, hi)` | Bounded repetition |

### Regex Matching & Predicates

| Operation | Syntax | Description |
|-----------|--------|-------------|
| Match | `matches(s, re)` | String matches regex |
| Digits | `isDigits(s)` | All `[0-9]` |
| Alpha | `isAlpha(s)` | All `[a-zA-Z]` |
| AlphaNum | `isAlphaNum(s)` | All `[a-zA-Z0-9]` |
| ASCII | `isAscii(s)` | All printable ASCII `[ -~]` |

## Why Strings Don't Need Kleis Axioms

You may notice that `stdlib/text.kleis` declares string axioms like:

```kleis
structure TextOps {
    axiom concat_assoc : ∀ a : String . ∀ b : String . ∀ c : String .
        equals(concat(concat(a, b), c), concat(a, concat(b, c)))

    axiom hasPrefix_empty : ∀ s : String .
        equals(hasPrefix(s, ""), True)
}
```

Yet when the MCP agent sends a proposition like `∀(s : String). implies(isDigits(s), isAlphaNum(s))` to Z3, **none of these axioms are loaded**. The MCP policy engine only loads the policy file — not stdlib. And it still works. Why?

### Z3's Built-in String Theory

Z3 has a **native string theory** (QF_SLIA — Quantifier-Free Strings and Linear Integer Arithmetic). It already knows the semantics of concatenation, length, prefix, suffix, contains, regex matching, and all the operations listed in this chapter. The Kleis Z3 backend translates operations directly to Z3's API:

```
Kleis expression          →  Z3 native operation
─────────────────────────────────────────────────
contains(s, "foo")        →  z3::ast::String::contains()
hasPrefix(s, "bar")       →  z3::ast::String::prefix()
isDigits(s)               →  regex_matches(s, re_plus(re_range("0","9")))
matches(s, re_star(re))   →  z3::ast::String::regex_matches(Regexp::star())
```

Z3 doesn't learn that `concat("", s) = s` from a Kleis axiom — it already knows this as a built-in theorem of its string theory. The axiom in `stdlib/text.kleis` is **redundant for Z3** but serves other purposes.

### Native Theories vs. Uninterpreted Functions

This is the fundamental architectural distinction in Kleis's solver integration:

| Domain | Z3 Knowledge | Needs Kleis Axioms? | Example |
|--------|-------------|---------------------|---------|
| **Strings** | Built-in native theory | ❌ No | `contains`, `hasPrefix`, `re_star` |
| **Integers** | Built-in native theory | ❌ No | `+`, `*`, `<`, `≥` |
| **Booleans** | Built-in native theory | ❌ No | `and`, `or`, `not`, `implies` |
| **Bitvectors** | Built-in native theory | ❌ No | `bvand`, `bvxor`, `bvshl` |
| **Tensors** | No built-in knowledge | ✅ Yes | `contract`, `trace`, `wedge` |
| **Groups** | No built-in knowledge | ✅ Yes | `•`, `inv`, `e` |
| **Manifolds** | No built-in knowledge | ✅ Yes | `christoffel`, `riemann` |

When you write `axiom commutativity : ∀(x y : G). x • y = y • x` in a Group structure, that axiom is **essential** — Z3 has no built-in theory of abstract algebra. Without it, Z3 treats `•` as an uninterpreted function and can't prove anything about it. The axiom teaches Z3 the rule.

But when you write `axiom concat_empty_right : ∀(s : String). concat(s, "") = s`, you're stating something Z3 already knows. The axiom is a **Kleis-level declaration**, not a Z3-level requirement.

### A Note on What Axioms Are

In general, axioms are **not** documentation — they are the primary mechanism by which Z3 learns the rules of a domain. When you write:

```kleis
structure AbelianGroup(G) {
    axiom commutativity : ∀(x y : G). x • y = y • x
}
```

This is the **only way** Z3 learns that `•` is commutative. Without it, `•` is an uninterpreted function — Z3 can't prove anything about it. Axioms are the substance of formal reasoning, not a comment beside it.

Strings are a **special case**. Because Z3 has a built-in string theory, the axioms in `stdlib/text.kleis` happen to restate what Z3 already knows. They still serve purposes, but different ones than axioms in other domains:

1. **Type checking** — The `structure TextOps` declares operations with type signatures (`operation concat : String → String → String`). The type checker uses these to verify that string operations are used correctly in `.kleis` files.

2. **Machine-checked specification** — Kleis can verify its own string axioms against Z3 (`cargo test`). If someone writes an incorrect axiom, Z3 will disprove it. The stdlib is a formal contract that the implementation must satisfy.

3. **Portability** — If a future solver backend doesn't have a native string theory, it would need these axioms to reason about strings. The axioms make the semantics explicit and backend-independent.

4. **Uniformity** — Every domain in Kleis follows the same pattern: structures declare operations, axioms declare their properties. String axioms maintain this pattern even when Z3 doesn't strictly need them.

### When You Would Need Axioms

Custom string predicates that aren't hardcoded in the Z3 backend **do** need axioms. For example:

```kleis
// Z3 backend knows isDigits natively — no axiom needed for verification
isDigits("123")  // → Z3 translates to regex_matches(s, re_plus(re_range("0","9")))

// But a custom predicate is opaque to Z3 without a definition
define isHexColor(s) = and(
    strlen(s) = 7,
    and(charAt(s, 0) = "#",
        matches(substr(s, 1, 6), re_plus(
            re_union(re_range("0", "9"),
            re_union(re_range("a", "f"), re_range("A", "F")))
        )))
)
```

Here `isHexColor` is defined in terms of operations Z3 already knows, so Z3 can reason about it by expanding the definition. But if you declared `isHexColor` as an uninterpreted function (no body), Z3 would need axioms:

```kleis
structure HexColors {
    operation isHexColor : String → Bool

    // Without this axiom, Z3 knows nothing about isHexColor
    axiom hex_length : ∀(s : String). isHexColor(s) → strlen(s) = 7
    axiom hex_prefix : ∀(s : String). isHexColor(s) → charAt(s, 0) = "#"
}
```

**Rule of thumb:** If an operation maps to a Z3 native theory, no axioms needed for verification. If it's an uninterpreted function, axioms are the only way Z3 learns its properties.

## Summary

| Feature | Status |
|---------|--------|
| Basic operations | ✅ Native Z3 |
| Substring ops | ✅ Native Z3 |
| Regex constructors | ✅ Native Z3 |
| Regex matching | ✅ Native Z3 |
| Convenience predicates | ✅ Native Z3 |
| Int conversion | ✅ Native Z3 |
| Monoid structure | ✅ Algebraic |

See `src/solvers/z3/capabilities.toml` for the complete list of supported string and regex operations.

## What's Next?

Explore set theory operations and Z3's set reasoning:

→ [Sets](18-sets.md)

