# Changelog

All notable changes to Kleis will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Structural editor primitive templates (□ = □, □ + □, □ − □, □·□, −□, ±)
- Content-aware viewBox calculation (proportional padding, handles negative coordinates)
- 8 balanced palette tabs (Basics, Fences, Accents, Calculus, Linear Algebra, Greek, Logic, Physics)
- Complete type system design documentation (algebraic hierarchy, extensibility)
- Evaluation syntax design (substitute, eval, multi-valued operations)
- Type system UX design (context management, inference prompts)
- Universal verification vision (math + business + legal + medical domains)
- arXiv integration vision (native .kleis format, verified papers)
- AI verification framework (catch LLM hallucinations)

### Changed
- Major README overhaul - positioned as "formal mathematical reasoning engine"
- Documentation structure reorganized (theory/, type-system/, vision/, archive/)
- Removed 19 obsolete session summary documents
- Updated palette organization for better balance

### Fixed
- ViewBox clipping for expressions with negative coordinates
- Plus-minus symbol (±) rendering in structural mode

---

## [2.0.0] - 2024-11-22

### Added
- UUID-based deterministic positioning (92.7% of gallery examples)
- Option B filtering (show leaf markers only, hide parent operations)
- Vertical clipping fix via viewBox expansion
- `literal_chain` UUID wrapping for correct "1n", "mn" positioning
- `function_call` argument wrapping for correct multi-argument positioning
- `mathrm` skip-wrap to prevent literal UUID text in output

### Changed
- Transitioned from spatial heuristics to deterministic UUID lookup
- Modified bounding box calculation to use direct UUID→Position mapping

### Fixed
- Matrix element markers (all 9 cells positioned correctly)
- Integral variable markers (dx, dy, dz each get unique positions)
- Subscript/superscript positioning in complex nested structures

---

## [1.0.0] - 2024-11-01

### Added
- Typst-based structural editor (WYSIWYG)
- 100+ mathematical operations
- 91 gallery examples
- LaTeX parser with template inference
- Web UI with bidirectional text ↔ structural editing
- Semantic bounding boxes for interactive overlays
- Full palette system (Greek, operators, templates)

### Technical
- Rust-based renderer
- Actix-web server
- MathJax integration
- Typst compilation pipeline

---

## [0.1.0] - Initial Release

### Added
- Basic expression rendering
- LaTeX output
- Unicode output
- Core mathematical operations

---

## Version Naming

- **Major (X.0.0):** Breaking changes, major architectural shifts
- **Minor (0.X.0):** New features, backward compatible
- **Patch (0.0.X):** Bug fixes, documentation

---

**Current Version:** 2.0.0+ (unreleased features in main branch)
**Release Cycle:** When type system implementation complete

