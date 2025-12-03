# Template Implementation Strategy for Kleis Structural Editor

**Date:** 2024-11-22  
**Status:** Design Document  
**Related ADRs:** [ADR-006](adr-006-template-grammar-duality.md) (Template-Grammar Duality), [ADR-009](adr-009-wysiwyg-structural-editor.md) (WYSIWYG Structural Editor)

**Note:** This document provides detailed implementation guidance for the structural editor architecture described in ADR-009. Read ADR-009 first for the high-level vision and motivation.

---

## Overview

This document outlines the strategy for implementing the template-based structural editor described in ADR-009. The template system is the bridge between user-defined operations and the visual editing interface - templates encode both the grammar structure (via placeholders) and the rendering instructions.

**Core Principle (from ADR-006):** Template placeholders ARE the grammar structure. When a template says `{left} ∧ {right}`, it defines a 2-argument operation where placeholders are named "left" and "right".

---

## 1. Template Structure - Current State

### What Already Exists

The system already has templates in `src/render.rs`:

```rust
GlyphContext {
    latex_templates: HashMap<String, String> {
        "scalar_divide" → "\frac{{num}}{{den}}",
        "sqrt" → "\sqrt{{arg}}",
        "wedge" → "{left} \\wedge {right}",
        "int_bounds" → "\\int_{{{lower}}}^{{{upper}}} {integrand} \\, d{var}",
        // ... 73 total operations
    },
    unicode_templates: HashMap<String, String> {
        "scalar_divide" → "{num}/{den}",
        "sqrt" → "√{arg}",
        "wedge" → "{left} ∧ {right}",
        // ... corresponding Unicode versions
    }
}
```

### Key Observation

These template strings already contain all the metadata needed:
- **Placeholders**: Text in `{braces}` 
- **Arity**: Count of distinct placeholders
- **Order**: Sequence placeholders appear in template
- **Rendering**: The literal text between placeholders

---

## 2. Template Metadata Extraction

### What We Need to Extract

For each template, extract:

| Property | Description | Example (from `\frac{{num}}{{den}}`) |
|----------|-------------|--------------------------------------|
| **Arity** | Number of distinct placeholders | 2 |
| **Placeholder names** | Variable names from `{...}` | `["num", "den"]` |
| **Placeholder hints** | User-facing descriptions | `["numerator", "denominator"]` |
| **Glyph** | Visual symbol for palette | `"÷"` or `"[a/b]"` |
| **Tab order** | Sequence to navigate | `[0, 1]` (num first, then den) |

### Template Parsing

Need a mini-parser for templates:

**Input:** `"\frac{{num}}{{den}}"`

**Parse to:**
```rust
struct TemplateParts {
    parts: Vec<TemplatePart>
}

enum TemplatePart {
    Literal(String),        // Fixed text like "\frac{" or "}{"
    Placeholder(String)     // Variable part like "num" or "den"
}

// Result:
TemplateParts {
    parts: [
        Literal("\\frac{"),
        Placeholder("num"),
        Literal("}{"),
        Placeholder("den"),
        Literal("}")
    ]
}
```

**Use this for:**
- Count placeholders → determine arity
- Extract names → create Placeholder AST nodes
- Validate structure → ensure balanced braces
- Generate rendering → know where to insert values

### Parsing Algorithm

```rust
fn parse_template(template: &str) -> Result<TemplateParts, Error> {
    let mut parts = Vec::new();
    let mut current_literal = String::new();
    let mut in_placeholder = false;
    let mut placeholder_name = String::new();
    
    for ch in template.chars() {
        match (ch, in_placeholder) {
            ('{', false) => {
                if !current_literal.is_empty() {
                    parts.push(TemplatePart::Literal(current_literal.clone()));
                    current_literal.clear();
                }
                in_placeholder = true;
            }
            ('}', true) => {
                parts.push(TemplatePart::Placeholder(placeholder_name.clone()));
                placeholder_name.clear();
                in_placeholder = false;
            }
            (ch, true) => placeholder_name.push(ch),
            (ch, false) => current_literal.push(ch),
        }
    }
    
    if !current_literal.is_empty() {
        parts.push(TemplatePart::Literal(current_literal));
    }
    
    Ok(TemplateParts { parts })
}

fn extract_placeholders(template: &str) -> Vec<String> {
    let parts = parse_template(template)?;
    parts.parts.iter()
        .filter_map(|p| match p {
            TemplatePart::Placeholder(name) => Some(name.clone()),
            _ => None
        })
        .collect()
}
```

---

## 3. Template Instantiation - Creating Placeholder ASTs

### The Insertion Flow

When user clicks a palette button:

```
User clicks [÷] button
    ↓
Look up template: "scalar_divide"
    ↓
Extract placeholders: ["num", "den"]
    ↓
Generate AST:
    Operation {
        name: "scalar_divide",
        args: [
            Placeholder { id: 0, hint: "numerator" },
            Placeholder { id: 1, hint: "denominator" }
        ]
    }
    ↓
Render with placeholder boxes visible
    ↓
Focus first placeholder (id: 0)
```

### Implementation Sketch

```rust
fn instantiate_template(
    operation_name: &str,
    registry: &TemplateRegistry,
    id_generator: &mut PlaceholderIdGenerator
) -> Expression {
    let metadata = registry.get(operation_name).unwrap();
    
    let args: Vec<Expression> = metadata.placeholders
        .iter()
        .map(|ph| Expression::Placeholder {
            id: id_generator.next(),
            hint: ph.hint.clone(),
            expected_type: ph.expected_type.clone()
        })
        .collect();
    
    Expression::Operation {
        name: operation_name.to_string(),
        args
    }
}
```

---

## 4. Two-Level Template System

### Level 1: Bootstrap Templates (Hardcoded in Rust)

**Location:** `src/render.rs` - already exists

**Contains:**
- Core mathematical operations (73 currently)
- Predefined placeholder mappings
- Ships with the system
- Always available

**Examples:**
- `scalar_divide`, `sqrt`, `sup`, `sub`
- `int_bounds`, `sum_bounds`, `prod_bounds`
- `grad`, `d_part`, `nabla`
- `ket`, `bra`, `inner`

### Level 2: User-Defined Templates (from .kleis files)

**Location:** User `.kleis` files or packages

**Syntax:**
```kleis
operation wedge : (Form, Form) -> Form

template wedge {
    glyph: "∧",
    latex: "{left} \\wedge {right}",
    unicode: "{left} ∧ {right}",
    hints: ["left form", "right form"]
}
```

**Parser extracts:**
- `glyph`: "∧" → becomes palette button icon
- `latex` template → used for LaTeX rendering
- `unicode` template → used for Unicode rendering
- `hints` → user-facing placeholder descriptions
- Placeholders: `["left", "right"]` → extracted from template strings
- Arity: 2 → counted from placeholders

