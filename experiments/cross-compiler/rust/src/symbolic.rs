// ============================================================================
// KLEIS SYMBOLIC COMPUTATION LAYER
// ============================================================================
//
// Provides symbolic values that propagate through computation.
// Variables are NOT concrete Rust values - they are symbolic placeholders.
//
// Usage in generated code:
//   use kleis_runtime::Sym;
//   let x: SymReal = Sym::var("x");
//   let expr = x + Sym::concrete(3.0);
//
// ============================================================================

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg, Not, BitAnd, BitOr};

// ============================================================================
// SYMBOLIC VALUE TYPE
// ============================================================================

/// A symbolic value can be:
/// - Concrete: a known value that can be computed
/// - Variable: an unknown that represents a free variable
/// - Expr: a symbolic expression tree
#[derive(Debug, Clone)]
pub enum Sym<T: Clone> {
    /// Known value - can be computed directly
    Concrete(T),
    /// Unknown variable - placeholder for solver
    Variable(String),
    /// Expression tree - deferred computation
    Expr { op: String, args: Vec<Sym<T>> },
}

/// Type aliases for common symbolic types
pub type SymReal = Sym<f64>;
pub type SymInt = Sym<i64>;
pub type SymBool = Sym<bool>;

// ============================================================================
// CONSTRUCTORS
// ============================================================================

impl<T: Clone> Sym<T> {
    /// Create a concrete (known) value
    pub fn concrete(value: T) -> Self {
        Sym::Concrete(value)
    }

    /// Create a symbolic variable
    pub fn var(name: &str) -> Self {
        Sym::Variable(name.to_string())
    }

    /// Create a symbolic expression
    pub fn expr(op: &str, args: Vec<Sym<T>>) -> Self {
        Sym::Expr {
            op: op.to_string(),
            args,
        }
    }

    /// Check if this value is concrete
    pub fn is_concrete(&self) -> bool {
        matches!(self, Sym::Concrete(_))
    }

    /// Check if this value is symbolic
    pub fn is_symbolic(&self) -> bool {
        !self.is_concrete()
    }

    /// Get the concrete value if available
    pub fn get_concrete(&self) -> Option<&T> {
        match self {
            Sym::Concrete(v) => Some(v),
            _ => None,
        }
    }

    /// Collect all variable names in this expression
    pub fn collect_vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        self.collect_vars_into(&mut vars);
        vars
    }

    fn collect_vars_into(&self, vars: &mut HashSet<String>) {
        match self {
            Sym::Concrete(_) => {}
            Sym::Variable(name) => {
                vars.insert(name.clone());
            }
            Sym::Expr { args, .. } => {
                for arg in args {
                    arg.collect_vars_into(vars);
                }
            }
        }
    }
}

// ============================================================================
// SYMBOLIC ARITHMETIC (with constant folding)
// ============================================================================

impl Add for SymReal {
    type Output = SymReal;

    fn add(self, other: SymReal) -> SymReal {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a + b),
            (Sym::Concrete(a), _) if *a == 0.0 => other,
            (_, Sym::Concrete(b)) if *b == 0.0 => self,
            _ => Sym::expr("add", vec![self, other]),
        }
    }
}

impl Sub for SymReal {
    type Output = SymReal;

    fn sub(self, other: SymReal) -> SymReal {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a - b),
            (_, Sym::Concrete(b)) if *b == 0.0 => self,
            _ => Sym::expr("sub", vec![self, other]),
        }
    }
}

impl Mul for SymReal {
    type Output = SymReal;

    fn mul(self, other: SymReal) -> SymReal {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a * b),
            (Sym::Concrete(a), _) if *a == 0.0 => Sym::Concrete(0.0),
            (_, Sym::Concrete(b)) if *b == 0.0 => Sym::Concrete(0.0),
            (Sym::Concrete(a), _) if *a == 1.0 => other,
            (_, Sym::Concrete(b)) if *b == 1.0 => self,
            _ => Sym::expr("mul", vec![self, other]),
        }
    }
}

impl Div for SymReal {
    type Output = SymReal;

    fn div(self, other: SymReal) -> SymReal {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a / b),
            (_, Sym::Concrete(b)) if *b == 1.0 => self,
            _ => Sym::expr("div", vec![self, other]),
        }
    }
}

impl Neg for SymReal {
    type Output = SymReal;

    fn neg(self) -> SymReal {
        match &self {
            Sym::Concrete(a) => Sym::Concrete(-a),
            Sym::Expr { op, args } if op == "neg" && args.len() == 1 => args[0].clone(),
            _ => Sym::expr("neg", vec![self]),
        }
    }
}

// ============================================================================
// SYMBOLIC COMPARISONS
// ============================================================================

impl SymReal {
    pub fn sym_eq(self, other: SymReal) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a == b),
            (Sym::Variable(n1), Sym::Variable(n2)) if n1 == n2 => Sym::Concrete(true),
            _ => Sym::expr("eq", vec![]),
        }
    }

    pub fn sym_ne(self, other: SymReal) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a != b),
            _ => Sym::expr("ne", vec![]),
        }
    }

    pub fn sym_lt(self, other: SymReal) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a < b),
            _ => Sym::expr("lt", vec![]),
        }
    }

    pub fn sym_le(self, other: SymReal) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a <= b),
            _ => Sym::expr("le", vec![]),
        }
    }

    pub fn sym_gt(self, other: SymReal) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a > b),
            _ => Sym::expr("gt", vec![]),
        }
    }

    pub fn sym_ge(self, other: SymReal) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(a), Sym::Concrete(b)) => Sym::Concrete(a >= b),
            _ => Sym::expr("ge", vec![]),
        }
    }
}

