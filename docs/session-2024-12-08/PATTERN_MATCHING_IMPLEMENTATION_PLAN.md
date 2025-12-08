# Pattern Matching Implementation Plan

**Date:** December 8, 2024  
**Status:** 📋 PLANNED (AST structures complete)  
**Estimated Time:** 4-6 hours total  
**Complexity:** Very High (complete language feature)

---

## Current Status

**What's Complete:** ✅
- AST structures (Expression::Match, MatchCase, Pattern)
- Helper methods for creating patterns
- Placeholder implementations in all match expressions
- All 315 tests still passing

**What's Next:**
- Parser for match syntax (Step 3) - ~2 hours
- Type inference for patterns (Step 4) - ~1-2 hours
- Pattern evaluation (Step 5) - ~1 hour
- Exhaustiveness checking (Step 6) - ~1-2 hours
- Comprehensive tests (Step 7) - ~1 hour

---

## Syntax Design (from ADR-021)

### Grammar

```ebnf
matchExpr ::= "match" expression "{" matchCases "}"
matchCases ::= matchCase ("|" matchCase)*
matchCase ::= pattern "=>" expression
pattern ::= wildcard | variable | constructor | constant
wildcard ::= "_"
variable ::= identifier
constructor ::= identifier [ "(" patterns ")" ]
patterns ::= pattern ("," pattern)*
constant ::= number | string
```

### Examples

```kleis
// Simple match on Bool
match condition {
  True => 1
  False => 0
}

// Match on Option with variable binding
match maybeValue {
  None => 0
  Some(x) => x
}

// Nested patterns
match result {
  Ok(Some(x)) => x
  Ok(None) => 0
  Err(_) => -1
}

// Wildcard
match status {
  Success => "good"
  _ => "bad"
}

// Multiple arguments
match pair {
  Pair(a, b) => a + b
}
```

---

## Step 3: Parser Implementation

### Overview

Parse `match` expressions into `Expression::Match` AST nodes.

### Implementation Location

**File:** `src/kleis_parser.rs`

**Integration point:** In `parse_primary()` method, add case for "match" keyword

### Detailed Implementation