**Result:** Generate insertion function automatically, no manual coding needed.

### Template Loading Priority

1. **Core bootstrap** templates loaded first (from Rust)
2. **Standard library** templates (from `kleis/` directory)
3. **Package** templates (from installed packages)
4. **User** templates (from current project)

Later definitions can override earlier ones.

---

## 5. Placeholder Rendering Strategy

### Placeholder AST Node

Add to `src/ast.rs`:

```rust
pub enum Expression {
    Const(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
    Placeholder { 
        id: usize,              // Unique within document
        hint: String,           // User-facing description
        expected_type: Option<Type>  // Future: type validation
    }
}
```

### Visual Rendering

Each placeholder needs:

| Aspect | Implementation |
|--------|---------------|
| **Visual box** | Empty rectangle with dashed border |
| **State tracking** | CSS classes: `.placeholder.empty`, `.placeholder.active`, `.placeholder.filled` |
| **AST path** | Data attribute: `data-ast-path="[0,1,0]"` |
| **Focus management** | Track which placeholder ID is currently active |
| **Click handling** | Attach `onclick` handlers to placeholder divs |

### Rendering Pipeline

```
AST with placeholders
    ↓
Walk tree recursively
    ↓
When encountering Placeholder node:
    - Generate: <div class="placeholder" 
                     data-id="X" 
                     data-hint="..." 
                     onclick="editPlaceholder(X)">
                  □
                </div>
    ↓
When encountering filled Operation/Const/Object:
    - Render normally using existing render.rs logic
    ↓
Attach click handlers to all placeholder divs
    ↓
On click: Focus that placeholder, show input method
```

### CSS Styling

```css
.placeholder {
    display: inline-block;
    min-width: 20px;
    min-height: 20px;
    border: 2px dashed #667eea;
    background: #f0f4ff;
    cursor: text;
    padding: 2px 4px;
    margin: 0 2px;
}

.placeholder.active {
    border-color: #ff6b6b;
    background: #fff5f5;
    animation: pulse 1s infinite;
}

.placeholder.filled {
    /* Placeholder replaced with actual content */
    border: none;
    background: transparent;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
}
```

---

## 6. Smart Palette Organization

### Palette Structure

Templates organized by multiple dimensions:

```rust
struct PaletteOrganization {
    by_category: HashMap<Category, Vec<TemplateButton>>,
    by_frequency: Vec<TemplateButton>,  // Recently/commonly used
    by_context: HashMap<Context, Vec<TemplateButton>>,  // Context-aware
    by_package: HashMap<PackageName, Vec<TemplateButton>>,
}

enum Category {
    BasicOps,
    Calculus,
    LinearAlgebra,
    DifferentialGeometry,
    QuantumMechanics,
    Custom(String),
}

struct TemplateButton {
    operation_name: String,
    glyph: String,
    tooltip: String,  // "Fraction (a/b)" or "Wedge product"
    category: Category,
    shortcut: Option<String>,  // "/" for fraction, "^" for superscript
}
```

### Context-Aware Suggestions

Based on current placeholder hint, suggest relevant templates:

```rust
fn suggest_templates(context: &PlaceholderContext) -> Vec<TemplateButton> {
    match context.hint.as_str() {
        hint if hint.contains("exponent") => 
            vec!["Number", "Variable", "Expression"],
        
        hint if hint.contains("condition") =>
            vec!["<", ">", "≤", "≥", "=", "≠"],
        
        hint if hint.contains("integrand") =>
            vec!["Fraction", "Product", "Power", "Function"],
        
        hint if hint.contains("tensor") =>
            vec!["Index", "Contraction", "Covariant deriv"],
        
        _ => all_templates()  // Show everything
    }
}
```

### Dynamic Palette Generation

For each operation in registry:

```rust
fn generate_palette(registry: &TemplateRegistry) -> Palette {
    let mut palette = Palette::new();
    
    for (op_name, metadata) in registry.iter() {
        let button = TemplateButton {
            operation_name: op_name.clone(),
            glyph: metadata.glyph.clone(),
            tooltip: format!("{} ({})", 
                           metadata.display_name, 
                           metadata.type_signature),
            category: metadata.category.clone(),
            shortcut: metadata.keyboard_shortcut.clone(),
        };
        
        palette.add_button(button);
    }
    
    palette
}
```

**Key:** No manual UI code - palette buttons generated from template metadata.

---

## 7. Template Metadata Storage

### Extended GlyphContext or Separate Registry?

**Option A: Extend GlyphContext**

```rust
pub struct GlyphContext {
    // Existing fields
    unicode_glyphs: HashMap<String, String>,
    unicode_templates: HashMap<String, String>,
    latex_glyphs: HashMap<String, String>,
    latex_templates: HashMap<String, String>,
    
    // NEW: Template metadata
    template_metadata: HashMap<String, TemplateMetadata>,
}

pub struct TemplateMetadata {
    operation_name: String,
    glyph: String,
    display_name: String,
    category: Category,
    placeholders: Vec<PlaceholderInfo>,
    type_signature: String,
    keyboard_shortcut: Option<String>,
}

pub struct PlaceholderInfo {
    name: String,           // "num", "den", "left", "right"
    hint: String,           // "numerator", "denominator"
    expected_type: Option<Type>,
    order: usize,           // Tab order
}
```

**Option B: Separate TemplateRegistry**

```rust
pub struct TemplateRegistry {
    templates: HashMap<String, TemplateMetadata>,
    glyph_context: GlyphContext,  // Rendering context
}

impl TemplateRegistry {
    pub fn register_template(&mut self, op: OperationDef, tmpl: TemplateDef) {
        // Extract placeholders from template strings
        let placeholders = self.extract_placeholders(&tmpl);
        
        // Validate arity matches
        self.validate_arity(&op, &placeholders)?;
        
        // Store metadata
        let metadata = TemplateMetadata {
            operation_name: op.name.clone(),
            glyph: tmpl.glyph.clone(),
            placeholders,
            // ...
        };
        
        self.templates.insert(op.name.clone(), metadata);
        
        // Register with rendering context
        self.glyph_context.add_template(op.name, tmpl);
    }
}
```

**Recommendation:** Option B (separate registry) for cleaner separation of concerns.

---

## 8. Template Validation

### Validation Requirements

Templates must be validated against operation signatures:

```kleis
operation scalar_divide : (Scalar, Scalar) -> Scalar
template scalar_divide { 
    latex: "\frac{{num}}{{den}}" 
}
         ↓              ↓
    2 placeholders = 2 args ✓
```

### Validation Checks

