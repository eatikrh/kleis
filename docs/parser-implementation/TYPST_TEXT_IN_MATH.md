# Typst: Text in Math Mode

**Date:** 2025-12-05  
**Status:** Important Usage Guide

## The Issue

When using Kleis templates with Typst rendering, multi-letter text in subscripts, superscripts, or other math contexts can cause errors like:

```
unknown variable: dimension
unknown variable: Hont
unknown variable: text
```

## Why This Happens

**Typst distinguishes between:**
1. **Variables** (single letters or known symbols): `x`, `i`, `n`, `α`, `∞`
2. **Text** (multi-letter words): `dimension`, `Hont`, `text`, `name`

In Typst math mode:
- `cal(H)_(n)` → Works! (n is a variable) ✅
- `cal(H)_(∞)` → Works! (∞ is a symbol) ✅
- `cal(H)_(Hont)` → **ERROR!** (Hont treated as undefined variable) ❌

## The Solution

**Wrap multi-letter text in quotes:**

```typst
cal(H)_("Hont")     ✅ Works!
cal(H)_("dimension") ✅ Works!
cal(M)_("H")        ✅ Works!
```

## Usage Guide for POT Operations

### Hont (Hilbert Ontology): 𝓗_subscript

**Template:** Click "Hont" button in POT tab → `𝓗_[□]`

**Correct usage:**

| What You Want | Type In Placeholder | Result |
|---------------|---------------------|--------|
| 𝓗_∞ | `∞` | ✅ Works (symbol) |
| 𝓗_n | `n` | ✅ Works (variable) |
| 𝓗_i | `i` | ✅ Works (variable) |
| 𝓗_Hont | `"Hont"` | ✅ Works (quoted text) |
| 𝓗_sep | `"sep"` | ✅ Works (quoted text) |
| 𝓗_dimension | `"dimension"` | ✅ Works (quoted text) |

**Wrong usage:**

| What You Type | Error |
|---------------|-------|
| `Hont` (no quotes) | ❌ unknown variable: Hont |
| `dimension` (no quotes) | ❌ unknown variable: dimension |
| `sep` (no quotes) | ❌ unknown variable: sep |

### Modal Space: 𝓜_subscript

**Same rule applies:**

| What You Want | Type In Placeholder | Result |
|---------------|---------------------|--------|
| 𝓜_H | `"H"` | ✅ Quoted for multi-letter |
| 𝓜_Hilbert | `"Hilbert"` | ✅ Quoted |
| 𝓜_config | `"config"` | ✅ Quoted |
| 𝓜_i | `i` | ✅ No quotes (single letter) |

### Other Operations That May Need Quotes

**When placeholder could be text:**

1. **Variable names in transforms:**
   - `ℱ[f]("omega")` if you type "omega" instead of `ω`
   - `ℒ[f]("time")` if you type "time" instead of `t`

2. **Function names:**
   - `K("spacetime", m)` if you type "spacetime"
   - Use `x` or `t` without quotes for single variables

3. **Domain names:**
   - `∫_("Domain") f dμ` if typing "Domain"
   - Use `M`, `V`, `D` without quotes for single letters

## General Rule

**Quote it if:**
- ✅ More than one letter: `"Hont"`, `"dimension"`, `"text"`
- ✅ You want it displayed as-is: `"POT"`, `"early"`, `"late"`
- ✅ Contains spaces: `"modal space"`

**Don't quote if:**
- ✅ Single letter: `x`, `i`, `n`, `m`
- ✅ Greek letter: `α`, `ω`, `μ`, `∞`
- ✅ Number: `1`, `2`, `42`
- ✅ Standard math symbol: `∞`, `∂`

## Examples

### Example 1: Hont Variants
```
Correct:
𝓗_("∞")       or  𝓗_(∞)        Both work
𝓗_("Hont")    Must quote
𝓗_("sep")     Must quote
𝓗_(n)         No quotes (single letter)
```