```rust
impl KleisParser {
    fn parse_primary(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();
        
        // Check for match keyword
        if self.peek_word() == Some("match") {
            return self.parse_match_expr();
        }
        
        // ... existing code ...
    }
    
    /// Parse a match expression
    /// Grammar: match expr { case1 | case2 ... }
    fn parse_match_expr(&mut self) -> Result<Expression, KleisParseError> {
        // Consume 'match' keyword
        self.expect_word("match")?;
        self.skip_whitespace();
        
        // Parse scrutinee expression
        let scrutinee = self.parse_expression()?;
        self.skip_whitespace();
        
        // Expect opening brace
        self.expect_char('{')?;
        self.skip_whitespace();
        
        // Parse cases
        let cases = self.parse_match_cases()?;
        self.skip_whitespace();
        
        // Expect closing brace
        self.expect_char('}')?;
        
        Ok(Expression::match_expr(scrutinee, cases))
    }
    
    /// Parse match cases separated by '|' or newlines
    fn parse_match_cases(&mut self) -> Result<Vec<MatchCase>, KleisParseError> {
        let mut cases = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            // Check for closing brace
            if self.peek() == Some('}') {
                break;
            }
            
            // Parse one case
            let case = self.parse_match_case()?;
            cases.push(case);
            
            self.skip_whitespace();
            
            // Optional separator
            if self.peek() == Some('|') {
                self.advance();
            }
        }
        
        if cases.is_empty() {
            return Err(KleisParseError {
                message: "Match expression must have at least one case".to_string(),
                position: self.pos,
            });
        }
        
        Ok(cases)
    }
    
    /// Parse a single match case
    /// Grammar: pattern => expression
    fn parse_match_case(&mut self) -> Result<MatchCase, KleisParseError> {
        self.skip_whitespace();
        
        // Parse pattern
        let pattern = self.parse_pattern()?;
        self.skip_whitespace();
        
        // Expect =>
        if !self.consume_str("=>") {
            return Err(KleisParseError {
                message: "Expected '=>' after pattern".to_string(),
                position: self.pos,
            });
        }
        self.skip_whitespace();
        
        // Parse body expression
        let body = self.parse_expression()?;
        
        Ok(MatchCase::new(pattern, body))
    }
    
    /// Parse a pattern
    fn parse_pattern(&mut self) -> Result<Pattern, KleisParseError> {
        self.skip_whitespace();
        
        // Wildcard: _
        if self.peek() == Some('_') {
            self.advance();
            // Make sure it's just underscore (not part of identifier)
            if self.peek().map_or(true, |ch| !ch.is_alphanumeric()) {
                return Ok(Pattern::wildcard());
            }
            // Otherwise, fall through to identifier
            self.pos -= 1;
        }
        
        // Number constant
        if self.peek().map_or(false, |ch| ch.is_numeric()) {
            let num = self.parse_number()?;
            return Ok(Pattern::constant(num));
        }
        
        // Constructor or variable
        if self.peek().map_or(false, |ch| ch.is_alphabetic() || ch == '_') {
            let id = self.parse_identifier()?;
            self.skip_whitespace();
            
            // Constructor with arguments: Some(x)
            if self.peek() == Some('(') {
                self.advance();
                let args = self.parse_pattern_args()?;
                self.skip_whitespace();
                if self.advance() != Some(')') {
                    return Err(KleisParseError {
                        message: "Expected ')' after constructor patterns".to_string(),
                        position: self.pos,
                    });
                }
                return Ok(Pattern::constructor(id, args));
            }
            
            // Determine if it's a constructor or variable
            // Heuristic: Capitalized = constructor, lowercase = variable
            if id.chars().next().unwrap().is_uppercase() {
                // Constructor without arguments: None, True, False
                return Ok(Pattern::constructor(id, vec![]));
            } else {
                // Variable binding: x, value, result
                return Ok(Pattern::variable(id));
            }
        }
        
        Err(KleisParseError {
            message: "Expected pattern (wildcard, variable, constructor, or constant)".to_string(),
            position: self.pos,
        })
    }
    
    /// Parse pattern arguments separated by commas
    fn parse_pattern_args(&mut self) -> Result<Vec<Pattern>, KleisParseError> {
        let mut args = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            // Check for closing paren
            if self.peek() == Some(')') {
                break;
            }
            
            // Parse one pattern
            let pattern = self.parse_pattern()?;
            args.push(pattern);
            
            self.skip_whitespace();
            
            // Check for comma
            if self.peek() == Some(',') {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(args)
    }
    
    /// Helper: peek at the next word without consuming
    fn peek_word(&self) -> Option<&str> {
        let start = self.pos;
        let mut end = start;
        
        while end < self.input.len() {
            let ch = self.input[end];
            if ch.is_alphanumeric() || ch == '_' {
                end += 1;
            } else {
                break;
            }
        }
        
        if end > start {
            Some(&self.input[start..end].iter().collect::<String>())
        } else {
            None
        }
    }
    
    /// Helper: consume a specific word
    fn expect_word(&mut self, word: &str) -> Result<(), KleisParseError> {
        self.skip_whitespace();
        
        for expected_ch in word.chars() {
            if self.advance() != Some(expected_ch) {
                return Err(KleisParseError {
                    message: format!("Expected keyword '{}'", word),
                    position: self.pos - 1,
                });
            }
        }
        
        // Make sure word boundary (not part of longer identifier)
        if self.peek().map_or(false, |ch| ch.is_alphanumeric() || ch == '_') {
            return Err(KleisParseError {
                message: format!("Expected keyword '{}', got longer identifier", word),
                position: self.pos,
            });
        }
        
        Ok(())
    }
    
    /// Helper: consume a specific character
    fn expect_char(&mut self, expected: char) -> Result<(), KleisParseError> {
        self.skip_whitespace();
        if self.advance() != Some(expected) {
            return Err(KleisParseError {
                message: format!("Expected '{}'", expected),
                position: self.pos - 1,
            });
        }
        Ok(())
    }
    
    /// Helper: try to consume a string
    fn consume_str(&mut self, s: &str) -> bool {
        let start_pos = self.pos;
        
        for expected_ch in s.chars() {
            if self.advance() != Some(expected_ch) {
                self.pos = start_pos;
                return false;
            }
        }
        
        true
    }
}
```