```rust
fn validate_template(
    op: &OperationDef, 
    template: &TemplateDef
) -> Result<(), ValidationError> {
    // 1. Extract placeholders from all template strings
    let latex_placeholders = extract_placeholders(&template.latex)?;
    let unicode_placeholders = extract_placeholders(&template.unicode)?;
    
    // 2. Check consistency across targets
    if latex_placeholders != unicode_placeholders {
        return Err(ValidationError::InconsistentPlaceholders {
            latex: latex_placeholders,
            unicode: unicode_placeholders,
        });
    }
    
    // 3. Check arity matches operation signature
    let template_arity = latex_placeholders.len();
    let operation_arity = op.input_types.len();
    
    if template_arity != operation_arity {
        return Err(ValidationError::ArityMismatch {
            operation: op.name.clone(),
            expected: operation_arity,
            found: template_arity,
        });
    }
    
    // 4. Check for duplicate placeholder names
    let unique_names: HashSet<_> = latex_placeholders.iter().collect();
    if unique_names.len() != latex_placeholders.len() {
        return Err(ValidationError::DuplicatePlaceholders);
    }
    
    // 5. Validate placeholder names (no special chars, not empty)
    for name in latex_placeholders.iter() {
        if !is_valid_placeholder_name(name) {
            return Err(ValidationError::InvalidPlaceholderName(name.clone()));
        }
    }
    
    Ok(())
}
```

### Type Hints from Placeholder Names

Placeholder names can suggest expected types:

```rust
fn infer_type_from_name(name: &str) -> Option<Type> {
    match name {
        "scalar" | "num" | "den" | "coeff" => Some(Type::Scalar),
        "vector" | "vec" => Some(Type::Vector),
        "matrix" | "mat" => Some(Type::Matrix),
        "tensor" => Some(Type::Tensor),
        "index" | "idx" | "upper" | "lower" => Some(Type::Index),
        "form" => Some(Type::DifferentialForm),
        _ => None
    }
}
```

---

## 9. Placeholder ID Management

### The ID Assignment Problem

Placeholders need globally unique IDs within a document:

```
Consider building: (a/b) + (c/d)

Operation("plus", [
  Operation("scalar_divide", [
    Placeholder { id: 0, hint: "num" },  ← Unique
    Placeholder { id: 1, hint: "den" }
  ]),
  Operation("scalar_divide", [
    Placeholder { id: 2, hint: "num" },  ← Can't reuse 0
    Placeholder { id: 3, hint: "den" }   ← Can't reuse 1
  ])
])
```

**Why unique IDs matter:**
- Focus management (which placeholder is active?)
- Navigation (Tab to next placeholder)
- Click handling (which placeholder was clicked?)
- Fill operations (replace specific placeholder)

### ID Generation Strategy

```rust
pub struct PlaceholderIdGenerator {
    next_id: usize,
}

impl PlaceholderIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    
    pub fn next(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

// Document-level state
pub struct EditorState {
    ast: Expression,
    id_generator: PlaceholderIdGenerator,
    active_placeholder: Option<usize>,
}
```

### ID Lifecycle

1. **Creation**: When template inserted, generate new IDs
2. **Filling**: When placeholder filled, ID removed (replaced with actual expression)
3. **Navigation**: Track current ID, find next/previous
4. **Persistence**: If saving document with unfilled placeholders, preserve IDs

---

## 10. Template Loading Architecture

### Startup Sequence

```
Application Start
    ↓
1. Initialize Core Registry
    - Load hardcoded templates from render.rs
    - 73 bootstrap operations
    ↓
2. Load Standard Library
    - Parse kleis/*.kleis files
    - Extract operation + template definitions
    ↓
3. Load Installed Packages
    - Scan ~/.kleis/packages/ or similar
    - Load package manifests
    - Parse package .kleis files
    ↓
4. Validate All Templates
    - Check arity matches
    - Verify placeholder consistency
    - Detect conflicts/overrides
    ↓
5. Build Template Registry
    - Merge all sources
    - Resolve precedence (user > package > stdlib > core)
    ↓
6. Generate Palette UI
    - Create buttons from glyphs
    - Organize by category
    - Attach insertion handlers
    ↓
7. Ready for User Interaction
```

### Template Source Priority

When multiple templates define same operation:

```
Priority (highest to lowest):
1. User project templates (./templates/)
2. Installed packages (~/.kleis/packages/)
3. Standard library (kleis/*.kleis)
4. Core bootstrap (src/render.rs)
```

Later definitions override earlier ones.

### Package Structure

```
my-diffgeo-package/
├── manifest.kleis
├── operations/
│   ├── christoffel.kleis
│   ├── riemann.kleis
│   └── covariant_deriv.kleis
├── templates/
│   ├── tensor_notation.kleis
│   └── index_notation.kleis
└── examples/
    └── einstein_equations.kleis
```

**manifest.kleis:**
```kleis
package diffgeo {
    name: "Differential Geometry Notation",
    version: "1.0.0",
    author: "Jane Mathematician",
    
    exports: [
        operations.christoffel,
        operations.riemann,
        operations.covariant_deriv
    ]
}
```

### Runtime Loading API

```rust
pub struct PackageLoader {
    registry: TemplateRegistry,
}

impl PackageLoader {
    pub fn load_package(&mut self, path: &Path) -> Result<(), Error> {
        // 1. Parse manifest
        let manifest = self.parse_manifest(path.join("manifest.kleis"))?;
        
        // 2. Load operations
        for op_file in manifest.exports.iter() {
            let op_def = self.parse_operation_file(path.join(op_file))?;
            let template_def = self.parse_template_file(path.join(op_file))?;
            
            // 3. Validate
            validate_template(&op_def, &template_def)?;
            
            // 4. Register
            self.registry.register_template(op_def, template_def)?;
        }
        
        Ok(())
    }
}
```

---

## 11. The "Empty Slate" Implementation

### Initial State

```html
<div id="equation-editor">
    <!-- Empty canvas -->
    <div id="equation-canvas" class="empty">
        <div class="empty-message">Click a template to start</div>
    </div>
    
    <!-- Palette (auto-generated from registry) -->
    <div id="template-palette">
        <!-- Generated buttons -->
    </div>
</div>
```

```javascript
// Initial state
const state = {
    ast: null,                    // No expression yet
    activeNode: null,             // No active placeholder
    idGenerator: new IdGenerator()
};
```

### First Interaction

User clicks `[÷]` button:

```javascript
function onPaletteButtonClick(operationName) {
    if (state.ast === null) {
        // Empty slate - create root expression
        state.ast = instantiateTemplate(operationName, state.idGenerator);
        state.activeNode = findFirstPlaceholder(state.ast);
    } else if (state.activeNode !== null) {
        // Replace active placeholder with template
        const newExpr = instantiateTemplate(operationName, state.idGenerator);
        replaceNode(state.ast, state.activeNode, newExpr);
        state.activeNode = findFirstPlaceholder(newExpr);
    } else {
        // No active placeholder - show error or append
        showMessage("Select a placeholder first");
    }
    
    render(state.ast);
    focusPlaceholder(state.activeNode);
}
```

