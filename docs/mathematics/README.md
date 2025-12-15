# Kleis Mathematical Foundations

This directory contains formal mathematical documentation for Kleis, presenting the theoretical foundations from multiple perspectives.

---

## 📚 Documents by Audience

### For Mathematicians

**[mathematicians_guide_to_kleis.pdf](mathematicians_guide_to_kleis.pdf)**
- Explains Kleis constructs in mathematical terms
- Maps to standard algebra (Bourbaki, universal algebra)
- Shows: extends, over, where, nested structures
- **Start here if you're a mathematician**

**[bourbaki-style_foundational_appendix.pdf](bourbaki-style_foundational_appendix.pdf)**
- Axiomatic foundations in Bourbaki style
- Rigorous set-theoretic treatment
- Foundational definitions

**[magma_semigroup_monoid.pdf](magma_semigroup_monoid.pdf)**
- Basic algebraic structures: magmas, semigroups, monoids
- Foundation for understanding algebraic hierarchies
- Examples and constructions

**[higher_algebraic_structures.pdf](higher_algebraic_structures.pdf)**
- Groups, rings, fields, and modules
- More advanced algebraic theory
- Connection to Kleis type system

**[even_higher_algebraid_structures.pdf](even_higher_algebraid_structures.pdf)**
- Advanced algebraic structures
- Higher-level abstractions
- Research-level mathematics

### For Programmers

**[kleis_fo_rust_and_java_programmers.pdf](kleis_fo_rust_and_java_programmers_a_guided_introduction.pdf)**
- Kleis explained through Rust traits and Java interfaces
- Structure = trait, implements = impl block
- Practical onboarding for engineers
- **Start here if you know Rust or Java**

**[how_rust_code_implements_these.pdf](how_rust_code_implements_these.pdf)**
- Shows Rust implementation patterns
- Maps formal Kleis to executable Rust
- Traits, impl blocks, const generics
- **For implementers and contributors**

**[from_theory_to_practice.pdf](from_theory_to_practice.pdf)**
- Bidirectional bridge: theory ↔ implementation
- Semantic rules → executable code
- Explains the mapping in both directions
- **Connects abstract and concrete**

### For Type Theorists

**[kleis_language_specification.pdf](kleis_language_specification.pdf)**
- Formal language specification
- Syntax, typing rules, operational semantics
- Lexical structure, type system, evaluation
- **Formal reference document**

**[a_pattern-matching-exhaustiveness_lemma.pdf](a_pattern-matching-exhaiustive_lemma_in_inference_form.pdf)**
- Formal proof using inference rules (mathpartir)
- Exhaustiveness judgment defined inductively
- Lemma with rigorous proof sketch
- **Graduate-level type theory**

**[an_operational_semantics.pdf](an_operational_semantics_using_inference_rule.pdf)**
- Big-step operational semantics
- Evaluation rules using inference notation
- Formal execution model

**[a_non-redundancy_judgement.pdf](a_non-redundancy_judgement.pdf)**
- Pattern match non-redundancy checking
- Formal judgment rules
- Ensures no unreachable patterns

**[an_algorithmic_redundancy-checking.pdf](an_algorithmic_redundancy-chekcing_algorithm.pdf)**
- Algorithm for redundancy checking
- Practical implementation of formal rules
- Complexity analysis

### For Category Theorists

**[cathetory-theoretic_guide_to_kleis.pdf](cathetory-theoretic_guide_to_kleis.pdf)**
- Maps Kleis to Lawvere theories
- Structures = algebraic theories
- implements = models, extends = morphisms
- over = fibrations
- **Deep categorical foundations**

---

## 🎯 Recommended Reading Paths

### Path 1: Mathematician → Implementer
1. mathematicians_guide_to_kleis.pdf
2. kleis_language_specification.pdf
3. how_rust_code_implements_these.pdf

### Path 2: Programmer → Theorist
1. kleis_fo_rust_and_java_programmers.pdf
2. from_theory_to_practice.pdf
3. mathematicians_guide_to_kleis.pdf

### Path 3: Type Theorist → System Designer
1. kleis_language_specification.pdf
2. pattern-matching exhaustiveness lemma
3. operational_semantics
4. cathetory-theoretic_guide.pdf

### Path 4: Quick Overview
1. mathematicians_guide_to_kleis.pdf (start here!)
2. kleis_fo_rust_and_java_programmers.pdf (if you code)

---

## 📊 Document Statistics

**Total:** 14 documents (PDF + TeX source files)
**Level:** Graduate to research-level mathematics  
**Prerequisites:** Varies by document (abstract algebra to category theory)

---

## 🔧 Building PDFs

All PDFs are included. To rebuild from source:

```bash
cd docs/mathematics
pdflatex document_name.tex
```

**Note:** Some documents use specialized packages:
- `mathpartir` - For inference rules
- `stmaryrd` - For special symbols
- Unicode characters properly declared

---

## 📝 Document Purposes

### Theory Documents
- Establish mathematical foundations
- Show Kleis isn't ad-hoc design
- Multiple theoretical perspectives

### Practical Documents  
- Bridge to familiar concepts (traits, interfaces)
- Implementation guidance
- Onboarding different audiences

### Formal Documents
- Rigorous specifications
- Proofs of properties
- Academic reference

---

## 🌟 Key Contributions

**What makes these documents valuable:**

1. **Multiple perspectives** - Algebra, category theory, type theory, programming
2. **Rigorous proofs** - Not just descriptions, actual formal mathematics
3. **Practical bridges** - Theory connects to implementation
4. **Academic quality** - Research-grade documentation

**Result:** Kleis presented as a formally grounded system, not just a tool.

---

## 🔗 Related Documentation

- **[Grammar](../grammar/)** - Formal grammar specifications
- **[Parser Implementation](../parser-implementation/)** - Implementation status
- **[Type System](../type-system/)** - Type system documentation
- **[Vision](../vision/)** - Future direction (REPL, Notebook)
- **[ADRs](../adr/)** - Architecture decision records

---

**Created:** December 11, 2025  
**Purpose:** Formal mathematical foundations for Kleis  
**Audience:** Mathematicians, type theorists, programmers, category theorists  
**Status:** Research-grade documentation