### Test Plan for Parser

```rust
#[test]
fn test_parse_match_simple() {
    let code = r#"
        match x {
          True => 1
          False => 0
        }
    "#;
    
    let expr = parse_kleis(code).unwrap();
    
    match expr {
        Expression::Match { scrutinee, cases } => {
            assert_eq!(cases.len(), 2);
            // Verify patterns and bodies
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_match_with_bindings() {
    let code = r#"
        match myOption {
          None => 0
          Some(x) => x + 1
        }
    "#;
    
    // Verify Some(x) pattern parsed correctly
}

#[test]
fn test_parse_nested_patterns() {
    let code = r#"
        match result {
          Ok(Some(x)) => x
          Ok(None) => 0
          Err(_) => -1
        }
    "#;
    
    // Verify nested constructor patterns
}

#[test]
fn test_parse_wildcard() {
    let code = r#"
        match status {
          Success => 1
          _ => 0
        }
    "#;
    
    // Verify wildcard pattern
}
```

### Challenges

1. **Ambiguity:** Is `Some` a constructor or variable?
   - Solution: Capitalized = constructor, lowercase = variable

2. **Nested patterns:** `Ok(Some(x))` requires recursive parsing
   - Solution: `parse_pattern()` calls itself recursively

3. **Separator:** Cases separated by `|` or just newlines?
   - Recommendation: Support both for flexibility

4. **Empty match:** Should `match x {}` be allowed?
   - Recommendation: Require at least one case

---

## Step 4: Type Inference Implementation

### Overview

Check patterns match scrutinee type, bind pattern variables, unify branch types.

### Implementation Location

**File:** `src/type_inference.rs`

**Method:** Expand `infer_match()` stub

### Detailed Implementation

```rust
fn infer_match(
    &mut self,
    scrutinee: &Expression,
    cases: &[MatchCase],
    context_builder: Option<&TypeContextBuilder>,
) -> Result<Type, String> {
    // Step 1: Infer scrutinee type
    let scrutinee_ty = self.infer(scrutinee, context_builder)?;
    
    // Step 2: Infer each branch and collect result types
    let mut branch_types = Vec::new();
    
    for case in cases {
        // Create new context for this branch (pattern bindings are local)
        let saved_context = self.context.clone();
        
        // Check pattern and bind variables
        self.check_pattern(&case.pattern, &scrutinee_ty)?;
        
        // Infer body type with pattern bindings in scope
        let body_ty = self.infer(&case.body, context_builder)?;
        branch_types.push(body_ty);
        
        // Restore context (pattern bindings don't escape)
        self.context = saved_context;
    }
    
    // Step 3: Unify all branch types (must all have same type)
    if branch_types.is_empty() {
        return Err("Match expression must have at least one case".to_string());
    }
    
    let result_ty = branch_types[0].clone();
    for (i, branch_ty) in branch_types.iter().enumerate().skip(1) {
        self.unify(&result_ty, branch_ty)
            .map_err(|e| format!("Branch {} type mismatch: {}", i + 1, e))?;
    }
    
    Ok(result_ty)
}

/// Check pattern matches expected type and bind variables
fn check_pattern(&mut self, pattern: &Pattern, expected_ty: &Type) -> Result<(), String> {
    match pattern {
        Pattern::Wildcard => {
            // Wildcard matches anything
            Ok(())
        }
        
        Pattern::Variable(name) => {
            // Variable matches anything and gets bound to the type
            self.context.bind(name.clone(), expected_ty.clone());
            Ok(())
        }
        
        Pattern::Constructor { name, args } => {
            // Look up constructor in data registry
            let registry = self.data_registry();
            
            if let Some((type_name, variant)) = registry.lookup_variant(name) {
                // Check scrutinee type matches constructor's type
                match expected_ty {
                    Type::Data { type_name: scrutinee_type, .. } => {
                        if type_name != scrutinee_type {
                            return Err(format!(
                                "Pattern mismatch: constructor {} belongs to type {}, \
                                 but scrutinee has type {}",
                                name, type_name, scrutinee_type
                            ));
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Pattern mismatch: constructor {} expects data type, \
                             but scrutinee has type {:?}",
                            name, expected_ty
                        ));
                    }
                }
                
                // Check arity
                if variant.fields.len() != args.len() {
                    return Err(format!(
                        "Constructor {} expects {} arguments, got {}",
                        name,
                        variant.fields.len(),
                        args.len()
                    ));
                }
                
                // Recursively check nested patterns
                for (pattern_arg, field) in args.iter().zip(&variant.fields) {
                    // Get field type from variant definition
                    let field_ty = self.type_expr_to_type(&field.type_expr)?;
                    self.check_pattern(pattern_arg, &field_ty)?;
                }
                
                Ok(())
            } else {
                Err(format!("Unknown constructor: {}", name))
            }
        }
        
        Pattern::Constant(value) => {
            // Constant patterns must match primitive types
            // For now, assume constants are Scalars
            // TODO: Add proper constant type checking
            Ok(())
        }
    }
}

/// Helper: Convert TypeExpr to Type (simplified)
fn type_expr_to_type(&self, type_expr: &TypeExpr) -> Result<Type, String> {
    match type_expr {
        TypeExpr::Named(name) if name == "ℝ" => Ok(Type::scalar()),
        TypeExpr::Named(name) => {
            // Check if it's a user-defined type
            if self.data_registry().has_type(name) {
                Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args: vec![],
                })
            } else {
                Err(format!("Unknown type: {}", name))
            }
        }
        TypeExpr::Parametric(name, params) => {
            // TODO: Handle parametric types properly
            Err("Parametric types in patterns not yet supported".to_string())
        }
        _ => Err("Unsupported type expression in pattern".to_string()),
    }
}
```

