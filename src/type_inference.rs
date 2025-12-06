//! Type Inference for Kleis - Proof of Concept
//!
//! Implements Hindley-Milner type inference for symbolic mathematical expressions.
//! This is a minimal PoC to demonstrate type inference on the existing AST.

use crate::ast::Expression;
use std::collections::HashMap;

/// Type representation for Kleis expressions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Scalar (ℝ)
    Scalar,
    
    /// Vector of dimension n
    Vector(usize),
    
    /// Matrix of dimensions m×n
    Matrix(usize, usize),
    
    /// Type variable (for inference)
    /// α, β, γ in type theory
    Var(TypeVar),
    
    /// Function type: input → output
    Function(Box<Type>, Box<Type>),
    
    /// Polymorphic type: ∀α. T
    /// For now, we'll represent this after generalization
    ForAll(TypeVar, Box<Type>),
}

/// Type variable (α, β, γ, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

/// Type substitution: maps type variables to types
/// Example: {α → Scalar, β → Vector(3)}
#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<TypeVar, Type>,
}

impl Substitution {
    pub fn empty() -> Self {
        Substitution {
            map: HashMap::new(),
        }
    }
    
    pub fn singleton(var: TypeVar, ty: Type) -> Self {
        let mut map = HashMap::new();
        map.insert(var, ty);
        Substitution { map }
    }
    
    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if let Some(t) = self.map.get(v) {
                    // Recursively apply in case of chained substitutions
                    self.apply(t)
                } else {
                    ty.clone()
                }
            }
            Type::Function(t1, t2) => {
                Type::Function(
                    Box::new(self.apply(t1)),
                    Box::new(self.apply(t2))
                )
            }
            Type::ForAll(v, t) => {
                Type::ForAll(v.clone(), Box::new(self.apply(t)))
            }
            _ => ty.clone(),
        }
    }
    
    /// Compose two substitutions
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut map = self.map.clone();
        for (var, ty) in &other.map {
            map.insert(var.clone(), self.apply(ty));
        }
        Substitution { map }
    }
}

/// Type constraint: represents equality between types
/// Example: α = Scalar
#[derive(Debug, Clone)]
pub struct Constraint {
    left: Type,
    right: Type,
}

/// Type inference context: maps variable names to types
#[derive(Debug, Clone)]
pub struct TypeContext {
    vars: HashMap<String, Type>,
    next_var: usize,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            vars: HashMap::new(),
            next_var: 0,
        }
    }
    
    /// Get type of a variable
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
    
    /// Bind a variable to a type
    pub fn bind(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }
    
    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let var = TypeVar(self.next_var);
        self.next_var += 1;
        Type::Var(var)
    }
    
    /// Get all bound variables
    pub fn vars(&self) -> &HashMap<String, Type> {
        &self.vars
    }
}

/// Type inference engine
pub struct TypeInference {
    context: TypeContext,
    constraints: Vec<Constraint>,
}

impl TypeInference {
    pub fn new() -> Self {
        TypeInference {
            context: TypeContext::new(),
            constraints: Vec::new(),
        }
    }
    
    /// Add a constraint
    fn add_constraint(&mut self, left: Type, right: Type) {
        self.constraints.push(Constraint { left, right });
    }
    
    /// Bind a variable to a type
    pub fn bind(&mut self, name: String, ty: Type) {
        self.context.bind(name, ty);
    }
    
    /// Get the context
    pub fn context(&self) -> &TypeContext {
        &self.context
    }
    