// ============================================================================
// SYMBOLIC LOGIC
// ============================================================================

impl BitAnd for SymBool {
    type Output = SymBool;

    fn bitand(self, other: SymBool) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(true), _) => other,
            (_, Sym::Concrete(true)) => self,
            (Sym::Concrete(false), _) => Sym::Concrete(false),
            (_, Sym::Concrete(false)) => Sym::Concrete(false),
            _ => Sym::expr("and", vec![self, other]),
        }
    }
}

impl BitOr for SymBool {
    type Output = SymBool;

    fn bitor(self, other: SymBool) -> SymBool {
        match (&self, &other) {
            (Sym::Concrete(true), _) => Sym::Concrete(true),
            (_, Sym::Concrete(true)) => Sym::Concrete(true),
            (Sym::Concrete(false), _) => other,
            (_, Sym::Concrete(false)) => self,
            _ => Sym::expr("or", vec![self, other]),
        }
    }
}

impl Not for SymBool {
    type Output = SymBool;

    fn not(self) -> SymBool {
        match &self {
            Sym::Concrete(b) => Sym::Concrete(!b),
            Sym::Expr { op, args } if op == "not" && args.len() == 1 => args[0].clone(),
            _ => Sym::expr("not", vec![self]),
        }
    }
}

impl SymBool {
    pub fn implies(self, other: SymBool) -> SymBool {
        (!self) | other
    }
}

// ============================================================================
// SUBSTITUTION AND EVALUATION
// ============================================================================

impl SymReal {
    /// Substitute a variable with a value
    pub fn substitute(self, var_name: &str, value: SymReal) -> SymReal {
        match self {
            Sym::Concrete(v) => Sym::Concrete(v),
            Sym::Variable(name) if name == var_name => value,
            Sym::Variable(name) => Sym::Variable(name),
            Sym::Expr { op, args } => Sym::Expr {
                op,
                args: args
                    .into_iter()
                    .map(|a| a.substitute(var_name, value.clone()))
                    .collect(),
            },
        }
    }

    /// Evaluate with bindings
    pub fn eval_with(self, bindings: &HashMap<String, f64>) -> SymReal {
        match self {
            Sym::Concrete(v) => Sym::Concrete(v),
            Sym::Variable(name) => bindings
                .get(&name)
                .map(|&v| Sym::Concrete(v))
                .unwrap_or(Sym::Variable(name)),
            Sym::Expr { op, args } => {
                let evaled: Vec<SymReal> =
                    args.into_iter().map(|a| a.eval_with(bindings)).collect();
                if evaled.iter().all(|a| a.is_concrete()) {
                    Self::reduce_op(&op, &evaled)
                } else {
                    Sym::Expr { op, args: evaled }
                }
            }
        }
    }

    fn reduce_op(op: &str, args: &[SymReal]) -> SymReal {
        match (op, args) {
            ("add", [Sym::Concrete(a), Sym::Concrete(b)]) => Sym::Concrete(a + b),
            ("sub", [Sym::Concrete(a), Sym::Concrete(b)]) => Sym::Concrete(a - b),
            ("mul", [Sym::Concrete(a), Sym::Concrete(b)]) => Sym::Concrete(a * b),
            ("div", [Sym::Concrete(a), Sym::Concrete(b)]) => Sym::Concrete(a / b),
            ("neg", [Sym::Concrete(a)]) => Sym::Concrete(-a),
            _ => Sym::Expr {
                op: op.to_string(),
                args: args.to_vec(),
            },
        }
    }
}

// ============================================================================
// PRETTY PRINTING
// ============================================================================

impl fmt::Display for SymReal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sym::Concrete(v) => write!(f, "{}", v),
            Sym::Variable(name) => write!(f, "{}", name),
            Sym::Expr { op, args } => match (op.as_str(), args.as_slice()) {
                ("add", [a, b]) => write!(f, "({} + {})", a, b),
                ("sub", [a, b]) => write!(f, "({} - {})", a, b),
                ("mul", [a, b]) => write!(f, "({} * {})", a, b),
                ("div", [a, b]) => write!(f, "({} / {})", a, b),
                ("neg", [a]) => write!(f, "-{}", a),
                _ => {
                    write!(f, "{}(", op)?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ")")
                }
            },
        }
    }
}

impl fmt::Display for SymBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sym::Concrete(v) => write!(f, "{}", v),
            Sym::Variable(name) => write!(f, "{}", name),
            Sym::Expr { op, args } => match (op.as_str(), args.as_slice()) {
                ("and", [a, b]) => write!(f, "({} ∧ {})", a, b),
                ("or", [a, b]) => write!(f, "({} ∨ {})", a, b),
                ("not", [a]) => write!(f, "¬{}", a),
                _ => write!(f, "{}(...)", op),
            },
        }
    }
}

// ============================================================================
// SYMBOLIC IF-THEN-ELSE
// ============================================================================

/// Symbolic if-then-else: evaluates condition if concrete, defers otherwise
pub fn sym_ite<T: Clone>(cond: SymBool, then_val: Sym<T>, else_val: Sym<T>) -> Sym<T> {
    match cond {
        Sym::Concrete(true) => then_val,
        Sym::Concrete(false) => else_val,
        _ => Sym::expr("ite", vec![]),
    }
}

