Here’s one end-to-end option that will work with your current architecture (parser → AST → evaluator → debug hook), while matching your debugger goal: locations only for executable expressions.

I’ll describe it as a complete pipeline you can implement step-by-step, with concrete shapes and minimal churn.

Goal

You want the debugger to step on “things that execute”:

Operation / infix op

Call

Let

If / Conditional

Match

maybe Return / Block (if you have them)

You don’t care about stepping on:

literals (Const, String, etc.)

identifiers

type annotations

parentheses

So: only those executable nodes need spans and only those nodes should emit debug events.

Step 1: AST change (small + targeted)

In /mnt/data/ast.rs, keep Expression mostly the same, but add span: Option<SourceSpan> only to executable variants.

Example (sketch; adapt to your actual enum variants):

pub enum Expression {
    // atoms: no span
    Const(String),
    StringLiteral(String),
    Ident(String),
    Placeholder,

    // executable: span
    Operation {
        name: String,
        args: Vec<Expression>,
        span: Option<SourceSpan>,
    },

    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
        span: Option<SourceSpan>,
    },

    Let {
        pattern: Pattern,
        type_annotation: Option<String>,
        value: Box<Expression>,
        body: Box<Expression>,
        span: Option<SourceSpan>,
    },

    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
        span: Option<SourceSpan>,
    },

    Match {
        scrutinee: Box<Expression>,
        cases: Vec<MatchCase>,
        span: Option<SourceSpan>,
    },

    // optional: if you want “lambda creation” to be steppable:
    Lambda {
        params: Vec<LambdaParam>,
        body: Box<Expression>,
        span: Option<SourceSpan>,
    },
}


Why this works:

It doesn’t force you to wrap every node.

It gives location exactly where you want breakpoints/stepping.

Step 2: Parser attaches spans only for executable nodes

In /mnt/data/kleis_parser.rs you already track:

line, column

current_span() returning a SourceSpan “point”

and SourceSpan::with_end(...) exists in ast.rs

Add these parser helpers:

fn mark(&self) -> SourceSpan {
    self.current_span()
}

fn range(&self, start: SourceSpan, end: SourceSpan) -> SourceSpan {
    start.with_end(end.line, end.column)
}


Now, in parsing functions that build executable nodes, do:

let start = self.mark();

parse pieces

let end = self.mark();

span: Some(self.range(start, end))

Example: parsing a binary operation (or generic operation)
fn parse_operation_expr(&mut self) -> Result<Expression, KleisParseError> {
    let start = self.mark();

    // parse "f(arg1, arg2)" or "a + b" in whatever style you use
    let name = self.parse_identifier()?;
    self.expect_char('(')?;
    let args = self.parse_args()?; // Vec<Expression>
    self.expect_char(')')?;

    let end = self.mark();

    Ok(Expression::Operation {
        name,
        args,
        span: Some(self.range(start, end)),
    })
}

Example: parsing let
fn parse_let_expr(&mut self) -> Result<Expression, KleisParseError> {
    let start = self.mark();

    self.expect_keyword("let")?;
    let pattern = self.parse_pattern()?;
    let type_annotation = self.parse_optional_type_annotation()?;
    self.expect_char('=')?;
    let value = Box::new(self.parse_expression()?);
    self.expect_keyword("in")?;
    let body = Box::new(self.parse_expression()?);

    let end = self.mark();

    Ok(Expression::Let {
        pattern,
        type_annotation,
        value,
        body,
        span: Some(self.range(start, end)),
    })
}


And for literals, don’t add spans:

fn parse_number_expr(&mut self) -> Result<Expression, KleisParseError> {
    let n = self.parse_number()?;
    Ok(Expression::Const(n)) // no span
}

Step 3: Evaluator uses spans to emit debug events (only when executable)

In /mnt/data/evaluator.rs, add:

fn expr_span(expr: &Expression) -> Option<SourceSpan> {
    match expr {
        Expression::Operation { span, .. } => *span,
        Expression::Call { span, .. } => *span,
        Expression::Let { span, .. } => *span,
        Expression::Conditional { span, .. } => *span,
        Expression::Match { span, .. } => *span,
        Expression::Lambda { span, .. } => *span,
        _ => None,
    }
}

fn is_executable(expr: &Expression) -> bool {
    expr_span(expr).is_some()
}


Then in eval_internal() wrap the hook calls:

let span = expr_span(expr);

if let (Some(span), Some(ref mut hook)) = (span, self.debug_hook.borrow_mut().as_deref_mut()) {
    // convert SourceSpan -> SourceLocation
    let mut loc = SourceLocation::new(span.line, span.column);

    // attach file if you can:
    // Option A: evaluator has a current file field (see Step 4)
    // Option B: use closure.file for function bodies (you already do this)

    hook.on_eval_start(expr, &loc, depth);
}

// ... do evaluation ...

if let (Some(span), Some(ref mut hook)) = (span, self.debug_hook.borrow_mut().as_deref_mut()) {
    let mut loc = SourceLocation::new(span.line, span.column);
    hook.on_eval_end(expr, &result, depth);
}


Now your debugger only sees events for executable nodes.

Step 4: File name handling (simple option that works)

You have file names on closures (closure.file) and function definitions, but not globally.

Simplest working model:

Add to Evaluator:

current_file: Option<std::path::PathBuf>,


When you start evaluating a top-level file, set it once:

evaluator.current_file = Some(file_path.clone());


When converting span → SourceLocation, attach:

if let Some(ref f) = self.current_file {
    loc = loc.with_file(f.display().to_string());
}


This is “good enough” for a debugger. You can later evolve into a full source map.

Step 5: Preserve spans through substitute() and any AST rebuilding

In evaluator.rs, you rebuild expressions in substitute() and elsewhere.

You must copy the spans when reconstructing executable nodes:

Expression::Operation { name, args, span } => Expression::Operation {
    name: name.clone(),
    args: args.iter().map(|a| self.substitute(a, ...)).collect(),
    span: *span,
}


Same for Let, Conditional, Match, Call, Lambda.

If you forget this, spans disappear after substitution and the debugger will “go blind”.

Step 6: Breakpoints and stepping become straightforward

Now breakpoints can be matched by line/col:

store breakpoints as (file, line) or (file, line, col)

when on_eval_start fires, check if its location hits a breakpoint

if yes, pause

Stepping:

“step over”: pause at the next executable event at same depth

“step into”: pause at the next executable event at depth+1

“step out”: pause when depth decreases

Your existing depth parameter in the hook is already set up for this.

Why this end-to-end option is good for Kleis

Minimal AST change (only executable variants)

Minimal parser change (add span: Some(...) where you build those variants)

Evaluator becomes the single place defining “what is executable”

Debugger UX feels natural (no stopping on constants)