### Example 2: Modal Space
```
Correct:
𝓜_("H")           Quote even single if you want text "H"
𝓜_(H)             Treats H as variable (might work if H is defined)
𝓜_("Hilbert")     Must quote
𝓜_("config")      Must quote
```

### Example 3: Subscripts in General
```
Good:
x_(i)             Single letter variable ✅
x_(n)             Single letter variable ✅
T_("in")          Two letters = quote ✅
T_("out")         Multi-letter = quote ✅

Bad:
x_(in)            ERROR: unknown variable "in" ❌
T_(out)           ERROR: unknown variable "out" ❌
```

## Technical Details

### Typst Math Mode Rules
- In math mode, unquoted multi-letter sequences are identifiers
- Identifiers must be defined variables/functions
- Use `"text"` to insert literal text
- Alternative: `text("text")` function

### Our Templates
```typst
// What we generate:
cal(H)_({dimension})

// What user types:
{dimension} = "Hont"

// What Typst receives:
cal(H)_("Hont")  ✅ Correct!

// If user types without quotes:
{dimension} = Hont

// What Typst receives:
cal(H)_(Hont)  ❌ Error: unknown variable
```

## Quick Reference Card

```
╔══════════════════════════════════════════════════════════╗
║  TYPST TEXT IN MATH MODE - QUICK REFERENCE              ║
╠══════════════════════════════════════════════════════════╣
║  Single letter:     x, i, n         → No quotes needed   ║
║  Greek letter:      α, ω, ∞         → No quotes needed   ║
║  Multi-letter:      "Hont", "sep"   → MUST use quotes    ║
║  Text with spaces:  "modal space"   → MUST use quotes    ║
╚══════════════════════════════════════════════════════════╝
```

## POT-Specific Usage Examples

### Correct POT Notation
```
𝓗_("Hont")              Hilbert Ontology
𝓗_("∞")      or  𝓗_(∞)  Infinite dimensional
𝓗_("sep")               Separable
𝓜_("H")                 Modal space (Hilbert)
𝓜_("config")            Configuration space
c(x)                    Causal bound at x
K(x, m)                 Projection kernel
Π[ψ](x)                 Projection operator
```

### Building Complex Expressions
```
Expression: 𝓗_("Hont") → 𝓜_("phase") → Π → ℝ⁴

Steps:
1. Insert 𝓗 template, type: "Hont"
2. Insert →
3. Insert 𝓜 template, type: "phase"  
4. Insert →
5. Insert Π
6. Insert →
7. Insert ℝ⁴

Result: 𝓗_("Hont") → 𝓜_("phase") → Π → ℝ⁴ ✅
```

## Common Mistakes

### ❌ Mistake 1: Forgetting Quotes
```
Input:  𝓗_(Hont)
Error:  unknown variable: Hont
Fix:    𝓗_("Hont")
```

### ❌ Mistake 2: Quoting Single Letters
```
Input:  𝓗_("n")
Works:  Yes, but unnecessary
Better: 𝓗_(n)
```

### ❌ Mistake 3: Inconsistent Quoting
```
Input:  𝓗_(Hont) and 𝓜_(config)
Error:  Both fail
Fix:    𝓗_("Hont") and 𝓜_("config")
```

## When in Doubt

**Rule of thumb:** If it's more than one letter that isn't a standard math symbol, **use quotes!**

```
One letter or symbol?  → No quotes
Multiple letters?      → Use quotes
Not sure?             → Use quotes (always safe)
```

## Summary

✅ **Solution:** Type `"ont"` (with quotes) in the Hont subscript placeholder  
✅ **Works for:** All multi-letter text in subscripts/superscripts  
✅ **Applies to:** 𝓗, 𝓜, and any subscripted templates  
✅ **Remember:** Single letters (x, i, n) don't need quotes  

**Refresh your browser and try typing `"ont"` with quotes - it should work perfectly now!** 🎯