### Test Plan for Type Inference

```rust
#[test]
fn test_match_type_inference_simple() {
    let mut checker = TypeChecker::new();
    checker.load_data_types("data Bool = True | False").unwrap();
    
    // match bool { True => 1 | False => 0 }
    // Should infer: Bool → Scalar
    let expr = Expression::match_expr(
        Expression::object("bool"),
        vec![
            MatchCase::new(Pattern::constructor("True", vec![]), Expression::constant("1")),
            MatchCase::new(Pattern::constructor("False", vec![]), Expression::constant("0")),
        ],
    );
    
    let result = checker.check(&expr);
    // Should infer Scalar
}

#[test]
fn test_match_variable_binding() {
    let mut checker = TypeChecker::new();
    checker.load_data_types("data Option(T) = None | Some(T)").unwrap();
    
    // match opt { None => 0 | Some(x) => x }
    // x should be bound to T
}

#[test]
fn test_match_branch_type_mismatch() {
    // match x { True => 1 | False => "string" }
    // Should error: branches have different types
}
```

---

## Step 5: Pattern Evaluation

### Overview

Runtime support for matching scrutinee values against patterns.

### Implementation Location

**File:** New module `src/pattern_matcher.rs` or extend `src/type_inference.rs`

### Detailed Implementation