    /// Infer type of an expression
    pub fn infer(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            // Constants are scalars
            Expression::Const(_) => Ok(Type::Scalar),
            
            // Variables: look up in context or create fresh var
            Expression::Object(name) => {
                if let Some(ty) = self.context.get(name) {
                    Ok(ty.clone())
                } else {
                    // Unknown variable: create fresh type variable
                    let ty = self.context.fresh_var();
                    self.context.bind(name.clone(), ty.clone());
                    Ok(ty)
                }
            }
            
            // Placeholders: unknown type (fresh variable)
            Expression::Placeholder { .. } => {
                Ok(self.context.fresh_var())
            }
            
            // Operations: infer based on operation type
            Expression::Operation { name, args } => {
                self.infer_operation(name, args)
            }
        }
    }
    
    /// Infer type of an operation
    fn infer_operation(&mut self, name: &str, args: &[Expression]) -> Result<Type, String> {
        match name {
            // Addition: T + T → T (same types)
            "plus" | "minus" => {
                if args.len() != 2 {
                    return Err(format!("{} requires 2 arguments", name));
                }
                let t1 = self.infer(&args[0])?;
                let t2 = self.infer(&args[1])?;
                
                // Add constraint: t1 = t2
                self.add_constraint(t1.clone(), t2.clone());
                
                Ok(t1)
            }
            
            // Multiplication: polymorphic!
            // Scalar × Scalar → Scalar
            // Scalar × Vector → Vector
            // Vector × Vector → Scalar (dot product)
            // Matrix × Matrix → Matrix
            "scalar_multiply" | "times" => {
                if args.len() != 2 {
                    return Err(format!("{} requires 2 arguments", name));
                }
                let t1 = self.infer(&args[0])?;
                let t2 = self.infer(&args[1])?;
                
                // Result type depends on inputs
                let result_ty = self.context.fresh_var();
                
                // TODO: Add more sophisticated multiplication rules
                // For now, just return fresh variable
                Ok(result_ty)
            }
            
            // Division: T / Scalar → T
            "scalar_divide" | "frac" => {
                if args.len() != 2 {
                    return Err(format!("{} requires 2 arguments", name));
                }
                let t1 = self.infer(&args[0])?;
                let t2 = self.infer(&args[1])?;
                
                // Divisor must be scalar
                self.add_constraint(t2, Type::Scalar);
                
                // Result has same type as dividend
                Ok(t1)
            }
            
            // Square root: Scalar → Scalar
            "sqrt" => {
                if args.len() != 1 {
                    return Err("sqrt requires 1 argument".to_string());
                }
                let t1 = self.infer(&args[0])?;
                
                // Argument must be scalar
                self.add_constraint(t1, Type::Scalar);
                
                Ok(Type::Scalar)
            }
            
            // Power: Scalar ^ Scalar → Scalar
            "sup" | "power" => {
                if args.len() != 2 {
                    return Err(format!("{} requires 2 arguments", name));
                }
                let t1 = self.infer(&args[0])?;
                let t2 = self.infer(&args[1])?;
                
                // Both must be scalars
                self.add_constraint(t1, Type::Scalar);
                self.add_constraint(t2, Type::Scalar);
                
                Ok(Type::Scalar)
            }
            
            // Differentiation: (Scalar → Scalar) → (Scalar → Scalar)
            "derivative" | "d_dx" | "partial" => {
                if args.is_empty() {
                    return Err("derivative requires arguments".to_string());
                }
                let t1 = self.infer(&args[0])?;
                
                // Function type: Scalar → Scalar
                self.add_constraint(
                    t1,
                    Type::Function(Box::new(Type::Scalar), Box::new(Type::Scalar))
                );
                
                Ok(Type::Function(Box::new(Type::Scalar), Box::new(Type::Scalar)))
            }
            
            // Integration: (Scalar → Scalar) → Scalar
            "integral" | "int" => {
                if args.is_empty() {
                    return Err("integral requires arguments".to_string());
                }
                let t1 = self.infer(&args[0])?;
                
                // Integrand should be function or scalar
                // Result is scalar
                Ok(Type::Scalar)
            }
            
            // Unknown operation: create fresh variable
            _ => {
                // Infer all argument types
                for arg in args {
                    self.infer(arg)?;
                }
                
                // Return fresh variable for result
                Ok(self.context.fresh_var())
            }
        }
    }
    
    /// Solve all constraints using unification
    pub fn solve(&self) -> Result<Substitution, String> {
        let mut subst = Substitution::empty();
        
        for constraint in &self.constraints {
            let t1 = subst.apply(&constraint.left);
            let t2 = subst.apply(&constraint.right);
            
            let new_subst = unify(&t1, &t2)?;
            subst = subst.compose(&new_subst);
        }
        
        Ok(subst)
    }
    
    /// Infer and solve: complete type inference
    pub fn infer_and_solve(&mut self, expr: &Expression) -> Result<Type, String> {
        let ty = self.infer(expr)?;
        let subst = self.solve()?;
        Ok(subst.apply(&ty))
    }
}

