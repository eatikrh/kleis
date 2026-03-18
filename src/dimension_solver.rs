//! Dimension Solver for Type-Level Arithmetic
//!
//! This module provides constraint solving for dimension expressions used in
//! parametric types like `Matrix(2*n, 2*n, ℝ)`.
//!
//! ## Supported Operations
//!
//! | Category | Operations | Examples |
//! |----------|------------|----------|
//! | Arithmetic | `+`, `-`, `*`, `/` | `n+1`, `2*n`, `n/2` |
//! | Power | `^` | `n^2`, `2^k` |
//! | Functions | `min`, `max` | `min(m,n)` |
//!
//! ## Design
//!
//! The solver uses a simple constraint-based approach:
//! 1. Check structural equality first (fast path)
//! 2. Simplify expressions by evaluating constants
//! 3. Attempt to solve for dimension variables
//!
//! For unsolvable cases, returns a clear error message.

use crate::kleis_ast::DimExpr;
use std::collections::HashMap;

/// Result of dimension unification
#[derive(Debug, Clone, PartialEq)]
pub enum DimUnifyResult {
    /// Expressions are equal (or can be made equal with substitution)
    Equal(DimSubstitution),
    /// Expressions cannot be unified
    Unequal(String),
}

/// Substitution for dimension variables
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DimSubstitution {
    map: HashMap<String, DimExpr>,
}

impl DimSubstitution {
    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn singleton(var: String, expr: DimExpr) -> Self {
        let mut map = HashMap::new();
        map.insert(var, expr);
        Self { map }
    }

    /// Apply substitution to a dimension expression
    pub fn apply(&self, expr: &DimExpr) -> DimExpr {
        match expr {
            DimExpr::Lit(n) => DimExpr::Lit(*n),
            DimExpr::Var(name) => {
                if let Some(replacement) = self.map.get(name) {
                    self.apply(replacement)
                } else {
                    DimExpr::Var(name.clone())
                }
            }
            DimExpr::Add(l, r) => DimExpr::Add(Box::new(self.apply(l)), Box::new(self.apply(r))),
            DimExpr::Sub(l, r) => DimExpr::Sub(Box::new(self.apply(l)), Box::new(self.apply(r))),
            DimExpr::Mul(l, r) => DimExpr::Mul(Box::new(self.apply(l)), Box::new(self.apply(r))),
            DimExpr::Div(l, r) => DimExpr::Div(Box::new(self.apply(l)), Box::new(self.apply(r))),
            DimExpr::Pow(l, r) => DimExpr::Pow(Box::new(self.apply(l)), Box::new(self.apply(r))),
            DimExpr::Call(name, args) => {
                let new_args: Vec<_> = args.iter().map(|a| self.apply(a)).collect();
                DimExpr::Call(name.clone(), new_args)
            }
        }
    }

    /// Compose two substitutions
    pub fn compose(&self, other: &DimSubstitution) -> DimSubstitution {
        let mut map = self.map.clone();
        for (var, expr) in &other.map {
            map.insert(var.clone(), self.apply(expr));
        }
        DimSubstitution { map }
    }
}