```rust
pub struct PatternMatcher {
    data_registry: DataTypeRegistry,
}

impl PatternMatcher {
    pub fn new(data_registry: DataTypeRegistry) -> Self {
        PatternMatcher { data_registry }
    }
    
    /// Try to match a value against a pattern
    /// Returns bindings if match succeeds, None if fails
    pub fn match_pattern(
        &self,
        value: &Expression,
        pattern: &Pattern,
    ) -> Option<HashMap<String, Expression>> {
        let mut bindings = HashMap::new();
        
        if self.match_pattern_internal(value, pattern, &mut bindings) {
            Some(bindings)
        } else {
            None
        }
    }
    
    fn match_pattern_internal(
        &self,
        value: &Expression,
        pattern: &Pattern,
        bindings: &mut HashMap<String, Expression>,
    ) -> bool {
        match pattern {
            Pattern::Wildcard => {
                // Wildcard always matches
                true
            }
            
            Pattern::Variable(name) => {
                // Variable matches anything and binds the value
                bindings.insert(name.clone(), value.clone());
                true
            }
            
            Pattern::Constructor { name, args } => {
                // Check if value is a matching constructor
                match value {
                    Expression::Operation { 
                        name: value_name, 
                        args: value_args 
                    } if value_name == name => {
                        // Check arity
                        if value_args.len() != args.len() {
                            return false;
                        }
                        
                        // Recursively match arguments
                        for (val_arg, pat_arg) in value_args.iter().zip(args) {
                            if !self.match_pattern_internal(val_arg, pat_arg, bindings) {
                                return false;
                            }
                        }
                        
                        true
                    }
                    
                    Expression::Object(value_name) if value_name == name && args.is_empty() => {
                        // 0-arity constructor: True, False, None
                        true
                    }
                    
                    _ => false,
                }
            }
            
            Pattern::Constant(pattern_value) => {
                // Constant must match exactly
                match value {
                    Expression::Const(value_str) => value_str == pattern_value,
                    _ => false,
                }
            }
        }
    }
    
    /// Evaluate a match expression
    pub fn eval_match(
        &self,
        scrutinee: &Expression,
        cases: &[MatchCase],
    ) -> Result<Expression, String> {
        // Try each case in order
        for case in cases {
            if let Some(bindings) = self.match_pattern(scrutinee, &case.pattern) {
                // Found a match! Substitute bindings into body
                return Ok(self.substitute_bindings(&case.body, &bindings));
            }
        }
        
        // No case matched - non-exhaustive match at runtime
        Err("Non-exhaustive match: no pattern matched".to_string())
    }
    
    /// Substitute variable bindings into expression
    fn substitute_bindings(
        &self,
        expr: &Expression,
        bindings: &HashMap<String, Expression>,
    ) -> Expression {
        match expr {
            Expression::Object(name) => {
                if let Some(bound_value) = bindings.get(name) {
                    bound_value.clone()
                } else {
                    expr.clone()
                }
            }
            
            Expression::Operation { name, args } => {
                let substituted_args = args
                    .iter()
                    .map(|arg| self.substitute_bindings(arg, bindings))
                    .collect();
                Expression::operation(name.clone(), substituted_args)
            }
            
            Expression::Match { scrutinee, cases } => {
                let subst_scrutinee = self.substitute_bindings(scrutinee, bindings);
                let subst_cases = cases
                    .iter()
                    .map(|case| MatchCase {
                        pattern: case.pattern.clone(), // Patterns don't substitute
                        body: self.substitute_bindings(&case.body, bindings),
                    })
                    .collect();
                Expression::match_expr(subst_scrutinee, subst_cases)
            }
            
            _ => expr.clone(),
        }
    }
}
```

### Test Plan for Evaluation

```rust
#[test]
fn test_eval_match_bool() {
    let matcher = PatternMatcher::new(registry);
    
    // match True { True => 1 | False => 0 }
    let scrutinee = Expression::object("True");
    let cases = vec![
        MatchCase::new(Pattern::constructor("True", vec![]), Expression::constant("1")),
        MatchCase::new(Pattern::constructor("False", vec![]), Expression::constant("0")),
    ];
    
    let result = matcher.eval_match(&scrutinee, &cases).unwrap();
    assert_eq!(result, Expression::constant("1"));
}

#[test]
fn test_eval_match_with_binding() {
    // match Some(5) { None => 0 | Some(x) => x }
    // Should return 5 with x bound to 5
}

#[test]
fn test_eval_non_exhaustive_error() {
    // match Some(5) { None => 0 }
    // Should error: Some case not handled
}
```

---

## Step 6: Exhaustiveness Checking

### Overview

Verify all constructors are covered, warn on missing cases.

### Implementation Location

**File:** `src/pattern_matcher.rs` or `src/type_checker.rs`

### Detailed Implementation

