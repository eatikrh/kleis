// Kleis.g4 v0.4 - ANTLR4 Grammar for Kleis with Algebraic Data Types
// Date: 2024-12-08
// NEW: Algebraic data types (ADR-021) for self-hosting
// Includes: data types, structures, implementations, type inference, axioms

grammar Kleis;

// ============================================
// TOP-LEVEL DECLARATIONS
// ============================================

program
    : declaration* EOF
    ;

declaration
    : libraryAnnotation
    | versionAnnotation
    | structureDef
    | implementsDef
    | dataDef          // NEW in v0.4: Algebraic data types
    | functionDef
    | operationDecl
    | objectDecl
    | typeAlias
    | constDecl      // Deprecated, use functionDef
    | morphismDecl   // Deprecated
    ;

// ============================================
// ANNOTATIONS
// ============================================

libraryAnnotation
    : '@library' '(' STRING ')'
    ;

versionAnnotation
    : '@version' '(' STRING ')'
    ;

// ============================================
// DATA TYPE DEFINITIONS (NEW in v0.4)
// ============================================

// Algebraic data types enable user-defined types
// Examples:
//   data Bool = True | False
//   data Option(T) = None | Some(T)
//   data Type = Scalar | Vector(n: Nat) | Matrix(m: Nat, n: Nat)

dataDef
    : 'data' IDENTIFIER ('(' typeParams ')')? '=' 
      dataVariant ('|' dataVariant)*
    ;

dataVariant
    : IDENTIFIER ('(' dataFields ')')?
    ;

dataFields
    : dataField (',' dataField)*
    ;

dataField
    : IDENTIFIER ':' type      // Named field: m: Nat, value: T
    | type                      // Positional field: T, Nat
    ;

// ============================================
// STRUCTURE DEFINITIONS
// ============================================

structureDef
    : 'structure' IDENTIFIER '(' typeParams ')' 
      extendsClause?
      overClause?
      '{' structureMember* '}'
    ;

typeParams
    : typeParam (',' typeParam)*
    ;

typeParam
    : IDENTIFIER (':' kind)?
    ;

kind
    : 'Type'
    | 'ℕ'
    | 'Field'
    | '*'
    | kind '->' kind
    ;

extendsClause
    : 'extends' IDENTIFIER ('(' typeArgs ')')?
    ;

overClause
    : 'over' 'Field' '(' type ')'
    ;

structureMember
    : operationDecl
    | elementDecl
    | axiomDecl
    | nestedStructure
    | supportsBlock
    | notationDecl
    ;

operationDecl
    : 'operation' operatorSymbol ':' typeSignature
    ;

elementDecl
    : 'element' IDENTIFIER ':' type
    ;

axiomDecl
    : 'axiom' IDENTIFIER ':' proposition
    ;

nestedStructure
    : 'structure' IDENTIFIER ':' IDENTIFIER '(' type ')'
      '{' structureMember* '}'
    ;

supportsBlock
    : 'supports' '{' operationDecl* '}'
    ;

notationDecl
    : 'notation' IDENTIFIER '(' params ')' '=' expression
    ;

// ============================================
// IMPLEMENTATIONS
// ============================================

implementsDef
    : 'implements' IDENTIFIER '(' typeArgs ')' 
      overClause?
      ('{' implMember* '}')?
    ;

implMember
    : elementImpl
    | operationImpl
    | verifyStmt
    ;

elementImpl
    : 'element' IDENTIFIER '=' expression
    ;

operationImpl
    : 'operation' operatorSymbol '=' implementation
    | 'operation' operatorSymbol '(' params ')' '=' expression
    ;

implementation
    : IDENTIFIER          // Function name (builtin_add)
    | expression          // Inline expression
    ;

verifyStmt
    : 'verify' IDENTIFIER
    ;

// ============================================
// FUNCTION DEFINITIONS
// ============================================

functionDef
    : 'define' IDENTIFIER typeAnnotation? '=' expression
    | 'define' IDENTIFIER '(' params ')' (':' type)? '=' expression
    ;

params
    : param (',' param)*
    ;

param
    : IDENTIFIER (':' type)?
    | '(' IDENTIFIER+ ':' type ')'    // Multiple params with same type
    ;

// ============================================
// TYPE SYSTEM
// ============================================

typeSignature
    : polymorphicType
    | type
    ;

polymorphicType
    : '∀' typeVarList '.' constraints? type
    | 'forall' typeVarList '.' constraints? type
    ;

typeVarList
    : typeVarDecl+
    | '(' typeVarDecl (',' typeVarDecl)* ')'
    ;

typeVarDecl
    : IDENTIFIER (':' kind)?
    ;

constraints
    : constraint (',' constraint)* '⇒'
    | constraint (',' constraint)* '=>'
    ;

constraint
    : IDENTIFIER '(' type ')'             // Monoid(T)
    | type '=' type                       // m = n
    | expression                          // x ≠ 0
    ;

type
    : primitiveType
    | parametricType
    | functionType
    | typeVariable
    | '(' type ')'
    ;

primitiveType
    : 'ℝ' | 'ℂ' | 'ℤ' | 'ℕ' | 'ℚ'
    | 'Real' | 'Complex' | 'Integer' | 'Nat' | 'Rational'
    | 'Bool' | 'String'
    ;

