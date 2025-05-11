// Kleis.g4 - ANTLR Grammar for Kleis v0.2

grammar Kleis;

program         : statement* EOF ;

statement
    : objectDecl
    | morphismDecl
    | constDecl
    | operationDecl
    | defineStmt
    | assertStmt
    ;

objectDecl      : 'object' type IDENT ( '{' property* '}' )? ;
morphismDecl    : 'narrow' expression '->' expression '[' IDENT ']' annotation? ;
constDecl       : 'const' IDENT ( '=' expression )? ;
operationDecl   : 'operation' IDENT ':' '(' typeList ')' '->' type ;
defineStmt      : 'define' IDENT '=' expression ;
assertStmt      : 'assert' expression '==' expression ;

expression
    : IDENT
    | NUMBER
    | '(' expression ')'
    | expression op=('*' | '/' | '+' | '-' | '×' | '·') expression
    ;

typeList        : type (',' type)* ;
type            : IDENT ('[' typeList ']')? ;

property        : IDENT ( '=' expression )? ;

annotation      : '@{' annotationPair* '}' ;
annotationPair  : IDENT '=' '[' IDENT* ']' ;

IDENT           : [a-zA-Z_] [a-zA-Z0-9_]* ;
NUMBER          : [0-9]+ ('.' [0-9]+)? ;

WS              : [ \t\r\n]+ -> skip ;