```rust
pub struct ExhaustivenessChecker {
    data_registry: DataTypeRegistry,
}

impl ExhaustivenessChecker {
    /// Check if patterns are exhaustive for a type
    pub fn check_exhaustive(
        &self,
        patterns: &[Pattern],
        scrutinee_ty: &Type,
    ) -> Result<(), Vec<String>> {
        match scrutinee_ty {
            Type::Data { type_name, .. } => {
                // Get all constructors for this type
                if let Some(data_def) = self.data_registry.get_type(type_name) {
                    let all_constructors: HashSet<_> = 
                        data_def.variants.iter().map(|v| &v.name).collect();
                    
                    // Get covered constructors from patterns
                    let mut covered = HashSet::new();
                    let mut has_wildcard = false;
                    
                    for pattern in patterns {
                        match pattern {
                            Pattern::Wildcard | Pattern::Variable(_) => {
                                has_wildcard = true;
                            }
                            Pattern::Constructor { name, .. } => {
                                covered.insert(name);
                            }
                            Pattern::Constant(_) => {
                                // Constants don't contribute to exhaustiveness
                            }
                        }
                    }
                    
                    // If wildcard, automatically exhaustive
                    if has_wildcard {
                        return Ok(());
                    }
                    
                    // Check all constructors covered
                    let missing: Vec<_> = all_constructors
                        .difference(&covered)
                        .map(|s| s.to_string())
                        .collect();
                    
                    if missing.is_empty() {
                        Ok(())
                    } else {
                        Err(missing)
                    }
                } else {
                    // Unknown type - can't check exhaustiveness
                    Ok(())
                }
            }
            
            // Other types (Scalar, Var, etc.) - can't enumerate cases
            _ => Ok(()),
        }
    }
    
    /// Check for unreachable patterns
    pub fn check_reachable(&self, patterns: &[Pattern]) -> Vec<usize> {
        let mut unreachable = Vec::new();
        
        for (i, pattern) in patterns.iter().enumerate() {
            // Check if this pattern is shadowed by earlier patterns
            if i > 0 && self.is_subsumed(pattern, &patterns[..i]) {
                unreachable.push(i);
            }
        }
        
        unreachable
    }
    
    /// Check if pattern is subsumed by earlier patterns
    fn is_subsumed(&self, pattern: &Pattern, earlier: &[Pattern]) -> bool {
        for earlier_pattern in earlier {
            if self.pattern_subsumes(earlier_pattern, pattern) {
                return true;
            }
        }
        false
    }
    
    /// Check if pattern1 subsumes pattern2 (makes it unreachable)
    fn pattern_subsumes(&self, p1: &Pattern, p2: &Pattern) -> bool {
        match (p1, p2) {
            // Wildcard subsumes everything
            (Pattern::Wildcard, _) => true,
            (Pattern::Variable(_), _) => true,
            
            // Same constructor
            (Pattern::Constructor { name: n1, args: a1 }, 
             Pattern::Constructor { name: n2, args: a2 }) if n1 == n2 => {
                // All sub-patterns must be subsumed
                a1.iter().zip(a2).all(|(p1, p2)| self.pattern_subsumes(p1, p2))
            }
            
            // Same constant
            (Pattern::Constant(c1), Pattern::Constant(c2)) => c1 == c2,
            
            _ => false,
        }
    }
}
```

### Integration with Type Inference

```rust
// In infer_match():
// After inferring all branches, check exhaustiveness
let checker = ExhaustivenessChecker::new(self.data_registry.clone());
let patterns: Vec<_> = cases.iter().map(|c| &c.pattern).collect();

match checker.check_exhaustive(&patterns, &scrutinee_ty) {
    Ok(()) => {} // Exhaustive - good!
    Err(missing) => {
        eprintln!("Warning: Non-exhaustive match. Missing cases: {:?}", missing);
        // For now, just warn. Could be error in strict mode.
    }
}

// Check for unreachable patterns
let unreachable = checker.check_reachable(&patterns);
if !unreachable.is_empty() {
    eprintln!("Warning: Unreachable patterns at indices: {:?}", unreachable);
}
```

### Test Plan for Exhaustiveness