### Building Process

```
State: Empty canvas
    ↓
User clicks [=] button
    ↓
ast = Operation("equals", [Placeholder(0), Placeholder(1)])
activeNode = 0 (left side)
    ↓
Render: □ = □
        ↑ (active)
    ↓
User clicks [÷] button
    ↓
Replace Placeholder(0) with:
    Operation("scalar_divide", [Placeholder(2), Placeholder(3)])
activeNode = 2 (numerator)
    ↓
Render: □/□ = □
        ↑ (active)
    ↓
User types "1"
    ↓
Replace Placeholder(2) with Const("1")
activeNode = 3 (denominator)
    ↓
Render: 1/□ = □
          ↑ (active)
    ↓
Continue...
```

---

## 12. Incremental Building Flow

### AST Transformation Operations

```rust
// Core operations for building expressions
impl Expression {
    /// Find node at path
    pub fn get_node_at(&self, path: &[usize]) -> Option<&Expression> {
        // Navigate through AST following path indices
    }
    
    /// Replace node at path
    pub fn replace_node_at(&mut self, path: &[usize], new_expr: Expression) {
        // Replace node at specified path
    }
    
    /// Find first placeholder in subtree
    pub fn find_first_placeholder(&self) -> Option<usize> {
        // Depth-first search for Placeholder node
    }
    
    /// Find next placeholder after given ID
    pub fn find_next_placeholder(&self, current_id: usize) -> Option<usize> {
        // DFS for next Placeholder with id > current_id
    }
    
    /// Count unfilled placeholders
    pub fn count_placeholders(&self) -> usize {
        // Count all Placeholder nodes
    }
    
    /// Check if expression is complete (no placeholders)
    pub fn is_complete(&self) -> bool {
        self.count_placeholders() == 0
    }
}
```

### User Interaction Handlers

```javascript
// Fill placeholder with value
function fillPlaceholder(placeholderId, value) {
    // 1. Parse value (if text input)
    const parsed = parseLatexFragment(value);
    
    // 2. Find placeholder in AST
    const path = findPlaceholderPath(state.ast, placeholderId);
    
    // 3. Replace placeholder with parsed expression
    state.ast.replaceNodeAt(path, parsed);
    
    // 4. Move to next placeholder
    state.activeNode = state.ast.findNextPlaceholder(placeholderId);
    
    // 5. Re-render
    render(state.ast);
    
    // 6. Focus next placeholder
    if (state.activeNode !== null) {
        focusPlaceholder(state.activeNode);
    } else {
        // No more placeholders - expression complete
        onExpressionComplete();
    }
}

// Insert template at placeholder
function insertTemplate(placeholderId, operationName) {
    // 1. Instantiate template
    const newExpr = instantiateTemplate(operationName, state.idGenerator);
    
    // 2. Find placeholder
    const path = findPlaceholderPath(state.ast, placeholderId);
    
    // 3. Replace with template
    state.ast.replaceNodeAt(path, newExpr);
    
    // 4. Find first placeholder in new template
    state.activeNode = newExpr.findFirstPlaceholder();
    
    // 5. Re-render
    render(state.ast);
    focusPlaceholder(state.activeNode);
}
```

### Keyboard Shortcuts

```javascript
function handleKeyboard(event) {
    switch(event.key) {
        case 'Tab':
            event.preventDefault();
            if (event.shiftKey) {
                navigatePreviousPlaceholder();
            } else {
                navigateNextPlaceholder();
            }
            break;
            
        case 'Escape':
            clearCurrentPlaceholder();
            break;
            
        case 'Enter':
            acceptPlaceholderAndNext();
            break;
            
        case '/':
            insertTemplate(state.activeNode, 'scalar_divide');
            break;
            
        case '^':
            insertTemplate(state.activeNode, 'sup');
            break;
            
        case '_':
            insertTemplate(state.activeNode, 'sub');
            break;
    }
}
```

---

## 13. The Package System

### User Installs Package

```bash
$ kleis add diffgeo-pkg
```

**What happens:**

```
1. Download package
    → ~/.kleis/packages/diffgeo-pkg/
    
2. Parse manifest
    → Read exports list
    
3. Load operations
    → Parse .kleis files
    → Extract operation definitions
    
4. Load templates
    → Extract template definitions
    → Validate against operations
    
5. Register with runtime
    → Add to TemplateRegistry
    → Update GlyphContext
    
6. Update UI
    → Generate new palette buttons
    → Add to appropriate categories
    
7. Ready to use
    → User can immediately use new notation
```

### Package Contents

**diffgeo-pkg/operations/christoffel.kleis:**
```kleis
operation christoffel : (Metric, Index, Index, Index) -> Scalar

template christoffel {
    glyph: "Γ",
    category: "Differential Geometry",
    latex: "\\Gamma^{{{upper}}}_{{{lower1} {lower2}}}",
    unicode: "Γ^{upper}_{lower1 lower2}",
    hints: ["metric", "upper index", "lower index 1", "lower index 2"]
}

annotation christoffel {
    description: "Christoffel symbol of the second kind",
    see_also: ["riemann", "ricci"]
}
```

### Using Package in Editor

After installation:

1. **Palette updates** - New button appears: `[Γ]` in "Differential Geometry" category
2. **Tooltip** - Shows: "Christoffel symbol (Metric, Index, Index, Index) → Scalar"
3. **Click button** - Inserts: `Γ^□_□□` with 4 placeholders
4. **Fill placeholders** - Build complete expression
5. **Export** - Perfect LaTeX: `\Gamma^{\mu}_{\nu \rho}`

**Key point:** User never wrote UI code. Template definition automatically generated working editor interface.

---

## 14. Critical Design Decisions

### A. Template String Format

**Decision:** Use current format from render.rs

**Syntax:**
- `{placeholder_name}` for substitution
- Double braces `{{...}}` when needed for LaTeX escaping
- Simple, parseable with regex or basic state machine

**Examples:**
```
"\frac{{num}}{{den}}"              → Fraction
"\sqrt{{arg}}"                      → Square root  
"{left} \\wedge {right}"            → Wedge product
"\\int_{{{lower}}}^{{{upper}}} {integrand} \\, d{var}"  → Integral
```

**Rationale:**
- Already proven in current system
- Human-readable and writable
- Easy to parse
- Familiar to LaTeX users

### B. Placeholder AST Representation

**Decision:** Add new variant to Expression enum