/// Unification: make two types equal
fn unify(t1: &Type, t2: &Type) -> Result<Substitution, String> {
    match (t1, t2) {
        // Same concrete types unify trivially
        (Type::Scalar, Type::Scalar) => Ok(Substitution::empty()),
        (Type::Vector(n1), Type::Vector(n2)) if n1 == n2 => Ok(Substitution::empty()),
        (Type::Matrix(m1, n1), Type::Matrix(m2, n2)) if m1 == m2 && n1 == n2 => {
            Ok(Substitution::empty())
        }
        
        // Type variable unifies with anything (if not occurs)
        (Type::Var(v), t) | (t, Type::Var(v)) => {
            if occurs(v, t) {
                Err(format!("Occurs check failed: {:?} occurs in {:?}", v, t))
            } else {
                Ok(Substitution::singleton(v.clone(), t.clone()))
            }
        }
        
        // Function types: unify components
        (Type::Function(a1, b1), Type::Function(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&s1.apply(b1), &s1.apply(b2))?;
            Ok(s1.compose(&s2))
        }
        
        // Otherwise: cannot unify
        _ => Err(format!("Cannot unify {:?} with {:?}", t1, t2)),
    }
}

/// Occurs check: does variable v occur in type t?
fn occurs(v: &TypeVar, t: &Type) -> bool {
    match t {
        Type::Var(v2) => v == v2,
        Type::Function(t1, t2) => occurs(v, t1) || occurs(v, t2),
        Type::ForAll(_, t) => occurs(v, t),
        _ => false,
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Scalar => write!(f, "ℝ"),
            Type::Vector(n) => write!(f, "Vector(ℝ{})", n),
            Type::Matrix(m, n) => write!(f, "Matrix({}, {})", m, n),
            Type::Var(TypeVar(n)) => write!(f, "α{}", n),
            Type::Function(t1, t2) => write!(f, "{} → {}", t1, t2),
            Type::ForAll(TypeVar(n), t) => write!(f, "∀α{}. {}", n, t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_const_type() {
        let mut infer = TypeInference::new();
        let expr = Expression::Const("42".to_string());
        let ty = infer.infer_and_solve(&expr).unwrap();
        assert_eq!(ty, Type::Scalar);
    }
    
    #[test]
    fn test_addition_type() {
        let mut infer = TypeInference::new();
        
        // 1 + 2
        let expr = Expression::operation(
            "plus",
            vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        );
        
        let ty = infer.infer_and_solve(&expr).unwrap();
        assert_eq!(ty, Type::Scalar);
    }
    
    #[test]
    fn test_variable_inference() {
        let mut infer = TypeInference::new();
        
        // x + 1 (where x is unknown)
        let expr = Expression::operation(
            "plus",
            vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        );
        
        let ty = infer.infer_and_solve(&expr).unwrap();
        // Should infer x : Scalar
        assert_eq!(ty, Type::Scalar);
    }
    
    #[test]
    fn test_division_type() {
        let mut infer = TypeInference::new();
        
        // x / 2
        let expr = Expression::operation(
            "scalar_divide",
            vec![
                Expression::Object("x".to_string()),
                Expression::Const("2".to_string()),
            ],
        );
        
        let ty = infer.infer_and_solve(&expr).unwrap();
        // Should infer x : α (unknown), result: α
        println!("Inferred type: {}", ty);
    }
}