/// Unify two dimension expressions
///
/// Returns `Equal` with a substitution if they can be unified,
/// or `Unequal` with an error message if not.
pub fn unify_dims(e1: &DimExpr, e2: &DimExpr) -> DimUnifyResult {
    // Fast path: structural equality
    if e1 == e2 {
        return DimUnifyResult::Equal(DimSubstitution::empty());
    }

    // Simplify both expressions
    let s1 = simplify(e1);
    let s2 = simplify(e2);

    // Check again after simplification
    if s1 == s2 {
        return DimUnifyResult::Equal(DimSubstitution::empty());
    }

    // Try to solve
    match (&s1, &s2) {
        // Variable unifies with anything (if no occurs check failure)
        (DimExpr::Var(v), expr) | (expr, DimExpr::Var(v)) => {
            if occurs_in_dim(v, expr) {
                DimUnifyResult::Unequal(format!(
                    "Occurs check failed: {} occurs in {}",
                    v,
                    format_dim(expr)
                ))
            } else {
                DimUnifyResult::Equal(DimSubstitution::singleton(v.clone(), expr.clone()))
            }
        }

        // Literal vs literal (we already checked equality)
        (DimExpr::Lit(n1), DimExpr::Lit(n2)) => {
            DimUnifyResult::Unequal(format!("Cannot unify dimensions {} and {}", n1, n2))
        }

        // Try to solve linear equations
        (DimExpr::Mul(coef, var), DimExpr::Lit(n)) | (DimExpr::Lit(n), DimExpr::Mul(coef, var)) => {
            // Pattern: c * x = n  =>  x = n / c
            if let (DimExpr::Lit(c), DimExpr::Var(v)) = (coef.as_ref(), var.as_ref()) {
                if *c != 0 && n % c == 0 {
                    return DimUnifyResult::Equal(DimSubstitution::singleton(
                        v.clone(),
                        DimExpr::Lit(n / c),
                    ));
                }
            }
            // Try the other order: x * c = n
            if let (DimExpr::Var(v), DimExpr::Lit(c)) = (coef.as_ref(), var.as_ref()) {
                if *c != 0 && n % c == 0 {
                    return DimUnifyResult::Equal(DimSubstitution::singleton(
                        v.clone(),
                        DimExpr::Lit(n / c),
                    ));
                }
            }
            DimUnifyResult::Unequal(format!(
                "Cannot solve: {} = {}",
                format_dim(&s1),
                format_dim(&s2)
            ))
        }

        // Try to solve: x + c = n  =>  x = n - c
        (DimExpr::Add(var, delta), DimExpr::Lit(n))
        | (DimExpr::Lit(n), DimExpr::Add(var, delta)) => {
            if let (DimExpr::Var(v), DimExpr::Lit(d)) = (var.as_ref(), delta.as_ref()) {
                if *n >= *d {
                    return DimUnifyResult::Equal(DimSubstitution::singleton(
                        v.clone(),
                        DimExpr::Lit(n - d),
                    ));
                }
            }
            // Try: c + x = n
            if let (DimExpr::Lit(d), DimExpr::Var(v)) = (var.as_ref(), delta.as_ref()) {
                if *n >= *d {
                    return DimUnifyResult::Equal(DimSubstitution::singleton(
                        v.clone(),
                        DimExpr::Lit(n - d),
                    ));
                }
            }
            DimUnifyResult::Unequal(format!(
                "Cannot solve: {} = {}",
                format_dim(&s1),
                format_dim(&s2)
            ))
        }

        // Try to solve: x^c = n (for small c)
        (DimExpr::Pow(base, exp), DimExpr::Lit(n)) | (DimExpr::Lit(n), DimExpr::Pow(base, exp)) => {
            if let (DimExpr::Var(v), DimExpr::Lit(e)) = (base.as_ref(), exp.as_ref()) {
                if *e == 2 {
                    // x^2 = n => x = sqrt(n) if perfect square
                    let root = (*n as f64).sqrt() as usize;
                    if root * root == *n {
                        return DimUnifyResult::Equal(DimSubstitution::singleton(
                            v.clone(),
                            DimExpr::Lit(root),
                        ));
                    }
                }
                if *e == 3 {
                    // x^3 = n => x = cbrt(n) if perfect cube
                    let root = (*n as f64).cbrt().round() as usize;
                    if root * root * root == *n {
                        return DimUnifyResult::Equal(DimSubstitution::singleton(
                            v.clone(),
                            DimExpr::Lit(root),
                        ));
                    }
                }
            }
            // Try c^x = n (exponential)
            if let (DimExpr::Lit(c), DimExpr::Var(v)) = (base.as_ref(), exp.as_ref()) {
                if *c > 1 {
                    // Find x such that c^x = n
                    let mut power = 1usize;
                    let mut x = 0usize;
                    while power < *n {
                        power *= c;
                        x += 1;
                    }
                    if power == *n {
                        return DimUnifyResult::Equal(DimSubstitution::singleton(
                            v.clone(),
                            DimExpr::Lit(x),
                        ));
                    }
                }
            }
            DimUnifyResult::Unequal(format!(
                "Cannot solve: {} = {}",
                format_dim(&s1),
                format_dim(&s2)
            ))
        }

        // Structural comparison for compound expressions
        (DimExpr::Add(l1, r1), DimExpr::Add(l2, r2))
        | (DimExpr::Sub(l1, r1), DimExpr::Sub(l2, r2))
        | (DimExpr::Mul(l1, r1), DimExpr::Mul(l2, r2))
        | (DimExpr::Div(l1, r1), DimExpr::Div(l2, r2))
        | (DimExpr::Pow(l1, r1), DimExpr::Pow(l2, r2)) => {
            // Unify left sides first
            match unify_dims(l1, l2) {
                DimUnifyResult::Equal(subst1) => {
                    // Apply substitution and unify right sides
                    let r1_sub = subst1.apply(r1);
                    let r2_sub = subst1.apply(r2);
                    match unify_dims(&r1_sub, &r2_sub) {
                        DimUnifyResult::Equal(subst2) => {
                            DimUnifyResult::Equal(subst1.compose(&subst2))
                        }
                        err => err,
                    }
                }
                err => err,
            }
        }

        // Function calls: must have same name and arity
        (DimExpr::Call(n1, args1), DimExpr::Call(n2, args2)) => {
            if n1 != n2 || args1.len() != args2.len() {
                return DimUnifyResult::Unequal(format!(
                    "Cannot unify {}({} args) with {}({} args)",
                    n1,
                    args1.len(),
                    n2,
                    args2.len()
                ));
            }
            let mut subst = DimSubstitution::empty();
            for (a1, a2) in args1.iter().zip(args2.iter()) {
                let a1_sub = subst.apply(a1);
                let a2_sub = subst.apply(a2);
                match unify_dims(&a1_sub, &a2_sub) {
                    DimUnifyResult::Equal(s) => {
                        subst = subst.compose(&s);
                    }
                    err => return err,
                }
            }
            DimUnifyResult::Equal(subst)
        }

        // Default: cannot unify
        _ => DimUnifyResult::Unequal(format!(
            "Cannot unify dimensions: {} vs {}",
            format_dim(&s1),
            format_dim(&s2)
        )),
    }
}