```rust
pub enum Expression {
    Const(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
    Placeholder {                  // NEW
        id: usize,                 // Unique within document
        hint: String,              // User-facing description
        expected_type: Option<Type> // Optional type constraint
    }
}
```

**Rationale:**
- Clean separation from other expression types
- Easy to identify and handle specially in rendering
- Can traverse and count placeholders easily
- Type information available for future validation

### C. Template Metadata Storage

**Decision:** Create separate TemplateRegistry

```rust
pub struct TemplateRegistry {
    templates: HashMap<String, TemplateMetadata>,
    glyph_context: GlyphContext,
}

pub struct TemplateMetadata {
    operation_name: String,
    glyph: String,
    display_name: String,
    category: Category,
    placeholders: Vec<PlaceholderInfo>,
    type_signature: String,
    keyboard_shortcut: Option<String>,
}
```

**Rationale:**
- Separation of concerns (rendering vs metadata)
- Easier to extend with new metadata fields
- Can validate before adding to rendering context
- Cleaner API for palette generation

### D. Palette Auto-Generation

**Decision:** Declarative palette generation from registry

```rust
fn generate_palette_ui(registry: &TemplateRegistry) -> PaletteHTML {
    let mut html = String::new();
    
    // Group by category
    for (category, templates) in registry.group_by_category() {
        html.push_str(&format!("<div class='category'>{}</div>", category));
        
        for template in templates {
            html.push_str(&format!(
                "<button class='template-btn' 
                         data-operation='{}' 
                         title='{}'>
                    {}
                 </button>",
                template.operation_name,
                template.tooltip(),
                template.glyph
            ));
        }
    }
    
    html
}
```

**Rationale:**
- No manual UI code for new operations
- Consistent appearance
- Automatic organization
- Easy to reskin or reorganize

### E. Placeholder Focus Management

**Decision:** Single active placeholder tracked in editor state

```javascript
const state = {
    ast: Expression,
    activeNode: number | null,  // ID of active placeholder
    // ...
};
```

**Rationale:**
- Clear focus state
- Simple navigation (next/previous)
- Easy to highlight in UI
- Natural for keyboard-driven editing

---

## 15. Why This Will Work

### 1. Foundation Already Exists

✅ **GlyphContext** already has template strings for 73 operations  
✅ **Parser** can convert LaTeX ↔ AST bidirectionally  
✅ **Renderer** can output to multiple targets (LaTeX, Unicode)  
✅ **AST** is unified and clean (ast.rs)

### 2. Template Metadata is Implicit

Templates already encode everything needed:
- **Placeholders** → Grammar structure
- **Arity** → Count of placeholders
- **Order** → Sequence in template
- **Rendering** → Literal template text

Just need to **extract** what's already there.

### 3. Clean Separation of Concerns

```
Templates (render.rs)
    ↓ Extract metadata
TemplateRegistry
    ↓ Generate UI
Palette Buttons
    ↓ User clicks
AST with Placeholders
    ↓ Render
Visual Editor
```

Each layer has clear responsibility.

### 4. Incremental Implementation Path

**Phase 1:** Hardcoded templates (already done)  
**Phase 2:** Add Placeholder to AST, render as boxes  
**Phase 3:** Extract metadata, generate palette  
**Phase 4:** Parse .kleis template definitions  
**Phase 5:** Package system with runtime loading

Each phase builds on previous, no big-bang rewrite.

### 5. Aligns with ADRs

- **ADR-006:** Template placeholders = grammar structure ✓
- **ADR-009:** Foundation complete, ready for UI layer ✓
- **ADR-007/008:** Bootstrap grammar supports this ✓

### 6. Proven Approach

Similar systems exist:
- MathType (but not extensible)
- Jupyter notebooks (but text-based)
- Scratch (visual programming with palette)
- Lean/Coq proof assistants (structural editing)

Kleis combines best aspects: **visual + structural + extensible**.

---

## 16. Implementation Roadmap

### Phase 1: Placeholder Foundation (Week 1-2)

**Tasks:**
- [ ] Add `Placeholder` variant to Expression enum
- [ ] Update renderer to display placeholders as boxes
- [ ] Add CSS styling for placeholder states
- [ ] Basic click handler to identify placeholders
- [ ] Simple input dialog to fill placeholders

**Deliverable:** Can manually create AST with placeholders and see them rendered.

### Phase 2: Template Metadata (Week 3-4)

**Tasks:**
- [ ] Implement template parser (extract placeholders)
- [ ] Build TemplateRegistry structure
- [ ] Extract metadata from existing render.rs templates
- [ ] Validate arity against operation definitions
- [ ] Generate placeholder hints from names

**Deliverable:** Registry of all 73 operations with metadata.

### Phase 3: Palette Generation (Week 5-6)

**Tasks:**
- [ ] Implement palette UI generator
- [ ] Add category grouping
- [ ] Create template buttons with glyphs
- [ ] Wire up click handlers
- [ ] Implement template instantiation

**Deliverable:** Working palette with all 73 operations, can insert templates.

### Phase 4: Navigation & Editing (Week 7-8)

**Tasks:**
- [ ] Implement Tab navigation between placeholders
- [ ] Add keyboard shortcuts (/, ^, _, etc.)
- [ ] Inline editing of placeholders
- [ ] Auto-focus next placeholder after fill
- [ ] Completion detection (no more placeholders)

**Deliverable:** Full editing workflow from empty slate to complete expression.

### Phase 5: Template Loading (Week 9-12)

**Tasks:**
- [ ] Parse operation definitions from .kleis files
- [ ] Parse template definitions from .kleis files
- [ ] Implement validation layer
- [ ] Runtime registration of new templates
- [ ] Package manifest parser
- [ ] Package loader

**Deliverable:** User-defined operations work in editor.

### Phase 6: Package System (Week 13-16)

**Tasks:**
- [ ] Package discovery and installation
- [ ] Package versioning and dependencies
- [ ] Hot-reload of packages during development
- [ ] Package documentation integration
- [ ] Example packages (diffgeo, quantum, etc.)

**Deliverable:** Full package ecosystem working.

---

## 17. Open Questions

### Q1: Placeholder Naming Conventions

Should we enforce naming conventions for placeholders?

**Options:**
- **Free-form:** Any name allowed (`{x}`, `{foo}`, `{thing}`)
- **Semantic:** Suggest patterns (`{left}`, `{right}`, `{arg}`, `{index}`)
- **Typed:** Encode type in name (`{scalar:x}`, `{vector:v}`)

**Recommendation:** Free-form with semantic suggestions.

### Q2: Partial Template Instantiation

Can user insert incomplete templates?

**Example:** Insert integral without bounds?
- `∫ f(x) dx` vs `∫ₐᵇ f(x) dx`

