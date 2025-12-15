# Architecture Decision Records (ADRs)

This directory contains all Architecture Decision Records for the Kleis project.

---

## Core Language Design

### ADR-001: Scalar Multiply
**File:** `adr-001-scalar-multiply.md`  
**Status:** Accepted  
**Summary:** Scalar multiplication semantics and operator precedence

### ADR-002: Eval vs Simplify
**File:** `adr-002-eval-vs-simplify.md`  
**Status:** Accepted  
**Summary:** Symbolic evaluation strategy (simplify, not evaluate to numbers)

### ADR-003: Self-Hosting Strategy
**File:** `adr-003-self-hosting.md`  
**Status:** In Progress  
**Summary:** Path to defining Kleis in Kleis itself  
**Current:** Level 1 complete (data types), Level 2 partial (simple functions)

---

## Type System

### ADR-014: Hindley-Milner Type System
**File:** `adr-014-hindley-milner-type-system.md`  
**Status:** Accepted & Implemented  
**Summary:** Constraint-based type inference with unification

### ADR-015: Text as Source of Truth
**File:** `adr-015-text-as-source-of-truth.md`  
**Status:** Accepted  
**Summary:** Kleis source files are canonical, not internal representations  
**Validation:** See `adr-015-validation-report.md`

### ADR-016: Operations in Structures
**File:** `adr-016-operations-in-structures.md`  
**Status:** Accepted & Implemented  
**Summary:** Type operations defined in structures, not hardcoded in type checker

### ADR-019: Dimensional Type Checking
**File:** `adr-019-dimensional-type-checking.md`  
**Status:** Proposed  
**Summary:** Physical dimensions and units as part of type system

### ADR-020: Metalanguage for Type Theory
**File:** `adr-020-metalanguage-for-type-theory.md`  
**Status:** Accepted  
**Summary:** Type-level vs value-level distinction

### ADR-021: Algebraic Data Types
**File:** `adr-021-algebraic-data-types.md`  
**Status:** Accepted & Implemented  
**Summary:** User-defined data types with pattern matching

---

## User Interface & Editing

### ADR-004: Input Visualization
**File:** `adr-004-input-visualization.md`  
**Status:** Accepted  
**Summary:** Visual feedback during mathematical input

### ADR-005: Visual Authoring
**File:** `adr-005-visual-authoring.md`  
**Status:** Proposed  
**Summary:** Visual tools for mathematical authoring

### ADR-009: WYSIWYG Structural Editor
**File:** `adr-009-wysiwyg-structural-editor.md`  
**Status:** Accepted  
**Summary:** Direct manipulation of mathematical structures

### ADR-010: Inline Editing
**File:** `adr-010-inline-editing.md`  
**Status:** Accepted  
**Summary:** Edit mathematics inline with rendered output

### ADR-011: Notebook Environment
**File:** `adr-011-notebook-environment.md`  
**Status:** Accepted  
**Summary:** Interactive computational notebook interface

### ADR-012: Document Authoring
**File:** `adr-012-document-authoring.md`  
**Status:** Accepted  
**Summary:** Full document authoring capabilities

### ADR-017: Vite + PatternFly Frontend
**File:** `adr-017-vite-patternfly-frontend.md`  
**Status:** Accepted  
**Summary:** Modern web frontend technology choices

---

## Grammar & Parsing

### ADR-006: Template-Grammar Duality
**File:** `adr-006-template-grammar-duality.md`  
**Status:** Accepted  
**Summary:** Templates and grammar as dual representations

### ADR-007: Bootstrap Grammar
**File:** `adr-007-bootstrap-grammar.md`  
**Status:** Accepted  
**Summary:** Minimal grammar for bootstrapping the system

### ADR-008: Bootstrap Grammar Boundary
**File:** `adr-008-bootstrap-grammar-boundary.md`  
**Status:** Accepted  
**Summary:** Defining what's in vs out of bootstrap grammar

---

## Formalism & Theory

### ADR-013: Paper Scope Hierarchy
**File:** `adr-013-paper-scope-hierarchy.md`  
**Status:** Accepted  
**Summary:** Scoping rules for mathematical papers and documents

### ADR-018: Universal Formalism
**File:** `adr-018-universal-formalism.md`  
**Status:** Proposed  
**Summary:** Unified formalism for mathematics across domains

---

## Implementation Notes

### ADR-015 Validation Report
**File:** `adr-015-validation-report.md`  
**Type:** Validation Document  
**Summary:** Evidence that text-as-source-of-truth works in practice

---

## Status Legend

- **Proposed:** Under consideration
- **Accepted:** Decision made, may not be implemented yet
- **Implemented:** Fully implemented in codebase
- **Accepted & Implemented:** Both decided and built
- **In Progress:** Partially implemented
- **Superseded:** Replaced by newer ADR

---

## Naming Convention

ADRs follow the pattern: `adr-NNN-short-title.md`

- Numbers are sequential
- Titles use kebab-case
- Some historical ADRs use different capitalization (ADR-016)

---

## Adding New ADRs

When creating a new ADR:

1. Use next available number
2. Follow template structure (problem, decision, consequences)
3. Add entry to this README
4. Update status as implementation progresses

---

**Total ADRs:** 22  
**Last Updated:** December 9, 2025