parametricType
    : IDENTIFIER '(' typeArgs ')'
    ;

typeArgs
    : type (',' type)*
    ;

functionType
    : type '→' type
    | type '->' type
    ;

typeVariable
    : GREEK_LOWER      // α, β, γ, etc.
    | IDENTIFIER       // a, b, T, M, etc.
    ;

typeAnnotation
    : ':' type
    ;

// ============================================
// PROPOSITIONS (for axioms)
// ============================================

proposition
    : forAllProp
    | existsProp
    | expression
    ;

forAllProp
    : '∀' variables whereClause? '.' proposition
    | 'forall' variables whereClause? '.' proposition
    ;

existsProp
    : '∃' variables whereClause? '.' proposition
    | 'exists' variables whereClause? '.' proposition
    ;

variables
    : varDecl+
    | '(' varDecl+ ')'
    ;

varDecl
    : IDENTIFIER (':' type)?
    | '(' IDENTIFIER+ ':' type ')'
    ;

whereClause
    : 'where' expression
    ;

// ============================================
// EXPRESSIONS
// ============================================

expression
    : primary
    | prefixOp expression
    | expression postfixOp
    | expression infixOp expression
    | expression '(' arguments ')'        // Function application
    | '[' expressions ']'                 // Vector/list literal
    | lambda
    | letBinding
    | conditional
    ;

primary
    : IDENTIFIER
    | NUMBER
    | STRING
    | symbolicConstant
    | '(' expression ')'
    | placeholder
    ;

symbolicConstant
    : 'π' | 'e' | 'i' | 'ℏ' | 'c'
    | 'φ' | '∞' | '∅'
    ;

placeholder
    : '□'
    ;

prefixOp
    : '-' | '∇' | '∂' | '¬' | '√'
    ;

postfixOp
    : '!' | '†' | '*' | 'ᵀ' | '^T'
    ;

infixOp
    : arithmeticOp
    | relationOp
    | logicOp
    | calcOp
    ;

arithmeticOp
    : '+' | '-' | '×' | '/' | '·' | '*'
    | '^' | '⊗' | '∘'
    ;

relationOp
    : '=' | '≠' | '<' | '>' | '≤' | '≥'
    | '≈' | '≡' | '~' | '∈' | '∉' | '⊂' | '⊆'
    ;

logicOp
    : '∧' | '∨' | '⟹' | '⟺' | '→' | '⇒'
    ;

calcOp
    : '∂' | '∫' | '∇' | 'd/dx'
    ;

operatorSymbol
    : infixOp
    | prefixOp
    | postfixOp
    | '(' infixOp ')'     // For declaring infix as function
    ;

arguments
    : expression (',' expression)*
    ;

expressions
    : expression (',' expression)*
    ;

lambda
    : 'λ' params '.' expression
    | 'lambda' params '.' expression
    ;

letBinding
    : 'let' IDENTIFIER typeAnnotation? '=' expression 'in' expression
    ;

conditional
    : 'if' expression 'then' expression 'else' expression
    ;

// ============================================
// DEPRECATED (from v0.2, for compatibility)
// ============================================

objectDecl
    : 'object' type IDENTIFIER ('{' property* '}')?
    ;

morphismDecl
    : 'narrow' expression '->' expression '[' IDENTIFIER ']' annotation?
    ;

constDecl
    : 'const' IDENTIFIER ('=' expression)?
    ;

property
    : IDENTIFIER ('=' expression)?
    ;

annotation
    : '@{' annotationPair* '}'
    ;

annotationPair
    : IDENTIFIER '=' '[' IDENTIFIER* ']'
    ;

// ============================================
// TYPE ALIASES
// ============================================

typeAlias
    : 'type' IDENTIFIER '=' type
    ;

// ============================================
// LEXER RULES
// ============================================

// Identifiers
IDENTIFIER
    : [a-zA-Z_] [a-zA-Z0-9_]*
    ;

// Greek lowercase (type variables)
GREEK_LOWER
    : 'α' | 'β' | 'γ' | 'δ' | 'ε' | 'ζ' | 'η' | 'θ'
    | 'ι' | 'κ' | 'λ' | 'μ' | 'ν' | 'ξ' | 'ο' | 'π'
    | 'ρ' | 'σ' | 'τ' | 'υ' | 'φ' | 'χ' | 'ψ' | 'ω'
    ;

// Numbers
NUMBER
    : [0-9]+ ('.' [0-9]+)? ([eE] [+-]? [+-]? [0-9]+)?
    ;

// Strings
STRING
    : '"' (~["\\\r\n])* '"'
    ;

// Comments
LINE_COMMENT
    : '//' ~[\r\n]* -> skip
    ;

BLOCK_COMMENT
    : '/*' .*? '*/' -> skip
    ;

// Whitespace
WS
    : [ \t\r\n]+ -> skip
    ;

// ============================================
// CHANGE LOG
// ============================================

// Version 0.4 (2024-12-08):
//   - Added algebraic data types (dataDef, dataVariant, dataField)
//   - Implements ADR-021 for self-hosting type system
//   - Enables user-defined types in Kleis files
//
// Version 0.3 (2024-12-05):
//   - Type system with polymorphic types
//   - Structure definitions with axioms
//   - Implements blocks
//
// Version 0.2:
//   - Basic mathematical expressions
//   - Function definitions