```rust
#[test]
fn test_exhaustiveness_all_covered() {
    // match bool { True => 1 | False => 0 }
    // ✅ Exhaustive - all constructors covered
}

#[test]
fn test_exhaustiveness_wildcard() {
    // match bool { True => 1 | _ => 0 }
    // ✅ Exhaustive - wildcard catches rest
}

#[test]
fn test_exhaustiveness_missing_case() {
    // match bool { True => 1 }
    // ⚠️ Non-exhaustive - False not covered
}

#[test]
fn test_unreachable_pattern() {
    // match bool { True => 1 | _ => 0 | False => 2 }
    //                                   ^^^^^^^^ unreachable!
}

#[test]
fn test_exhaustiveness_nested() {
    // match result {
    //   Ok(Some(x)) => x
    //   Ok(None) => 0
    //   Err(_) => -1
    // }
    // ✅ Exhaustive - all Result constructors handled
}
```

---

## Step 7: Comprehensive Test Suite

### Test Categories

**1. Parser Tests** (in `src/kleis_parser.rs`)
- Simple match expressions
- Nested patterns
- Wildcards and variables
- Constructor patterns
- Error cases (missing braces, no cases, etc.)

**2. Type Inference Tests** (in `tests/pattern_matching_test.rs`)
- Pattern type checking
- Variable binding
- Branch type unification
- Nested pattern inference
- Error cases (type mismatches)

**3. Evaluation Tests** (in `tests/pattern_matching_test.rs`)
- Matching succeeds/fails correctly
- Variable substitution
- Nested pattern matching
- Non-exhaustive runtime errors

**4. Exhaustiveness Tests** (in `tests/pattern_matching_test.rs`)
- Complete coverage detection
- Missing case warnings
- Unreachable pattern warnings
- Wildcard handling

**5. Integration Tests** (in `tests/end_to_end_tests.rs`)
- Bool pattern matching
- Option pattern matching
- Result pattern matching
- Custom user types

### New Test File Structure

```rust
///! Comprehensive pattern matching tests for ADR-021
use kleis::ast::{Expression, MatchCase, Pattern};
use kleis::type_checker::{TypeChecker, TypeCheckResult};
use kleis::type_inference::Type;

// === Parser Tests ===

#[test]
fn test_parse_simple_match() { ... }

#[test]
fn test_parse_nested_patterns() { ... }

#[test]
fn test_parse_wildcard() { ... }

// === Type Inference Tests ===

#[test]
fn test_bool_pattern_types() { ... }

#[test]
fn test_option_pattern_types() { ... }

#[test]
fn test_branch_type_unification() { ... }

// === Evaluation Tests ===

#[test]
fn test_eval_bool_match() { ... }

#[test]
fn test_eval_option_match() { ... }

#[test]
fn test_eval_nested_match() { ... }

// === Exhaustiveness Tests ===

#[test]
fn test_exhaustive_all_cases() { ... }

#[test]
fn test_non_exhaustive_warning() { ... }

#[test]
fn test_unreachable_pattern_warning() { ... }

// === Integration Tests ===

#[test]
fn test_pattern_match_in_operation() { ... }

#[test]
fn test_pattern_match_with_user_types() { ... }
```

---

## Implementation Order

### Recommended Sequence

**Tonight (if continuing):**
1. ✅ Parser implementation (Step 3) - ~2 hours
2. ✅ Parser tests - ~30 minutes
3. Commit and document progress

**Next Session:**
4. Type inference implementation (Step 4) - ~1-2 hours
5. Evaluation implementation (Step 5) - ~1 hour  
6. Exhaustiveness checking (Step 6) - ~1-2 hours
7. Comprehensive tests (Step 7) - ~1 hour

### Why Split This Way

