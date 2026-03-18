// Kleis.g4 v0.5 - ANTLR4 Grammar for Kleis with Pattern Matching
// Date: 2024-12-11 (Updated to sync with EBNF)
// NEW: Pattern Matching (ADR-021 Part 2) - completes algebraic data types
// Includes: data types, pattern matching, structures, implementations, axioms
// UPDATED: Custom operators, named operations (synced with Dec 10 EBNF changes)

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
    | dataDef          // v0.4: Algebraic data types
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
// DATA TYPE DEFINITIONS (v0.4)
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
// PATTERN MATCHING (NEW in v0.5)
// ============================================

// Pattern matching enables USING algebraic data types
// Completes ADR-021: now you can define AND use data types!
// Examples:
//   match x { True => 1 | False => 0 }
//   match opt { None => 0 | Some(x) => x }
//   match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => -1 }

matchExpr
    : 'match' expression '{' matchCases '}'
    ;

matchCases
    : matchCase ('|' matchCase)*
    ;

matchCase
    : pattern '=>' expression
    ;

pattern
    : wildcardPattern
    | variablePattern
    | constructorPattern
    | constantPattern
    ;

wildcardPattern
    : '_'
    ;

variablePattern
    : LOWERCASE_IDENTIFIER       // Must start with lowercase
    ;

constructorPattern
    : UPPERCASE_IDENTIFIER ('(' patternArgs ')')?  // Must start with uppercase
    ;

patternArgs
    : pattern (',' pattern)*
    ;

constantPattern
    : NUMBER
    | STRING
    | BOOLEAN
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
    | matchExpr                           // NEW v0.5: Pattern matching
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
    : '+' | '-' | '×' | '/' | '·' | '*' | '^'
    | '⊗' | '∘' | '∗'
    | CUSTOM_OPERATOR     // User-defined operators: •, ⊕, ⊙, ⊛, etc.
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
    : '(' infixOp ')'     // Infix as function: (+), (×)
    | infixOp
    | prefixOp
    | postfixOp
    | IDENTIFIER          // Named operations: transpose, inverse, dot
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

// Boolean literals
BOOLEAN
    : 'True'
    | 'False'
    ;

// Identifiers (split for pattern matching)
UPPERCASE_IDENTIFIER
    : [A-Z] [a-zA-Z0-9_]*      // Constructors: Some, None, True
    ;

LOWERCASE_IDENTIFIER
    : [a-z_] [a-zA-Z0-9_]*     // Variables: x, value, _tmp
    ;

IDENTIFIER
    : [a-zA-Z_] [a-zA-Z0-9_]*  // General identifier
    ;

// Greek lowercase (type variables)
GREEK_LOWER
    : 'α' | 'β' | 'γ' | 'δ' | 'ε' | 'ζ' | 'η' | 'θ'
    | 'ι' | 'κ' | 'λ' | 'μ' | 'ν' | 'ξ' | 'ο' | 'π'
    | 'ρ' | 'σ' | 'τ' | 'υ' | 'φ' | 'χ' | 'ψ' | 'ω'
    ;

// Custom operators (Unicode math symbols)
// Examples: •, ⊕, ⊙, ⊛, ⋆, ∪, ∩, etc.
CUSTOM_OPERATOR
    : [\u2200-\u22FF]      // Mathematical Operators block
    | [\u2A00-\u2AFF]      // Supplemental Mathematical Operators
    | [\u27C0-\u27EF]      // Miscellaneous Mathematical Symbols-A
    | [\u2980-\u29FF]      // Miscellaneous Mathematical Symbols-B
    ;

// Numbers
NUMBER
    : [0-9]+ ('.' [0-9]+)? ([eE] [+-]? [0-9]+)?
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

// Version 0.5.1 (2024-12-11):
//   - SYNC: Added CUSTOM_OPERATOR lexer rule (Unicode math symbols)
//   - SYNC: Added IDENTIFIER to operatorSymbol (named operations)
//   - Now matches EBNF grammar updated on Dec 10
//   - Supports custom operators: •, ⊕, ⊙, ⊛, etc.
//   - Supports named operations: transpose, inverse, dot
//
// Version 0.5 (2024-12-08):
//   - Added pattern matching (matchExpr, pattern, matchCase)
//   - Completes ADR-021: Can now USE algebraic data types
//   - Wildcard, variable, constructor, and constant patterns
//   - Nested pattern matching support
//   - Exhaustiveness checking (compile-time warnings)
//   - Unreachable pattern detection
//   - Self-hosting capability: can implement type checker in Kleis!
//   - Split IDENTIFIER into UPPERCASE/LOWERCASE for pattern disambiguation
//
// Version 0.4 (2024-12-08):
//   - Added algebraic data types (dataDef, dataVariant, dataField)
//   - Implements ADR-021 Part 1: DEFINE data types
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


