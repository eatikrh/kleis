// ============================================================================
// KLEIS SOLVER API
// ============================================================================
//
// Z3-backed solver for symbolic constraints.
//
// Usage in generated code:
//   use kleis_runtime::{Sym, SymReal, KleisSolver};
//   let solver = KleisSolver::new();
//   let result = solver.check_sat(&constraint);
//
// ============================================================================

use crate::symbolic::{Sym, SymBool, SymReal};
use std::collections::HashMap;
use z3::ast::{Ast, Bool as Z3Bool, Real as Z3Real};
use z3::{Config, Context, SatResult as Z3SatResult, Solver};

// ============================================================================
// RESULT TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum SatResult {
    Sat,
    Unsat,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerifyResult {
    Valid,
    Invalid,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub enum Witness {
    NoWitness,
    Model(Vec<Binding>),
}

// ============================================================================
// KLEIS SOLVER
// ============================================================================

/// The main solver interface for Kleis generated code
pub struct KleisSolver {
    cfg: Config,
}

impl KleisSolver {
    pub fn new() -> Self {
        KleisSolver { cfg: Config::new() }
    }

    /// Check if a constraint is satisfiable
    pub fn check_sat(&self, constraint: &SymBool) -> SatResult {
        let ctx = Context::new(&self.cfg);
        let solver = Solver::new(&ctx);
        let mut vars = HashMap::new();

        let z3_expr = translate_bool(&ctx, constraint, &mut vars);
        solver.assert(&z3_expr);

        match solver.check() {
            Z3SatResult::Sat => SatResult::Sat,
            Z3SatResult::Unsat => SatResult::Unsat,
            Z3SatResult::Unknown => SatResult::Unknown,
        }
    }

    /// Check if a constraint is valid (always true)
    pub fn check_valid(&self, constraint: &SymBool) -> VerifyResult {
        let negated = !constraint.clone();
        match self.check_sat(&negated) {
            SatResult::Unsat => VerifyResult::Valid,
            SatResult::Sat => VerifyResult::Invalid,
            SatResult::Unknown => VerifyResult::Timeout,
        }
    }

    /// Get a model (witness) if satisfiable
    pub fn get_model(&self, constraint: &SymBool) -> Witness {
        let ctx = Context::new(&self.cfg);
        let solver = Solver::new(&ctx);
        let mut vars = HashMap::new();

        let z3_expr = translate_bool(&ctx, constraint, &mut vars);
        solver.assert(&z3_expr);

        if solver.check() == Z3SatResult::Sat {
            if let Some(model) = solver.get_model() {
                let bindings: Vec<Binding> = vars
                    .iter()
                    .filter_map(|(name, var)| {
                        model.eval(var, true).and_then(|val| {
                            val.as_real().map(|(num, den)| Binding {
                                name: name.clone(),
                                value: num as f64 / den as f64,
                            })
                        })
                    })
                    .collect();
                return Witness::Model(bindings);
            }
        }
        Witness::NoWitness
    }

    /// Solve for variables: find values that satisfy the constraint
    pub fn solve(&self, constraint: &SymBool) -> Option<HashMap<String, f64>> {
        match self.get_model(constraint) {
            Witness::Model(bindings) => {
                let mut result = HashMap::new();
                for b in bindings {
                    result.insert(b.name, b.value);
                }
                Some(result)
            }
            Witness::NoWitness => None,
        }
    }

    /// Solve for a specific symbolic expression
    pub fn solve_for(&self, expr: &SymReal, target: f64) -> Option<HashMap<String, f64>> {
        let constraint = expr.clone().sym_eq(Sym::concrete(target));
        self.solve(&constraint)
    }
}

impl Default for KleisSolver {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Z3 TRANSLATION
// ============================================================================

fn translate_real<'ctx>(
    ctx: &'ctx Context,
    expr: &SymReal,
    vars: &mut HashMap<String, Z3Real<'ctx>>,
) -> Z3Real<'ctx> {
    match expr {
        Sym::Concrete(v) => {
            let (num, den) = float_to_rational(*v);
            Z3Real::from_real(ctx, num, den)
        }
        Sym::Variable(name) => vars
            .entry(name.clone())
            .or_insert_with(|| Z3Real::new_const(ctx, name.as_str()))
            .clone(),
        Sym::Expr { op, args } => match (op.as_str(), args.as_slice()) {
            ("add", [a, b]) => translate_real(ctx, a, vars) + translate_real(ctx, b, vars),
            ("sub", [a, b]) => translate_real(ctx, a, vars) - translate_real(ctx, b, vars),
            ("mul", [a, b]) => translate_real(ctx, a, vars) * translate_real(ctx, b, vars),
            ("div", [a, b]) => translate_real(ctx, a, vars) / translate_real(ctx, b, vars),
            ("neg", [a]) => -translate_real(ctx, a, vars),
            _ => Z3Real::from_real(ctx, 0, 1),
        },
    }
}

fn translate_bool<'ctx>(
    ctx: &'ctx Context,
    expr: &SymBool,
    vars: &mut HashMap<String, Z3Real<'ctx>>,
) -> Z3Bool<'ctx> {
    match expr {
        Sym::Concrete(true) => Z3Bool::from_bool(ctx, true),
        Sym::Concrete(false) => Z3Bool::from_bool(ctx, false),
        Sym::Variable(name) => Z3Bool::new_const(ctx, name.as_str()),
        Sym::Expr { op, args } => match (op.as_str(), args.as_slice()) {
            ("and", [a, b]) => {
                let za = translate_bool(ctx, a, vars);
                let zb = translate_bool(ctx, b, vars);
                Z3Bool::and(ctx, &[&za, &zb])
            }
            ("or", [a, b]) => {
                let za = translate_bool(ctx, a, vars);
                let zb = translate_bool(ctx, b, vars);
                Z3Bool::or(ctx, &[&za, &zb])
            }
            ("not", [a]) => translate_bool(ctx, a, vars).not(),
            _ => Z3Bool::from_bool(ctx, true),
        },
    }
}

fn float_to_rational(f: f64) -> (i32, i32) {
    if f == f.floor() {
        (f as i32, 1)
    } else {
        let scale = 1000000;
        let num = (f * scale as f64).round() as i64;
        let den = scale as i64;
        let g = gcd(num.abs(), den);
        ((num / g) as i32, (den / g) as i32)
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// ============================================================================
// CONSTRAINT DSL
// ============================================================================

/// Build a constraint from symbolic expressions
pub fn constraint(expr: SymBool) -> SymBool {
    expr
}

/// Conjunction of constraints
pub fn all(constraints: Vec<SymBool>) -> SymBool {
    constraints
        .into_iter()
        .fold(Sym::Concrete(true), |acc, c| acc & c)
}

/// Disjunction of constraints
pub fn any(constraints: Vec<SymBool>) -> SymBool {
    constraints
        .into_iter()
        .fold(Sym::Concrete(false), |acc, c| acc | c)
}