**Options:**
- Always require all placeholders (even if empty)
- Allow optional placeholders marked in template
- Provide template variants (integral vs integral_bounds)

**Recommendation:** Template variants for different cases.

### Q3: Template Override Behavior

When user template conflicts with core template?

**Options:**
- **Replace:** User version completely overrides
- **Extend:** User adds to core (e.g., new rendering target)
- **Warn:** Show warning, let user choose
- **Namespace:** User templates in separate namespace

**Recommendation:** Replace with warning if overriding core.

### Q4: Placeholder Type Inference

How aggressive should type inference be?

**Options:**
- **None:** Placeholders accept any expression
- **Hint-based:** Infer from placeholder name (`{index}` → Index type)
- **Signature-based:** Use operation type signature strictly
- **Progressive:** Start loose, tighten with user input

**Recommendation:** Hint-based initially, signature-based eventually.

### Q5: Template Versioning

How to handle template changes across versions?

**Example:** Package updates template format

**Options:**
- Break compatibility (users must update)
- Support multiple versions simultaneously
- Migration tools to auto-convert
- Lock template versions in projects

**Recommendation:** Semantic versioning + migration tools.

---

## 18. Success Metrics

### How to Know Implementation is Working

**Metric 1: Template Coverage**
- All 73 current operations have metadata extracted ✓
- All palette buttons generate correctly ✓
- All insertions create valid ASTs ✓

**Metric 2: User Workflow**
- Can build expression starting from empty slate ✓
- Can navigate placeholders with Tab ✓
- Can fill placeholders with text or templates ✓
- Can export complete LaTeX ✓

**Metric 3: Extensibility**
- Can define new operation in .kleis file ✓
- Template auto-generates palette button ✓
- Can use new operation in editor ✓
- Exports correctly to LaTeX ✓

**Metric 4: Package System**
- Can install external package ✓
- Package operations appear in palette ✓
- Can use package notation ✓
- Multiple packages coexist ✓

---

## 19. References

### Related Documents
- **ADR-006:** Template-Grammar Duality and Synchronization
- **ADR-009:** WYSIWYG Structural Editor
- **ADR-007:** Bootstrap Grammar and Self-Definition
- **kleis_vision_executable_math.md:** Overall vision

### Code Locations
- **src/ast.rs:** Expression type definition (add Placeholder here)
- **src/render.rs:** GlyphContext with templates (metadata extraction)
- **src/parser.rs:** LaTeX parser (fragment parsing for fills)
- **static/index.html:** Current web UI (extend for structural editing)

### Examples to Study
- **Gallery examples:** 73 operations already working
- **Test suite:** 223 tests showing expected behavior
- **PARSER_TODO.md:** Current parsing capabilities

---

## Conclusion

The template implementation strategy is straightforward because **the foundation is complete**. The template strings in `render.rs` already contain all needed information - we just need to:

1. **Extract** metadata from template strings (placeholder names, arity)
2. **Render** placeholders as interactive boxes instead of filled values
3. **Generate** palette buttons from template metadata
4. **Handle** user interactions to fill/navigate placeholders
5. **Load** user-defined templates at runtime

The hard part (parsing, rendering, AST) is done. This is pure UI/UX work building on a solid foundation.

**The revolutionary aspect:** When this works, users can define new mathematical notation and the editor adapts automatically. Mathematics notation becomes programmable, not fixed.

---

## 20. AST Traversal Strategies and Placeholder Lookup

### The Core Challenge

When a user clicks a placeholder in the rendered equation, we need to:
1. Extract the placeholder ID from the DOM element
2. **Locate that placeholder in the AST** ← This section
3. Replace it with a new expression

The question is: **How do we efficiently find a placeholder by ID in the AST?**

### Baseline Approach: Depth-First Search (DFS)

The simplest approach is to traverse the tree until we find the matching ID:

```rust
pub fn find_placeholder_path(expr: &Expression, target_id: usize) -> Option<Vec<usize>> {
    fn search(
        expr: &Expression, 
        target_id: usize, 
        path: &mut Vec<usize>
    ) -> bool {
        match expr {
            Expression::Placeholder { id, .. } => {
                *id == target_id  // Found it!
            }
            Expression::Operation { args, .. } => {
                // Visit each child
                for (i, arg) in args.iter().enumerate() {
                    path.push(i);  // Going down
                    if search(arg, target_id, path) {
                        return true;  // Found in subtree
                    }
                    path.pop();  // Backtrack
                }
                false
            }
            _ => false  // Const, Object - no placeholders here
        }
    }
    
    let mut path = Vec::new();
    if search(expr, target_id, &mut path) {
        Some(path)
    } else {
        None
    }
}
```

**Time Complexity:** O(n) where n = number of nodes in AST

### Is DFS Fast Enough?

**YES!** For typical mathematical equations:

```
Typical equation AST sizes:
- Simple: (a+b)/c → ~5 nodes
- Medium: ∫₀¹ x² dx → ~10 nodes  
- Complex: Maxwell equations → ~50 nodes
- Very complex: Full GR field equations → ~200 nodes

DFS on 200 nodes: < 1 microsecond on modern hardware
```

**Actual bottleneck is MathJax re-rendering:**
```
Timing breakdown per placeholder edit:
- Find placeholder in AST: < 0.001ms (DFS)
- Replace node: < 0.001ms
- Convert AST to LaTeX: ~0.1ms
- MathJax render: ~10-50ms ← THIS is 99% of the time!
- Attach click handlers: ~0.1ms

Total: ~10-50ms (dominated by MathJax)
```

**Conclusion:** DFS is perfectly fine for Kleis. Optimizing it saves microseconds when MathJax costs milliseconds.

### Strategy 1: Visitor Pattern (Avoid Borrow Checker Issues)

The visitor pattern provides clean traversal without fighting Rust's borrow checker:

```rust
pub trait ExpressionVisitor {
    type Output;
    
    fn visit_placeholder(&mut self, id: usize, hint: &str) -> Self::Output;
    fn visit_const(&mut self, value: &str) -> Self::Output;
    fn visit_object(&mut self, name: &str) -> Self::Output;
    fn visit_operation(&mut self, name: &str, args: &[Expression]) -> Self::Output;
}

impl Expression {
    pub fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Expression::Placeholder { id, hint } => 
                visitor.visit_placeholder(*id, hint),
            Expression::Const(v) => 
                visitor.visit_const(v),
            Expression::Object(n) => 
                visitor.visit_object(n),
            Expression::Operation { name, args } => 
                visitor.visit_operation(name, args),
        }
    }
}

// Finding placeholder becomes clean:
struct PlaceholderFinder {
    target_id: usize,
    path: Vec<usize>,
    found_path: Option<Vec<usize>>,
}

impl ExpressionVisitor for PlaceholderFinder {
    type Output = bool;
    
    fn visit_placeholder(&mut self, id: usize, _hint: &str) -> bool {
        if id == self.target_id {
            self.found_path = Some(self.path.clone());
            true
        } else {
            false
        }
    }
    
    fn visit_operation(&mut self, _name: &str, args: &[Expression]) -> bool {
        for (i, arg) in args.iter().enumerate() {
            self.path.push(i);
            if arg.accept(self) {
                return true;
            }
            self.path.pop();
        }
        false
    }
    
    fn visit_const(&mut self, _: &str) -> bool { false }
    fn visit_object(&mut self, _: &str) -> bool { false }
}

// Usage:
pub fn find_placeholder_path(expr: &Expression, id: usize) -> Option<Vec<usize>> {
    let mut finder = PlaceholderFinder {
        target_id: id,
        path: Vec::new(),
        found_path: None,
    };
    expr.accept(&mut finder);
    finder.found_path
}
```