/// Check if a dimension is definitely concrete (evaluates to a number)
pub fn is_concrete(expr: &DimExpr) -> bool {
    match expr {
        DimExpr::Lit(_) => true,
        DimExpr::Var(_) => false,
        DimExpr::Add(l, r)
        | DimExpr::Sub(l, r)
        | DimExpr::Mul(l, r)
        | DimExpr::Div(l, r)
        | DimExpr::Pow(l, r) => is_concrete(l) && is_concrete(r),
        DimExpr::Call(_, args) => args.iter().all(is_concrete),
    }
}

/// Evaluate a concrete dimension expression
pub fn evaluate(expr: &DimExpr) -> Option<usize> {
    match expr {
        DimExpr::Lit(n) => Some(*n),
        DimExpr::Var(_) => None,
        DimExpr::Add(l, r) => Some(evaluate(l)? + evaluate(r)?),
        DimExpr::Sub(l, r) => {
            let lv = evaluate(l)?;
            let rv = evaluate(r)?;
            lv.checked_sub(rv)
        }
        DimExpr::Mul(l, r) => Some(evaluate(l)? * evaluate(r)?),
        DimExpr::Div(l, r) => {
            let rv = evaluate(r)?;
            if rv == 0 {
                None
            } else {
                Some(evaluate(l)? / rv)
            }
        }
        DimExpr::Pow(l, r) => {
            let base = evaluate(l)?;
            let exp = evaluate(r)?;
            Some(base.pow(exp as u32))
        }
        DimExpr::Call(name, args) => {
            let vals: Option<Vec<_>> = args.iter().map(evaluate).collect();
            let vals = vals?;
            match name.as_str() {
                "min" if vals.len() == 2 => Some(vals[0].min(vals[1])),
                "max" if vals.len() == 2 => Some(vals[0].max(vals[1])),
                "gcd" if vals.len() == 2 => Some(gcd(vals[0], vals[1])),
                "lcm" if vals.len() == 2 => Some(lcm(vals[0], vals[1])),
                _ => None,
            }
        }
    }
}