**Parser first:**
- Self-contained (doesn't depend on evaluation)
- Can test syntax independently
- Good checkpoint for tonight

**Type inference + evaluation + exhaustiveness:**
- Interdependent (evaluation needs type info)
- Best done together in one focused session
- Requires fresh mental energy

---

## Edge Cases to Handle

### Parser Edge Cases

1. **Empty match:** `match x {}`
   - Decision: Error (require at least one case)

2. **No separator:** Cases on separate lines without `|`
   - Decision: Support both (newline-separated or pipe-separated)

3. **Nested braces:** `match x { Some(f) => { nested } }`
   - Decision: Parse inner braces as part of expression

4. **Constructor vs variable ambiguity:** `Some` vs `some`
   - Decision: Capitalized = constructor, lowercase = variable

### Type Inference Edge Cases

1. **Polymorphic scrutinee:** `match x { ... }` where x is type variable
   - Decision: Infer concrete type from patterns

2. **Wildcard with typed branches:** Branch types must still unify

3. **Unreachable patterns:** `True => 1 | _ => 0 | False => 2`
   - Decision: Warn but allow (False is unreachable)

### Evaluation Edge Cases

1. **Non-exhaustive at runtime:** No pattern matches
   - Decision: Runtime error with helpful message

2. **Nested binding collision:** `Some(Some(x))` where x appears twice
   - Decision: Inner binding shadows outer

3. **Constructor name conflicts:** User type vs built-in operation
   - Decision: Data registry takes precedence

---

## Dependencies

**Required:**
- ✅ DataTypeRegistry (have it!)
- ✅ Type::Data variant (have it!)
- ✅ Pattern AST (have it!)

**Nice to have:**
- Value representation (for runtime evaluation)
- Error reporting framework (for better messages)
- Type pretty-printing (for error messages)

---

## Success Criteria

### Parser (Step 3)
- ✅ Parses simple match expressions
- ✅ Parses nested patterns
- ✅ Parses wildcards and variables
- ✅ Error messages for malformed syntax
- ✅ At least 10 parser tests passing

### Type Inference (Step 4)
- ✅ Checks patterns match scrutinee type
- ✅ Binds pattern variables correctly
- ✅ Unifies all branch types
- ✅ Errors on type mismatches
- ✅ At least 10 type inference tests passing

### Evaluation (Step 5)
- ✅ Matches values against patterns
- ✅ Binds and substitutes variables
- ✅ Handles nested patterns
- ✅ Errors on non-exhaustive matches
- ✅ At least 10 evaluation tests passing

### Exhaustiveness (Step 6)
- ✅ Detects missing constructors
- ✅ Handles wildcards correctly
- ✅ Warns on unreachable patterns
- ✅ Works with nested patterns
- ✅ At least 5 exhaustiveness tests passing

---

## Estimated Effort

| Step | Description | Time | Complexity |
|------|-------------|------|------------|
| 3 | Parser | 2 hours | High |
| 4 | Type inference | 1-2 hours | Very High |
| 5 | Evaluation | 1 hour | Medium |
| 6 | Exhaustiveness | 1-2 hours | High |
| 7 | Tests | 1 hour | Medium |
| **Total** | **Full implementation** | **6-8 hours** | **Very High** |

---

## Alternative: Phased Approach

### Phase 1: Basic Pattern Matching (Tonight/Next Session)
- Parser for simple patterns (no nesting)
- Type inference for simple cases
- Basic evaluation
- **Goal:** Match on Bool, Option with simple patterns

### Phase 2: Advanced Features (Future Session)
- Nested patterns
- Exhaustiveness checking  
- Unreachability detection
- **Goal:** Production-ready pattern matching

### Phase 3: Optimization (Future)
- Pattern compilation
- Decision tree optimization
- Performance tuning

---

## Context Status

**Current:** 233K / 1000K tokens (23%)  
**After parser impl:** ~350K (estimated)  
**Comfortable limit:** 500K

**Recommendation:**
- ✅ Implement parser tonight (Step 3)
- 📋 Document current state
- ✅ Commit progress
- 🎯 Continue Steps 4-6 next session fresh

---

## Files to Create/Modify

### Tonight (Parser):
- `src/kleis_parser.rs` - Add match parsing methods
- `src/kleis_parser.rs` - Add parser tests

### Next Session (Type Inference + Evaluation):
- `src/type_inference.rs` - Expand infer_match()
- `src/pattern_matcher.rs` - NEW file for evaluation
- `tests/pattern_matching_test.rs` - NEW comprehensive test file

---

## Related ADRs and Docs

- **ADR-021:** Algebraic Data Types (pattern matching is Part 2)
- **ADR-014:** Hindley-Milner (pattern type checking uses HM)
- **Grammar v0.4:** Includes data types, pattern matching not yet added

---

**Document Status:** Complete implementation plan ready  
**Next Action:** Implement parser (Step 3) - start with `parse_match_expr()`  
**Estimated time tonight:** 2-2.5 hours for parser + tests