**Advantages:**
- Clean separation of traversal logic
- Easy to add new visitors (count placeholders, collect IDs, etc.)
- No borrow checker fights
- Extensible pattern for other AST operations

**Still O(n), but with better code organization.**

### Strategy 2: Lazy Cache (Best Practical Approach)

Cache placeholder locations on first lookup, invalidate on mutation:

```rust
pub struct EditorState {
    ast: Expression,
    placeholder_cache: RefCell<HashMap<usize, Vec<usize>>>,
    cache_valid: Cell<bool>,
}

impl EditorState {
    pub fn new(ast: Expression) -> Self {
        Self {
            ast,
            placeholder_cache: RefCell::new(HashMap::new()),
            cache_valid: Cell::new(false),
        }
    }
    
    pub fn find_placeholder(&self, id: usize) -> Option<Vec<usize>> {
        // Check cache first
        if self.cache_valid.get() {
            if let Some(path) = self.placeholder_cache.borrow().get(&id) {
                return Some(path.clone());  // O(1) cache hit!
            }
        }
        
        // Cache miss - do DFS
        let path = find_placeholder_path(&self.ast, id)?;
        
        // Update cache
        self.placeholder_cache.borrow_mut().insert(id, path.clone());
        self.cache_valid.set(true);
        
        Some(path)
    }
    
    pub fn replace_placeholder(&mut self, id: usize, new_expr: Expression) -> Result<(), String> {
        // Get path (may use cache)
        let path = self.find_placeholder(id)
            .ok_or("Placeholder not found")?;
        
        // Replace in AST
        replace_at_path(&mut self.ast, &path, new_expr)?;
        
        // Invalidate cache
        self.cache_valid.set(false);
        
        Ok(())
    }
}
```

**Advantages:**
- First lookup: O(n) DFS
- Subsequent lookups: O(1) cache hit
- Simple invalidation on mutation
- Low memory overhead
- No complex pointer management

**Best of both worlds for typical usage patterns.**

### Strategy 3: Arena Allocation with HashMap (C-Style Pointers)

For those who want O(1) lookup with "real" pointers:

```rust
use typed_arena::Arena;
use std::collections::HashMap;

pub struct AstArena {
    arena: Arena<Expression>,
    placeholder_map: HashMap<usize, *const Expression>,
}

impl AstArena {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            placeholder_map: HashMap::new(),
        }
    }
    
    pub fn alloc(&mut self, expr: Expression) -> &mut Expression {
        let ptr = self.arena.alloc(expr);
        
        // Register all placeholders in this expression
        self.register_placeholders(ptr);
        
        ptr
    }
    
    fn register_placeholders(&mut self, expr: &Expression) {
        match expr {
            Expression::Placeholder { id, .. } => {
                self.placeholder_map.insert(*id, expr as *const _);
            }
            Expression::Operation { args, .. } => {
                for arg in args {
                    self.register_placeholders(arg);
                }
            }
            _ => {}
        }
    }
    
    pub fn find_placeholder(&self, id: usize) -> Option<&Expression> {
        self.placeholder_map.get(&id).map(|ptr| unsafe {
            // Safe because arena guarantees lifetime
            &**ptr
        })
    }
    
    pub fn find_placeholder_mut(&mut self, id: usize) -> Option<&mut Expression> {
        self.placeholder_map.get(&id).map(|ptr| unsafe {
            // Safe because we have &mut self (exclusive access)
            &mut *(*ptr as *mut Expression)
        })
    }
}

// Usage:
let mut arena = AstArena::new();

let expr = arena.alloc(Expression::Operation {
    name: "scalar_divide".to_string(),
    args: vec![
        Expression::Placeholder { id: 0, hint: "num".to_string() },
        Expression::Placeholder { id: 1, hint: "den".to_string() },
    ]
});

// O(1) lookup!
let placeholder = arena.find_placeholder(0).unwrap();
```

**Advantages:**
- True O(1) lookup
- No reference counting overhead
- Arena cleans up everything at once
- Efficient memory layout

**Disadvantages:**
- Unsafe code (requires careful reasoning)
- Can't move expressions between arenas
- Lifetime tied to arena
- More complex implementation

### Strategy 4: Rc + RefCell (Flexible Shared Ownership)

Use reference counting for maximum flexibility:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

pub enum Expression {
    Const(String),
    Object(String),
    Operation { 
        name: String, 
        args: Vec<Rc<RefCell<Expression>>> 
    },
    Placeholder { 
        id: usize, 
        hint: String 
    },
}

pub struct EditorState {
    ast: Rc<RefCell<Expression>>,
    placeholder_map: HashMap<usize, Weak<RefCell<Expression>>>,
}

impl EditorState {
    pub fn insert_operation(&mut self, op_name: &str, placeholder_ids: Vec<usize>) 
        -> Rc<RefCell<Expression>> 
    {
        let args: Vec<_> = placeholder_ids.iter().map(|id| {
            let ph = Rc::new(RefCell::new(Expression::Placeholder {
                id: *id,
                hint: "placeholder".to_string(),
            }));
            
            // Register in map
            self.placeholder_map.insert(*id, Rc::downgrade(&ph));
            
            ph
        }).collect();
        
        Rc::new(RefCell::new(Expression::Operation {
            name: op_name.to_string(),
            args,
        }))
    }
    
    pub fn find_placeholder(&self, id: usize) -> Option<Rc<RefCell<Expression>>> {
        self.placeholder_map.get(&id)
            .and_then(|weak| weak.upgrade())
    }
    