/// Simplify a dimension expression by evaluating constants
pub fn simplify(expr: &DimExpr) -> DimExpr {
    // If fully concrete, evaluate to literal
    if let Some(n) = evaluate(expr) {
        return DimExpr::Lit(n);
    }

    // Otherwise, simplify recursively
    match expr {
        DimExpr::Lit(n) => DimExpr::Lit(*n),
        DimExpr::Var(v) => DimExpr::Var(v.clone()),
        DimExpr::Add(l, r) => {
            let sl = simplify(l);
            let sr = simplify(r);
            // 0 + x = x, x + 0 = x
            match (&sl, &sr) {
                (DimExpr::Lit(0), _) => sr,
                (_, DimExpr::Lit(0)) => sl,
                (DimExpr::Lit(a), DimExpr::Lit(b)) => DimExpr::Lit(a + b),
                _ => DimExpr::Add(Box::new(sl), Box::new(sr)),
            }
        }
        DimExpr::Sub(l, r) => {
            let sl = simplify(l);
            let sr = simplify(r);
            // x - 0 = x
            match (&sl, &sr) {
                (_, DimExpr::Lit(0)) => sl,
                (DimExpr::Lit(a), DimExpr::Lit(b)) if a >= b => DimExpr::Lit(a - b),
                _ => DimExpr::Sub(Box::new(sl), Box::new(sr)),
            }
        }
        DimExpr::Mul(l, r) => {
            let sl = simplify(l);
            let sr = simplify(r);
            // 0 * x = 0, 1 * x = x, x * 1 = x
            match (&sl, &sr) {
                (DimExpr::Lit(0), _) | (_, DimExpr::Lit(0)) => DimExpr::Lit(0),
                (DimExpr::Lit(1), _) => sr,
                (_, DimExpr::Lit(1)) => sl,
                (DimExpr::Lit(a), DimExpr::Lit(b)) => DimExpr::Lit(a * b),
                _ => DimExpr::Mul(Box::new(sl), Box::new(sr)),
            }
        }
        DimExpr::Div(l, r) => {
            let sl = simplify(l);
            let sr = simplify(r);
            // 0 / x = 0, x / 1 = x
            match (&sl, &sr) {
                (DimExpr::Lit(0), _) => DimExpr::Lit(0),
                (_, DimExpr::Lit(1)) => sl,
                (DimExpr::Lit(a), DimExpr::Lit(b)) if *b != 0 => DimExpr::Lit(a / b),
                _ => DimExpr::Div(Box::new(sl), Box::new(sr)),
            }
        }
        DimExpr::Pow(l, r) => {
            let sl = simplify(l);
            let sr = simplify(r);
            // x^0 = 1, x^1 = x, 0^n = 0 (n>0), 1^n = 1
            match (&sl, &sr) {
                (_, DimExpr::Lit(0)) => DimExpr::Lit(1),
                (_, DimExpr::Lit(1)) => sl,
                (DimExpr::Lit(0), _) => DimExpr::Lit(0),
                (DimExpr::Lit(1), _) => DimExpr::Lit(1),
                (DimExpr::Lit(a), DimExpr::Lit(b)) => DimExpr::Lit(a.pow(*b as u32)),
                _ => DimExpr::Pow(Box::new(sl), Box::new(sr)),
            }
        }
        DimExpr::Call(name, args) => {
            let sargs: Vec<_> = args.iter().map(simplify).collect();
            // Try to evaluate if all args are concrete
            if sargs.iter().all(|a| matches!(a, DimExpr::Lit(_))) {
                if let Some(n) = evaluate(&DimExpr::Call(name.clone(), sargs.clone())) {
                    return DimExpr::Lit(n);
                }
            }
            DimExpr::Call(name.clone(), sargs)
        }
    }
}

/// Check if variable occurs in expression (for occurs check)
fn occurs_in_dim(var: &str, expr: &DimExpr) -> bool {
    match expr {
        DimExpr::Lit(_) => false,
        DimExpr::Var(v) => v == var,
        DimExpr::Add(l, r)
        | DimExpr::Sub(l, r)
        | DimExpr::Mul(l, r)
        | DimExpr::Div(l, r)
        | DimExpr::Pow(l, r) => occurs_in_dim(var, l) || occurs_in_dim(var, r),
        DimExpr::Call(_, args) => args.iter().any(|a| occurs_in_dim(var, a)),
    }
}

/// Format dimension expression for display
pub fn format_dim(expr: &DimExpr) -> String {
    match expr {
        DimExpr::Lit(n) => n.to_string(),
        DimExpr::Var(v) => v.clone(),
        DimExpr::Add(l, r) => format!("({}+{})", format_dim(l), format_dim(r)),
        DimExpr::Sub(l, r) => format!("({}-{})", format_dim(l), format_dim(r)),
        DimExpr::Mul(l, r) => format!("({}*{})", format_dim(l), format_dim(r)),
        DimExpr::Div(l, r) => format!("({}/{})", format_dim(l), format_dim(r)),
        DimExpr::Pow(l, r) => format!("({}^{})", format_dim(l), format_dim(r)),
        DimExpr::Call(name, args) => {
            let arg_strs: Vec<_> = args.iter().map(format_dim).collect();
            format!("{}({})", name, arg_strs.join(", "))
        }
    }
}