    pub fn replace_placeholder(&mut self, id: usize, new_expr: Expression) 
        -> Result<(), String> 
    {
        let placeholder_rc = self.find_placeholder(id)
            .ok_or("Placeholder not found")?;
        
        // Replace in-place
        *placeholder_rc.borrow_mut() = new_expr;
        
        // Remove from map
        self.placeholder_map.remove(&id);
        
        Ok(())
    }
}
```

**Advantages:**
- True O(1) lookup
- Can share nodes between trees
- Flexible mutations
- All safe Rust (no unsafe)

**Disadvantages:**
- RefCell runtime borrow checking overhead
- Reference counting overhead
- Weak pointers need upgrading
- More complex expression structure
- Harder to serialize/debug

### Strategy 5: SlotMap (Index-Based, Safe)

Use stable indices instead of pointers:

```rust
use slotmap::{SlotMap, DefaultKey};
use std::collections::HashMap;

pub struct EditorState {
    expressions: SlotMap<DefaultKey, Expression>,
    root: DefaultKey,
    placeholder_map: HashMap<usize, DefaultKey>,
}

pub enum Expression {
    Const(String),
    Object(String),
    Operation { 
        name: String, 
        args: Vec<DefaultKey>  // Indices instead of owned Expressions
    },
    Placeholder { 
        id: usize, 
        hint: String 
    },
}

impl EditorState {
    pub fn new() -> Self {
        let mut expressions = SlotMap::new();
        let root = expressions.insert(Expression::Placeholder { 
            id: 0, 
            hint: "root".to_string() 
        });
        
        let mut state = Self {
            expressions,
            root,
            placeholder_map: HashMap::new(),
        };
        
        state.placeholder_map.insert(0, root);
        state
    }
    
    pub fn insert_operation(&mut self, op_name: &str, placeholder_ids: Vec<usize>) 
        -> DefaultKey 
    {
        let arg_keys: Vec<_> = placeholder_ids.iter().map(|id| {
            let key = self.expressions.insert(Expression::Placeholder {
                id: *id,
                hint: "placeholder".to_string(),
            });
            self.placeholder_map.insert(*id, key);
            key
        }).collect();
        
        self.expressions.insert(Expression::Operation {
            name: op_name.to_string(),
            args: arg_keys,
        })
    }
    
    pub fn find_placeholder(&self, id: usize) -> Option<&Expression> {
        self.placeholder_map.get(&id)
            .and_then(|key| self.expressions.get(*key))
    }
    
    pub fn find_placeholder_mut(&mut self, id: usize) -> Option<&mut Expression> {
        self.placeholder_map.get(&id)
            .and_then(|key| self.expressions.get_mut(*key))
    }
    
    pub fn replace_placeholder(&mut self, id: usize, new_expr: Expression) 
        -> Result<(), String> 
    {
        let ph_key = self.placeholder_map.remove(&id)
            .ok_or("Placeholder not found")?;
        
        self.expressions[ph_key] = new_expr;
        
        Ok(())
    }
}
```

**Advantages:**
- O(1) lookup
- No unsafe code
- No RC overhead
- Safe deletions (keys auto-invalidate)
- Generational indices prevent use-after-free

**Disadvantages:**
- Changes Expression structure significantly
- All operations need SlotMap access
- More complex to serialize
- Less intuitive than tree structure

### Performance Comparison

| Approach | Lookup | Replace | Memory | Complexity | Rust-Friendly |
|----------|--------|---------|--------|------------|---------------|
| **DFS (naive)** | O(n) | O(n) | O(1) | Simple | ✓✓✓ |
| **Visitor + DFS** | O(n) | O(n) | O(1) | Medium | ✓✓✓ |
| **Lazy Cache** | O(1)* | O(n) | O(n) | Low | ✓✓✓ |
| **Arena + HashMap** | O(1) | O(1) | O(n) | High | ✓ (unsafe) |
| **Rc + RefCell** | O(1) | O(1) | O(n)+ | High | ✓✓ |
| **SlotMap** | O(1) | O(1) | O(n) | Medium | ✓✓✓ |

\* Amortized - first lookup is O(n), subsequent are O(1)  
\+ Additional RC overhead

### Recommendation for Kleis

**Start with: Visitor Pattern + Lazy Cache**

```rust
pub struct EditorState {
    ast: Expression,  // Keep simple tree structure
    placeholder_cache: RefCell<HashMap<usize, Vec<usize>>>,
    cache_valid: Cell<bool>,
}

impl EditorState {
    pub fn find_placeholder(&self, id: usize) -> Option<Vec<usize>> {
        // Use visitor for clean traversal
        // Cache result for repeated access
        // Simple invalidation on mutation
    }
}
```

**Why:**
1. **Simple tree structure** - Easy to serialize, clone, debug
2. **O(1) repeated lookups** - Cache handles common case
3. **No unsafe code** - All safe Rust
4. **Clean traversal** - Visitor pattern avoids borrow checker issues
5. **Low complexity** - Easy to understand and maintain
6. **Good enough** - Microsecond lookups vs millisecond rendering

**When to upgrade:**
- **If profiling shows DFS is bottleneck** (it won't) → Add eager cache rebuild
- **If need complex graph operations** → Consider SlotMap
- **If need shared subtrees** → Consider Rc + RefCell
- **If absolute max performance** → Consider Arena (with careful unsafe)

But for 99% of use cases, **Visitor + Lazy Cache is the sweet spot**.

### Code Organization

Suggested file structure:

```
src/
  ast.rs                  # Expression enum with Placeholder variant
  ast_visitor.rs          # Visitor trait and implementations
  placeholder_finder.rs   # PlaceholderFinder visitor
  editor_state.rs         # EditorState with lazy cache
  template_metadata.rs    # Template extraction and registry
  render.rs              # Existing renderer, add placeholder rendering
```

Keep concerns separated, start simple, optimize if needed (but you won't need to).

---

## 21. References and Further Reading

### Architecture Decision Records
- **[ADR-009: WYSIWYG Structural Editor](adr-009-wysiwyg-structural-editor.md)** - High-level architecture and vision (read this first!)
- **[ADR-006: Template-Grammar Duality](adr-006-template-grammar-duality.md)** - Core principle: template placeholders define grammar structure
- **[ADR-005: Visual Authoring](adr-005-visual-authoring.md)** - Long-term vision for visual math authoring
- **[ADR-004: Input Visualization](adr-004-input-visualization.md)** - Input strategy for complex expressions

### Implementation Targets
- **src/ast.rs** - Add `Placeholder` variant here
- **src/render.rs** - Modify to render placeholders with IDs
- **src/template_metadata.rs** - NEW - Extract metadata from templates
- **src/ast_visitor.rs** - NEW - Visitor pattern for traversal
- **src/editor_state.rs** - NEW - Editor state with lazy cache
- **static/index.html** - Modify for placeholder interaction

### External Resources
- **MathJax Documentation** - For understanding `\class{}`, `\bbox{}` commands
- **typed_arena crate** - If implementing arena-based approach
- **slotmap crate** - If implementing index-based approach

---