/// Greatest common divisor
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Least common multiple
fn lcm(a: usize, b: usize) -> usize {
    if a == 0 || b == 0 {
        0
    } else {
        a / gcd(a, b) * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(n: usize) -> DimExpr {
        DimExpr::Lit(n)
    }

    fn var(s: &str) -> DimExpr {
        DimExpr::Var(s.to_string())
    }

    fn mul(l: DimExpr, r: DimExpr) -> DimExpr {
        DimExpr::Mul(Box::new(l), Box::new(r))
    }

    fn add(l: DimExpr, r: DimExpr) -> DimExpr {
        DimExpr::Add(Box::new(l), Box::new(r))
    }

    fn pow(l: DimExpr, r: DimExpr) -> DimExpr {
        DimExpr::Pow(Box::new(l), Box::new(r))
    }

    #[test]
    fn test_structural_equality() {
        let e1 = mul(lit(2), var("n"));
        let e2 = mul(lit(2), var("n"));
        assert_eq!(
            unify_dims(&e1, &e2),
            DimUnifyResult::Equal(DimSubstitution::empty())
        );
    }

    #[test]
    fn test_literal_equality() {
        assert_eq!(
            unify_dims(&lit(3), &lit(3)),
            DimUnifyResult::Equal(DimSubstitution::empty())
        );
    }

    #[test]
    fn test_literal_inequality() {
        match unify_dims(&lit(2), &lit(3)) {
            DimUnifyResult::Unequal(_) => {}
            _ => panic!("Expected unequal"),
        }
    }

    #[test]
    fn test_variable_unification() {
        // n = 5 => {n -> 5}
        match unify_dims(&var("n"), &lit(5)) {
            DimUnifyResult::Equal(subst) => {
                assert_eq!(subst.apply(&var("n")), lit(5));
            }
            _ => panic!("Expected equal"),
        }
    }

    #[test]
    fn test_linear_solve_multiply() {
        // 2*n = 6 => n = 3
        let e1 = mul(lit(2), var("n"));
        let e2 = lit(6);
        match unify_dims(&e1, &e2) {
            DimUnifyResult::Equal(subst) => {
                assert_eq!(subst.apply(&var("n")), lit(3));
            }
            _ => panic!("Expected equal"),
        }
    }

    #[test]
    fn test_linear_solve_add() {
        // n + 1 = 5 => n = 4
        let e1 = add(var("n"), lit(1));
        let e2 = lit(5);
        match unify_dims(&e1, &e2) {
            DimUnifyResult::Equal(subst) => {
                assert_eq!(subst.apply(&var("n")), lit(4));
            }
            _ => panic!("Expected equal"),
        }
    }

    #[test]
    fn test_power_solve_square() {
        // n^2 = 9 => n = 3
        let e1 = pow(var("n"), lit(2));
        let e2 = lit(9);
        match unify_dims(&e1, &e2) {
            DimUnifyResult::Equal(subst) => {
                assert_eq!(subst.apply(&var("n")), lit(3));
            }
            _ => panic!("Expected equal"),
        }
    }

    #[test]
    fn test_power_solve_exp() {
        // 2^k = 8 => k = 3
        let e1 = pow(lit(2), var("k"));
        let e2 = lit(8);
        match unify_dims(&e1, &e2) {
            DimUnifyResult::Equal(subst) => {
                assert_eq!(subst.apply(&var("k")), lit(3));
            }
            _ => panic!("Expected equal"),
        }
    }

    #[test]
    fn test_simplify() {
        // 2 * 3 simplifies to 6
        assert_eq!(simplify(&mul(lit(2), lit(3))), lit(6));

        // n * 1 simplifies to n
        assert_eq!(simplify(&mul(var("n"), lit(1))), var("n"));

        // 0 + n simplifies to n
        assert_eq!(simplify(&add(lit(0), var("n"))), var("n"));
    }

    #[test]
    fn test_different_structures_fail() {
        // 2*n cannot unify with n (unless n = 0)
        let e1 = mul(lit(2), var("n"));
        let e2 = var("n");
        // This should unify with n = 0
        match unify_dims(&e1, &e2) {
            DimUnifyResult::Equal(subst) => {
                // n must be 0 for 2*n = n
                let simplified = simplify(&subst.apply(&e1));
                let expected = simplify(&subst.apply(&e2));
                assert_eq!(simplified, expected);
            }
            DimUnifyResult::Unequal(_) => {
                // Also acceptable - depends on solver sophistication
            }
        }
    }

    #[test]
    fn test_evaluate_min_max() {
        let e = DimExpr::Call("min".to_string(), vec![lit(3), lit(5)]);
        assert_eq!(evaluate(&e), Some(3));

        let e = DimExpr::Call("max".to_string(), vec![lit(3), lit(5)]);
        assert_eq!(evaluate(&e), Some(5));
    }
}
