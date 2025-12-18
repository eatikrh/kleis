//! Kleis REPL - Interactive Read-Eval-Print Loop
//!
//! Usage:
//!   cargo run --bin repl
//!
//! Commands:
//!   :help          Show help
//!   :ast <expr>    Show parsed AST
//!   :type <expr>   Show inferred type  
//!   :verify <expr> Verify with Z3
//!   :load <file>   Load .kleis file
//!   :env           Show defined functions
//!   :export [file] Export definitions to .kleis file
//!   :quit          Exit

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use kleis::evaluator::Evaluator;
use kleis::kleis_ast::TopLevel;
use kleis::kleis_parser::{parse_kleis_program, KleisParser};
use kleis::pretty_print::PrettyPrinter;
use kleis::render::{build_default_context, render_expression, RenderTarget};

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};
#[cfg(feature = "axiom-verification")]
use kleis::structure_registry::StructureRegistry;

const VERSION: &str = "0.1.0";

fn main() -> RlResult<()> {
    println!();
    println!("🧮 Kleis REPL v{}", VERSION);
    println!("   Type :help for commands, :quit to exit");
    println!();

    let mut rl = DefaultEditor::new()?;

    // Try to get history file path
    let history_file: PathBuf = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".kleis_history"))
        .unwrap_or_else(|_| PathBuf::from(".kleis_history"));

    // Load history if available
    let _ = rl.load_history(&history_file);

    // REPL state
    let mut evaluator = Evaluator::new();
    let render_ctx = build_default_context();

    #[cfg(feature = "axiom-verification")]
    let registry = StructureRegistry::new();

    let mut multiline_buffer = String::new();
    // Two separate modes: block mode (:{ ... :}) vs line continuation (\)
    let mut in_block_mode = false;
    let mut in_line_continuation = false;

    loop {
        let prompt = if in_block_mode || in_line_continuation {
            "   "
        } else {
            "λ> "
        };
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                let line_trimmed = line.trim();

                // Handle explicit multi-line block mode (:{ ... :})
                if line_trimmed == ":{" {
                    in_block_mode = true;
                    multiline_buffer.clear();
                    continue;
                }
                if line_trimmed == ":}" {
                    in_block_mode = false;
                    let full_input = std::mem::take(&mut multiline_buffer);
                    let full_input = full_input.trim();
                    if !full_input.is_empty() {
                        let _ = rl.add_history_entry(full_input);
                        process_input(
                            full_input,
                            &mut evaluator,
                            &render_ctx,
                            #[cfg(feature = "axiom-verification")]
                            &registry,
                        );
                    }
                    continue;
                }

                // In explicit block mode, accumulate until :}
                if in_block_mode {
                    multiline_buffer.push_str(&line);
                    multiline_buffer.push('\n');
                    continue;
                }

                // Single line - check if incomplete (unbalanced brackets)
                if line_trimmed.is_empty() {
                    continue;
                }

                // Check for line continuation (backslash at end)
                if let Some(without_backslash) = line_trimmed.strip_suffix('\\') {
                    multiline_buffer.push_str(without_backslash);
                    multiline_buffer.push(' ');
                    in_line_continuation = true;
                    continue;
                }

                // Complete the input (either from continuation or single line)
                let full_input = if in_line_continuation || !multiline_buffer.is_empty() {
                    multiline_buffer.push_str(line_trimmed);
                    in_line_continuation = false;
                    std::mem::take(&mut multiline_buffer)
                } else {
                    line_trimmed.to_string()
                };

                let _ = rl.add_history_entry(&full_input);

                // Check for quit
                if full_input == ":quit" || full_input == ":q" {
                    println!("Goodbye! 👋");
                    break;
                }

                process_input(
                    &full_input,
                    &mut evaluator,
                    &render_ctx,
                    #[cfg(feature = "axiom-verification")]
                    &registry,
                );
            }
            Err(ReadlineError::Interrupted) => {
                if in_block_mode || in_line_continuation {
                    println!("(multi-line cancelled)");
                    multiline_buffer.clear();
                    in_block_mode = false;
                    in_line_continuation = false;
                } else {
                    println!("^C");
                }
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_file);

    Ok(())
}

/// Process a complete input (command or expression)
#[cfg(feature = "axiom-verification")]
fn process_input(
    input: &str,
    evaluator: &mut Evaluator,
    ctx: &kleis::render::GlyphContext,
    registry: &StructureRegistry,
) {
    if input.starts_with(':') {
        handle_command(input, evaluator, ctx, registry);
    } else {
        eval_expression(input, evaluator, ctx);
    }
}

#[cfg(not(feature = "axiom-verification"))]
fn process_input(input: &str, evaluator: &mut Evaluator, ctx: &kleis::render::GlyphContext) {
    if input.starts_with(':') {
        handle_command_no_z3(input, evaluator, ctx);
    } else {
        eval_expression(input, evaluator, ctx);
    }
}

fn eval_expression(input: &str, evaluator: &Evaluator, ctx: &kleis::render::GlyphContext) {
    let mut parser = KleisParser::new(input);

    match parser.parse() {
        Ok(expr) => {
            // Try to evaluate
            match evaluator.eval(&expr) {
                Ok(result) => {
                    let rendered = render_expression(&result, ctx, &RenderTarget::Unicode);
                    println!("{}", rendered);
                }
                Err(_) => {
                    // Just show the parsed expression
                    let rendered = render_expression(&expr, ctx, &RenderTarget::Unicode);
                    println!("{}", rendered);
                }
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

#[cfg(feature = "axiom-verification")]
fn handle_command(
    line: &str,
    evaluator: &mut Evaluator,
    _ctx: &kleis::render::GlyphContext,
    registry: &StructureRegistry,
) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd {
        ":help" | ":h" => show_help(arg),
        ":quit" | ":q" => println!("Goodbye! 👋"),
        ":ast" => show_ast(arg),
        ":type" | ":t" => show_type(arg),
        ":verify" | ":v" => verify_expression(arg, registry, evaluator),
        ":load" | ":l" => load_file(arg, evaluator),
        ":env" | ":e" => show_env(evaluator),
        ":define" | ":def" => define_function(arg, evaluator),
        ":export" | ":x" => export_functions(arg, evaluator),
        ":syntax" | ":syn" => show_syntax(),
        ":examples" | ":ex" => show_examples(),
        ":symbols" | ":sym" => show_symbols(),
        _ => println!(
            "Unknown command: {}. Type :help for available commands.",
            cmd
        ),
    }
}

#[cfg(not(feature = "axiom-verification"))]
fn handle_command_no_z3(line: &str, evaluator: &mut Evaluator, _ctx: &kleis::render::GlyphContext) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd {
        ":help" | ":h" => show_help(arg),
        ":quit" | ":q" => println!("Goodbye! 👋"),
        ":ast" => show_ast(arg),
        ":type" | ":t" => show_type(arg),
        ":verify" | ":v" => {
            println!("⚠️  Z3 verification not available (compile with axiom-verification feature)")
        }
        ":syntax" | ":syn" => show_syntax(),
        ":examples" | ":ex" => show_examples(),
        ":symbols" | ":sym" => show_symbols(),
        ":load" | ":l" => load_file(arg, evaluator),
        ":env" | ":e" => show_env(evaluator),
        ":define" | ":def" => define_function(arg, evaluator),
        ":export" | ":x" => export_functions(arg, evaluator),
        _ => println!(
            "Unknown command: {}. Type :help for available commands.",
            cmd
        ),
    }
}

fn show_help(topic: &str) {
    if topic.is_empty() {
        print_help_main();
    } else {
        match topic.to_lowercase().as_str() {
            "quantifiers" | "quant" | "forall" | "exists" => print_help_quantifiers(),
            "operators" | "ops" => print_help_operators(),
            "types" => print_help_types(),
            "conditionals" | "if" | "let" => print_help_conditionals(),
            "functions" | "func" | "define" => print_help_functions(),
            "structures" | "struct" => print_help_structures(),
            "rust" | "java" | "programmers" | "prog" => print_help_for_programmers(),
            "adt" | "data" | "enum" => print_help_adt(),
            "match" | "pattern" => print_help_pattern_matching(),
            _ => {
                println!("Unknown help topic: {}", topic);
                println!("Available topics:");
                println!("  quantifiers, operators, types, conditionals, functions, structures");
                println!("  rust, java, programmers  - Guide for Rust/Java developers");
                println!("  adt, data, enum          - Algebraic data types");
                println!("  match, pattern           - Pattern matching");
            }
        }
    }
}

fn print_help_main() {
    println!();
    println!("📖 Kleis REPL Commands:");
    println!();
    println!("  :help, :h [topic]  Show help on a topic");
    println!("  :syntax, :syn      Complete Kleis syntax reference");
    println!("  :examples, :ex     Show example expressions");
    println!("  :symbols, :sym     Unicode math symbols palette (copy-paste!)");
    println!("  :quit, :q          Exit the REPL");
    println!();
    println!("  :ast <expr>        Show parsed AST");
    println!("  :type, :t <expr>   Show inferred type");
    println!("  :verify, :v <expr> Verify expression with Z3");
    println!("  :load, :l <file>   Load a .kleis file");
    println!("  :env, :e           Show defined functions");
    println!("  :define <def>      Define a function");
    println!("  :export, :x [file] Export definitions to .kleis (or stdout)");
    println!();
    println!("📝 Multi-line Input:");
    println!("  Method 1: End line with \\ (backslash)");
    println!("    λ> :verify ∀(a : R, b : R). \\");
    println!("       (a + b) * (a - b) = a * a - b * b");
    println!();
    println!("  Method 2: Use :{{ and :}} for block mode");
    println!("    λ> :{{");
    println!("       :verify ∀(x : R, y : R, z : R).");
    println!("         (x + y) + z = x + (y + z)");
    println!("       :}}");
    println!();
    println!("  Press Ctrl+C to cancel multi-line input");
    println!();
    println!("📚 Help Topics (:help <topic>):");
    println!("  quantifiers  - ∀ and ∃ syntax");
    println!("  operators    - Arithmetic, logic, set operators");
    println!("  types        - Type system (ℝ, ℤ, Matrix, etc.)");
    println!("  conditionals - if/then/else, let bindings");
    println!("  functions    - Function definitions");
    println!("  structures   - Algebraic structures");
    println!("  adt, data    - Algebraic data types");
    println!("  match        - Pattern matching");
    println!("  rust, java   - Guide for Rust/Java programmers");
    println!();
}

fn print_help_quantifiers() {
    println!();
    println!("📖 Quantifiers");
    println!("══════════════");
    println!();
    println!("  Universal (for all):");
    println!("    ∀(x : T). expression       Unicode forall");
    println!("    forall(x : T). expression  ASCII alternative");
    println!();
    println!("  Existential (there exists):");
    println!("    ∃(x : T). expression       Unicode exists");
    println!("    exists(x : T). expression  ASCII alternative");
    println!();
    println!("  Multiple variables:");
    println!("    ∀(x : R, y : R). x + y = y + x");
    println!("    ∀(x : R, y : R, z : R). (x + y) + z = x + (y + z)");
    println!();
    println!("  With constraints (where clause):");
    println!("    ∀(x : R) where x ≠ 0. x * (1/x) = 1");
    println!();
    println!("  Examples:");
    println!("    :verify ∀(x : R, y : R). x + y = y + x");
    println!("    :verify ∀(p : Bool, q : Bool). (p ∧ q) = (q ∧ p)");
    println!();
}

fn print_help_operators() {
    println!();
    println!("📖 Operators");
    println!("════════════");
    println!();
    println!("  Arithmetic:");
    println!("    +   Addition         x + y");
    println!("    -   Subtraction      x - y");
    println!("    *   Multiplication   x * y");
    println!("    /   Division         x / y");
    println!("    ^   Exponentiation   x ^ 2");
    println!();
    println!("  Comparison:");
    println!("    =   Equality         x = y");
    println!("    ≠   Not equal        x ≠ y  (or x != y)");
    println!("    <   Less than        x < y");
    println!("    >   Greater than     x > y");
    println!("    ≤   Less or equal    x ≤ y  (or x <= y)");
    println!("    ≥   Greater or equal x ≥ y  (or x >= y)");
    println!();
    println!("  Logical:");
    println!("    ∧   AND              p ∧ q  (or p and q)");
    println!("    ∨   OR               p ∨ q  (or p or q)");
    println!("    ¬   NOT              ¬p     (or not p)");
    println!("    →   Implies          p → q  (or p => q)");
    println!("    ↔   Iff              p ↔ q  (or p <=> q)");
    println!();
    println!("  Set/Collection:");
    println!("    ∈   Element of       x ∈ S");
    println!("    ∉   Not element of   x ∉ S");
    println!("    ⊂   Subset           A ⊂ B");
    println!("    ∪   Union            A ∪ B");
    println!("    ∩   Intersection     A ∩ B");
    println!();
    println!("  Special:");
    println!("    •   Generic binary   x • y  (for abstract algebra)");
    println!("    ∘   Composition      f ∘ g");
    println!();
}

fn print_help_types() {
    println!();
    println!("📖 Types");
    println!("════════");
    println!();
    println!("  Built-in types:");
    println!("    R, ℝ      Real numbers");
    println!("    Z, ℤ      Integers");
    println!("    N, ℕ      Natural numbers");
    println!("    Q, ℚ      Rationals");
    println!("    C, ℂ      Complex numbers");
    println!("    Bool      Booleans");
    println!();
    println!("  Parameterized types:");
    println!("    Vector(n)           n-dimensional vector");
    println!("    Matrix(m, n)        m×n matrix");
    println!("    Set(T)              Set of type T");
    println!("    List(T)             List of type T");
    println!();
    println!("  Type annotations:");
    println!("    x : R               Variable x has type R");
    println!("    f : R → R           Function from R to R");
    println!("    g : R × R → R       Binary function");
    println!();
}

fn print_help_conditionals() {
    println!();
    println!("📖 Conditionals & Let Bindings");
    println!("══════════════════════════════");
    println!();
    println!("  If-then-else:");
    println!("    if condition then expr1 else expr2");
    println!();
    println!("    Examples:");
    println!("      if x > 0 then x else 0 - x");
    println!("      if n = 0 then 1 else n * factorial(n - 1)");
    println!();
    println!("  Let bindings:");
    println!("    let name = value in body");
    println!();
    println!("    Examples:");
    println!("      let x = 5 in x * x");
    println!("      let a = 2 in let b = 3 in a + b");
    println!("      let sum = x + y in sum * sum");
    println!();
    println!("  Combined:");
    println!("    let abs = if x > 0 then x else 0 - x in abs * 2");
    println!();
}

fn print_help_functions() {
    println!();
    println!("📖 Functions");
    println!("════════════");
    println!();
    println!("  Define a function:");
    println!("    define name(params) = expression");
    println!();
    println!("    Examples:");
    println!("      define square(x) = x * x");
    println!("      define add(x, y) = x + y");
    println!("      define abs(x) = if x > 0 then x else 0 - x");
    println!();
    println!("  With type annotations:");
    println!("    define f(x : R) : R = x * x");
    println!();
    println!("  Recursive functions:");
    println!("    define factorial(n) = if n = 0 then 1 else n * factorial(n - 1)");
    println!();
    println!("  In REPL:");
    println!("    λ> :define square(x) = x * x");
    println!("    ✅ Defined: square");
    println!("    λ> square(5)");
    println!("    25");
    println!();
}

fn print_help_structures() {
    println!();
    println!("📖 Algebraic Structures");
    println!("═══════════════════════");
    println!();
    println!("  Structure definition:");
    println!("    structure Name(params) {{");
    println!("      carrier: Type");
    println!("      operation op : Type → Type → Type");
    println!("      axiom name: ∀(x : Type). property");
    println!("    }}");
    println!();
    println!("  Example - Monoid:");
    println!("    structure Monoid(M) {{");
    println!("      carrier: M");
    println!("      operation •  : M → M → M");
    println!("      constant  e  : M");
    println!("      axiom identity:    ∀(x : M). x • e = x");
    println!("      axiom associative: ∀(x y z : M). (x • y) • z = x • (y • z)");
    println!("    }}");
    println!();
    println!("  Example - Group:");
    println!("    structure Group(G) extends Monoid(G) {{");
    println!("      operation inv : G → G");
    println!("      axiom inverse: ∀(x : G). x • inv(x) = e");
    println!("    }}");
    println!();
}

fn print_help_for_programmers() {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    KLEIS FOR RUST AND JAVA PROGRAMMERS                         ");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("If you know Rust or Java, you already understand most of Kleis!");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ CONCEPT MAPPING                                                             │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Rust / Java                    │ Kleis                                      │");
    println!("├────────────────────────────────┼────────────────────────────────────────────┤");
    println!("│ trait / interface              │ structure                                  │");
    println!("│ impl / implements              │ implements                                 │");
    println!("│ enum / sealed class            │ data (ADT)                                 │");
    println!("│ match / switch                 │ match                                      │");
    println!("│ trait bounds / extends         │ constraints, kinds                         │");
    println!("│ generics                       │ polymorphic types, ∀                       │");
    println!("│ (none)                         │ axioms (laws!)                             │");
    println!("└────────────────────────────────┴────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ STRUCTURES = TRAITS / INTERFACES                                            │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Rust:   trait Add {{ fn add(self, other: Self) -> Self; }}                   │");
    println!("│ Java:   interface Add<T> {{ T add(T other); }}                               │");
    println!("│ Kleis:  structure Add(T) {{ operation add : T → T → T }}                     │");
    println!("│                                                                             │");
    println!("│ But Kleis adds AXIOMS:                                                      │");
    println!("│   structure Monoid(M) {{                                                     │");
    println!("│     operation (•) : M → M → M                                               │");
    println!("│     element e : M                                                           │");
    println!("│     axiom associativity: ∀(x y z : M). (x • y) • z = x • (y • z)            │");
    println!("│   }}                                                                         │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ IMPLEMENTS = IMPL BLOCKS                                                    │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!(
        "│ Rust:   impl Add for i32 {{ fn add(self, other: i32) -> i32 {{ self + other }} }}│"
    );
    println!("│ Java:   class MyInt implements Add<Integer> {{ ... }}                        │");
    println!("│ Kleis:  implements Add(ℝ) {{ operation add = builtin_add }}                  │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ DATA = ENUMS / SEALED CLASSES                                               │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Rust:   enum Option<T> {{ None, Some(T) }}                                   │");
    println!("│ Java:   sealed interface Option<T> permits None, Some {{ }}                  │");
    println!("│ Kleis:  data Option(T) = None | Some(T)                                     │");
    println!("│                                                                             │");
    println!("│ More examples:                                                              │");
    println!("│   data List(T)   = Nil | Cons(T, List(T))                                   │");
    println!("│   data Tree(T)   = Leaf(T) | Node(Tree(T), Tree(T))                         │");
    println!("│   data Either(A,B) = Left(A) | Right(B)                                     │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ PATTERN MATCHING                                                            │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Rust:   match x {{ Some(v) => v, None => 0 }}                                │");
    println!("│ Java:   switch(x) {{ case Some(var v) -> v; case None -> 0; }}               │");
    println!("│ Kleis:  match x {{ Some(v) => v | None => 0 }}                               │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ EXTENDS = TRAIT/INTERFACE INHERITANCE                                       │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Rust:   trait Group: Monoid {{ fn inv(&self) -> Self; }}                     │");
    println!("│ Java:   interface Group extends Monoid {{ T invert(T x); }}                  │");
    println!("│ Kleis:  structure Group(G) extends Monoid(G) {{ operation inv : G → G }}     │");
    println!("│                                                                             │");
    println!("│ Forms algebraic hierarchies:                                                │");
    println!("│   Semigroup ⊆ Monoid ⊆ Group ⊆ AbelianGroup                                 │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ GENERICS = QUANTIFIERS                                                      │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Rust:   fn norm<V: VectorSpace>(v: V) -> f64                                │");
    println!("│ Java:   <T extends VectorSpace<T>> double norm(T v)                         │");
    println!("│ Kleis:  operation norm : ∀(V : Type). VectorSpace(V) ⇒ V → ℝ                │");
    println!("│                                                                             │");
    println!("│ Kleis quantifiers (∀) generalize Rust/Java generics                         │");
    println!("│ Type inference is Hindley-Milner: types often inferred automatically        │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  Kleis is both MORE GENERAL than conventional languages and                 ");
    println!("  MORE PRECISE than typical proof assistants.                                ");
    println!();
}

fn print_help_adt() {
    println!();
    println!("📖 Algebraic Data Types (ADTs)");
    println!("══════════════════════════════");
    println!();
    println!("  ADTs define types with multiple variants (like Rust enums or Java sealed classes)");
    println!();
    println!("  Syntax:");
    println!("    data TypeName(params) = Variant1 | Variant2(fields) | ...");
    println!();
    println!("  Examples:");
    println!("    data Bool = True | False");
    println!("    data Option(T) = None | Some(T)");
    println!("    data Either(A, B) = Left(A) | Right(B)");
    println!("    data List(T) = Nil | Cons(T, List(T))");
    println!("    data Tree(T) = Leaf(T) | Node(Tree(T), Tree(T))");
    println!("    data Nat = Zero | Succ(Nat)");
    println!();
    println!("  Recursive types:");
    println!("    data Expr = Num(ℤ) | Add(Expr, Expr) | Mul(Expr, Expr)");
    println!();
    println!("  With multiple parameters:");
    println!("    data Result(T, E) = Ok(T) | Err(E)");
    println!("    data Map(K, V) = Empty | Entry(K, V, Map(K, V))");
    println!();
    println!("  Use with pattern matching:");
    println!("    match opt {{");
    println!("      Some(x) => x");
    println!("    | None    => default");
    println!("    }}");
    println!();
}

fn print_help_pattern_matching() {
    println!();
    println!("📖 Pattern Matching");
    println!("═══════════════════");
    println!();
    println!("  Deconstruct ADTs and match on structure:");
    println!();
    println!("  Basic syntax:");
    println!("    match expr {{");
    println!("      Pattern1 => result1");
    println!("    | Pattern2 => result2");
    println!("    | ...      => ...    ");
    println!("    }}");
    println!();
    println!("  Example with Option:");
    println!("    match opt {{");
    println!("      Some(x) => x * 2");
    println!("    | None    => 0");
    println!("    }}");
    println!();
    println!("  Example with List:");
    println!("    match list {{");
    println!("      Nil         => 0");
    println!("    | Cons(x, xs) => 1 + length(xs)");
    println!("    }}");
    println!();
    println!("  Nested patterns:");
    println!("    match pair {{");
    println!("      (Some(x), Some(y)) => x + y");
    println!("    | (Some(x), None)    => x");
    println!("    | (None, Some(y))    => y");
    println!("    | (None, None)       => 0");
    println!("    }}");
    println!();
    println!("  Wildcards:");
    println!("    match value {{");
    println!("      Specific(x) => handle(x)");
    println!("    | _           => default   // matches anything");
    println!("    }}");
    println!();
    println!("  Features:");
    println!("    • Exhaustiveness checking (all cases covered)");
    println!("    • Non-redundancy checking (no duplicate cases)");
    println!("    • Nested pattern matching");
    println!("    • Variable binding in patterns");
    println!();
}

fn show_syntax() {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                         KLEIS LANGUAGE SYNTAX REFERENCE                        ");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ EXPRESSIONS                                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Literals:      42, 3.14, -5, true, false                                    │");
    println!("│ Variables:     x, y, alpha, x₁, x_1                                         │");
    println!("│ Arithmetic:    x + y, x - y, x * y, x / y, x ^ n                            │");
    println!("│ Comparison:    x = y, x ≠ y, x < y, x > y, x ≤ y, x ≥ y                     │");
    println!("│ Logical:       p ∧ q, p ∨ q, ¬p, p → q, p ↔ q                               │");
    println!("│ Function call: f(x), g(x, y), sin(x)                                        │");
    println!("│ Parentheses:   (x + y) * z                                                  │");
    println!("│ Subscript:     x_i, a_{{i,j}}, M_{{m,n}}                                       │");
    println!("│ Superscript:   x^2, e^x, A^T                                                │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ CONDITIONALS & BINDINGS                                                     │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ If-then-else:  if condition then expr else expr                             │");
    println!("│ Let binding:   let x = value in body                                        │");
    println!("│ Match:         match expr {{ pattern => result, ... }}                        │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ QUANTIFIERS                                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Universal:     ∀(x : T). expr       forall(x : T). expr                     │");
    println!("│ Existential:   ∃(x : T). expr       exists(x : T). expr                     │");
    println!("│ Multi-var:     ∀(x : R, y : R). expr                                        │");
    println!("│ With where:    ∀(x : R) where x ≠ 0. expr                                   │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ DEFINITIONS                                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Function:      define f(x) = expr                                           │");
    println!("│ With types:    define f(x : R) : R = expr                                   │");
    println!("│ Multi-param:   define add(x, y) = x + y                                     │");
    println!("│ Constant:      define pi = 3.14159                                          │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ STRUCTURES (in .kleis files)                                                │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ structure Name(params) {{                                                    │");
    println!("│   carrier: Type                                                             │");
    println!("│   operation op : Type → Type                                                │");
    println!("│   constant  c  : Type                                                       │");
    println!("│   axiom name: ∀(x : T). property                                            │");
    println!("│ }}                                                                           │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ DATA TYPES (Algebraic Data Types)                                           │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ data Option(T) = None | Some(T)                                             │");
    println!("│ data List(T)   = Nil | Cons(T, List(T))                                     │");
    println!("│ data Tree(T)   = Leaf(T) | Node(Tree(T), Tree(T))                           │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  Type :help <topic> for details. Topics: quantifiers, operators, types,");
    println!("                                          conditionals, functions, structures");
    println!();
}

fn show_examples() {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                              KLEIS EXAMPLES                                    ");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ BASIC ARITHMETIC                                                            │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ λ> 2 + 3 * 4                                                                │");
    println!("│ λ> (1 + 2) ^ 3                                                              │");
    println!("│ λ> x + y - z                                                                │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ RING AXIOMS (Commutativity, Associativity, Distribution)                    │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ λ> :verify ∀(x : R, y : R). x + y = y + x                                   │");
    println!("│ ✅ Valid                                                                     │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(x : R, y : R, z : R). (x + y) + z = x + (y + z)                │");
    println!("│ ✅ Valid                                                                     │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(a : R, b : R, c : R). a * (b + c) = a * b + a * c              │");
    println!("│ ✅ Valid                                                                     │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ BOOLEAN ALGEBRA (De Morgan's Laws)                                          │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ λ> :verify ∀(p : Bool, q : Bool). ¬(p ∧ q) = (¬p ∨ ¬q)                      │");
    println!("│ ✅ Valid                                                                     │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(p : Bool, q : Bool). ¬(p ∨ q) = (¬p ∧ ¬q)                      │");
    println!("│ ✅ Valid                                                                     │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(p : Bool). ¬(¬p) = p                                           │");
    println!("│ ✅ Valid                                                                     │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ ALGEBRAIC IDENTITIES                                                        │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ λ> :verify ∀(a : R, b : R). (a + b) * (a - b) = a * a - b * b               │");
    println!("│ ✅ Valid   (Difference of squares)                                          │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(a : R, b : R). (a + b) * (a + b) = a*a + 2*a*b + b*b           │");
    println!("│ ✅ Valid   (Square of binomial)                                             │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(x : R). x * 0 = 0                                              │");
    println!("│ ✅ Valid   (Zero product)                                                   │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ CONDITIONALS & FUNCTIONS                                                    │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ λ> :define abs(x) = if x > 0 then x else 0 - x                              │");
    println!("│ ✅ Defined: abs                                                             │");
    println!("│                                                                             │");
    println!("│ λ> let x = 5 in x * x                                                       │");
    println!("│ 25                                                                          │");
    println!("│                                                                             │");
    println!("│ λ> let a = 3 in let b = 4 in a * a + b * b                                  │");
    println!("│ 25                                                                          │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ INVALID THEOREMS (Z3 finds counterexamples)                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ λ> :verify ∀(x : R). x + 1 = x                                              │");
    println!("│ ❌ Invalid - Counterexample: x -> 0                                         │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(x : R, y : R). x = y                                           │");
    println!("│ ❌ Invalid - Counterexample: x -> 0, y -> 1                                 │");
    println!("│                                                                             │");
    println!("│ λ> :verify ∀(a : R, b : R). a - b = b - a                                   │");
    println!("│ ❌ Invalid - Counterexample: a -> 1, b -> 0                                 │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
}

fn show_symbols() {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                         UNICODE MATH SYMBOLS PALETTE                           ");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("  Copy-paste these symbols directly into your expressions!");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ QUANTIFIERS                                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   ∀   forall (for all)          ∃   exists (there exists)                   │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ LOGICAL OPERATORS                                                           │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   ∧   and (logical AND)         ∨   or (logical OR)                         │");
    println!("│   ¬   not (negation)            →   implies                                 │");
    println!("│   ↔   iff (if and only if)      ⇒   implies (double arrow)                  │");
    println!("│   ⇔   iff (double arrow)                                                    │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ COMPARISON                                                                  │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   ≠   not equal (!=)            ≤   less or equal (<=)                      │");
    println!("│   ≥   greater or equal (>=)     ≡   equivalent                              │");
    println!("│   ≈   approximately equal       ≢   not equivalent                          │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ SET THEORY                                                                  │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   ∈   element of                ∉   not element of                          │");
    println!("│   ⊂   subset                    ⊃   superset                                │");
    println!("│   ⊆   subset or equal           ⊇   superset or equal                       │");
    println!("│   ∪   union                     ∩   intersection                            │");
    println!("│   ∅   empty set                 ℘   power set                               │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ NUMBER SETS                                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   ℕ   natural numbers           ℤ   integers                                │");
    println!("│   ℚ   rationals                 ℝ   real numbers                            │");
    println!("│   ℂ   complex numbers           𝔽   field                                   │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ ARITHMETIC & ALGEBRA                                                        │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   ×   times (multiplication)    ÷   division                                │");
    println!("│   ±   plus-minus                ∓   minus-plus                              │");
    println!("│   √   square root               ∛   cube root                               │");
    println!("│   ∞   infinity                  ∂   partial derivative                      │");
    println!("│   ∑   summation                 ∏   product                                 │");
    println!("│   ∫   integral                  ∮   line/contour integral                   │");
    println!("│   ∬   double integral           ∭   triple integral                         │");
    println!("│   ∯   surface integral          ∇   gradient (nabla)                        │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ ABSTRACT ALGEBRA                                                            │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   •   generic binary op         ∘   composition                             │");
    println!("│   ⊕   direct sum / xor          ⊗   tensor product                          │");
    println!("│   ⊖   symmetric difference      ⊙   dot product                             │");
    println!("│   ⟨⟩  angle brackets            ⟦⟧  semantic brackets                       │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ ARROWS                                                                      │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   →   right arrow (function)    ←   left arrow                              │");
    println!("│   ↦   maps to                   ⟼   long maps to                            │");
    println!("│   ⇒   double right arrow        ⇐   double left arrow                       │");
    println!("│   ⟹   implies (axioms)          ⟸   implied by                              │");
    println!("│   ↔   bidirectional             ⇔   double bidirectional                    │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ GREEK LETTERS (commonly used)                                               │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   α β γ δ ε ζ η θ ι κ λ μ ν ξ π ρ σ τ υ φ χ ψ ω                             │");
    println!("│   Α Β Γ Δ Ε Ζ Η Θ Ι Κ Λ Μ Ν Ξ Π Ρ Σ Τ Υ Φ Χ Ψ Ω                             │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ SUBSCRIPTS & SUPERSCRIPTS                                                   │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│   Subscripts:   ₀ ₁ ₂ ₃ ₄ ₅ ₆ ₇ ₈ ₉ ₊ ₋ ₌ ₍ ₎ ₐ ₑ ₒ ₓ                      │");
    println!("│   Superscripts: ⁰ ¹ ² ³ ⁴ ⁵ ⁶ ⁷ ⁸ ⁹ ⁺ ⁻ ⁼ ⁽ ⁾ ⁿ                            │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  💡 Tip: Most terminals support copy-paste. Select and copy any symbol above!");
    println!("  💡 Tip: On macOS, use Edit > Emoji & Symbols (Ctrl+Cmd+Space) for more.");
    println!("  💡 Tip: ASCII alternatives work too: forall, exists, and, or, not, <=, >=, !=");
    println!();
}

fn show_ast(input: &str) {
    if input.is_empty() {
        println!("Usage: :ast <expression>");
        return;
    }

    let mut parser = KleisParser::new(input);
    match parser.parse() {
        Ok(expr) => {
            println!("{:#?}", expr);
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

fn show_type(input: &str) {
    if input.is_empty() {
        println!("Usage: :type <expression>");
        return;
    }

    let mut parser = KleisParser::new(input);
    match parser.parse() {
        Ok(expr) => {
            // Use the TypeChecker to infer the type
            use kleis::type_checker::{TypeCheckResult, TypeChecker};

            let mut checker = match TypeChecker::with_stdlib() {
                Ok(tc) => tc,
                Err(e) => {
                    println!("⚠️  Type checker init failed: {}", e);
                    println!("Expression: {:?}", expr);
                    return;
                }
            };

            match checker.check(&expr) {
                TypeCheckResult::Success(ty) => {
                    println!("📐 Type: {}", ty);
                }
                TypeCheckResult::Polymorphic {
                    type_var,
                    available_types,
                } => {
                    println!("📐 Type: {} (polymorphic)", type_var);
                    if !available_types.is_empty() {
                        println!("   Could be: {}", available_types.join(", "));
                    }
                }
                TypeCheckResult::Error {
                    message,
                    suggestion,
                } => {
                    println!("⚠️  Type inference: {}", message);
                    if let Some(hint) = suggestion {
                        println!("   Hint: {}", hint);
                    }
                    println!("   Expression: {:?}", expr);
                }
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

#[cfg(feature = "axiom-verification")]
fn verify_expression(input: &str, registry: &StructureRegistry, evaluator: &Evaluator) {
    if input.is_empty() {
        println!("Usage: :verify <expression>");
        return;
    }

    // Use parse_proposition to support quantifiers (∀, ∃)
    let mut parser = KleisParser::new(input);
    match parser.parse_proposition() {
        Ok(expr) => {
            // Expand user-defined functions before verification
            let expanded = expand_user_functions(&expr, evaluator);

            match AxiomVerifier::new(registry) {
                Ok(mut verifier) => {
                    // Load ADT constructors as identity elements (e.g., TCP, UDP, ICMP)
                    verifier.load_adt_constructors(evaluator.get_adt_constructors().iter());

                    match verifier.verify_axiom(&expanded) {
                        Ok(result) => match result {
                            VerificationResult::Valid => {
                                println!("✅ Valid");
                            }
                            VerificationResult::Invalid { counterexample } => {
                                println!("❌ Invalid - Counterexample: {}", counterexample);
                            }
                            VerificationResult::Unknown => {
                                println!("❓ Unknown (Z3 couldn't determine)");
                            }
                            VerificationResult::Disabled => {
                                println!("⚠️  Verification disabled");
                            }
                        },
                        Err(e) => {
                            println!("❌ Verification error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to initialize verifier: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

/// Recursively expand user-defined functions in an expression
#[cfg(feature = "axiom-verification")]
fn expand_user_functions(
    expr: &kleis::ast::Expression,
    evaluator: &Evaluator,
) -> kleis::ast::Expression {
    use kleis::ast::Expression;

    match expr {
        Expression::Operation { name, args } => {
            // First, recursively expand args
            let expanded_args: Vec<Expression> = args
                .iter()
                .map(|a| expand_user_functions(a, evaluator))
                .collect();

            // Check if this is a user-defined function
            if let Some(closure) = evaluator.get_function(name) {
                if closure.params.len() == expanded_args.len() {
                    // Substitute parameters with arguments
                    let mut result = closure.body.clone();
                    for (param, arg) in closure.params.iter().zip(expanded_args.iter()) {
                        result = substitute_var(&result, param, arg);
                    }
                    // Recursively expand in case the body contains more function calls
                    return expand_user_functions(&result, evaluator);
                }
            }

            // Not a user function, return with expanded args
            Expression::Operation {
                name: name.clone(),
                args: expanded_args,
            }
        }
        Expression::Quantifier {
            quantifier,
            variables,
            where_clause,
            body,
        } => Expression::Quantifier {
            quantifier: quantifier.clone(),
            variables: variables.clone(),
            where_clause: where_clause
                .as_ref()
                .map(|w| Box::new(expand_user_functions(w, evaluator))),
            body: Box::new(expand_user_functions(body, evaluator)),
        },
        Expression::Conditional {
            condition,
            then_branch,
            else_branch,
        } => Expression::Conditional {
            condition: Box::new(expand_user_functions(condition, evaluator)),
            then_branch: Box::new(expand_user_functions(then_branch, evaluator)),
            else_branch: Box::new(expand_user_functions(else_branch, evaluator)),
        },
        Expression::Let {
            pattern,
            type_annotation,
            value,
            body,
        } => Expression::Let {
            pattern: pattern.clone(),
            type_annotation: type_annotation.clone(),
            value: Box::new(expand_user_functions(value, evaluator)),
            body: Box::new(expand_user_functions(body, evaluator)),
        },
        Expression::Match { scrutinee, cases } => {
            use kleis::ast::MatchCase;
            Expression::Match {
                scrutinee: Box::new(expand_user_functions(scrutinee, evaluator)),
                cases: cases
                    .iter()
                    .map(|c| MatchCase {
                        pattern: c.pattern.clone(),
                        guard: c
                            .guard
                            .as_ref()
                            .map(|g| expand_user_functions(g, evaluator)),
                        body: expand_user_functions(&c.body, evaluator),
                    })
                    .collect(),
            }
        }
        Expression::List(items) => Expression::List(
            items
                .iter()
                .map(|i| expand_user_functions(i, evaluator))
                .collect(),
        ),
        Expression::Lambda { params, body } => Expression::Lambda {
            params: params.clone(),
            body: Box::new(expand_user_functions(body, evaluator)),
        },
        Expression::Ascription {
            expr: inner,
            type_annotation,
        } => Expression::Ascription {
            expr: Box::new(expand_user_functions(inner, evaluator)),
            type_annotation: type_annotation.clone(),
        },
        // Leaf nodes - return as-is
        _ => expr.clone(),
    }
}

/// Substitute a variable name with an expression
#[cfg(feature = "axiom-verification")]
fn substitute_var(
    expr: &kleis::ast::Expression,
    var_name: &str,
    replacement: &kleis::ast::Expression,
) -> kleis::ast::Expression {
    use kleis::ast::Expression;

    match expr {
        Expression::Object(name) if name == var_name => replacement.clone(),
        Expression::Operation { name, args } => Expression::Operation {
            name: name.clone(),
            args: args
                .iter()
                .map(|a| substitute_var(a, var_name, replacement))
                .collect(),
        },
        Expression::Quantifier {
            quantifier,
            variables,
            where_clause,
            body,
        } => {
            // Don't substitute if this quantifier binds the same variable
            let binds_var = variables.iter().any(|v| v.name == var_name);
            if binds_var {
                expr.clone()
            } else {
                Expression::Quantifier {
                    quantifier: quantifier.clone(),
                    variables: variables.clone(),
                    where_clause: where_clause
                        .as_ref()
                        .map(|w| Box::new(substitute_var(w, var_name, replacement))),
                    body: Box::new(substitute_var(body, var_name, replacement)),
                }
            }
        }
        Expression::Conditional {
            condition,
            then_branch,
            else_branch,
        } => Expression::Conditional {
            condition: Box::new(substitute_var(condition, var_name, replacement)),
            then_branch: Box::new(substitute_var(then_branch, var_name, replacement)),
            else_branch: Box::new(substitute_var(else_branch, var_name, replacement)),
        },
        Expression::Let {
            pattern,
            type_annotation,
            value,
            body,
        } => {
            // Don't substitute in body if pattern binds the same variable
            let binds_var = pattern_binds_var(pattern, var_name);
            if binds_var {
                Expression::Let {
                    pattern: pattern.clone(),
                    type_annotation: type_annotation.clone(),
                    value: Box::new(substitute_var(value, var_name, replacement)),
                    body: body.clone(),
                }
            } else {
                Expression::Let {
                    pattern: pattern.clone(),
                    type_annotation: type_annotation.clone(),
                    value: Box::new(substitute_var(value, var_name, replacement)),
                    body: Box::new(substitute_var(body, var_name, replacement)),
                }
            }
        }
        Expression::Match { scrutinee, cases } => {
            use kleis::ast::MatchCase;
            Expression::Match {
                scrutinee: Box::new(substitute_var(scrutinee, var_name, replacement)),
                cases: cases
                    .iter()
                    .map(|c| {
                        // Check if pattern binds this variable - if so, don't substitute in body
                        let binds_var = pattern_binds_var(&c.pattern, var_name);
                        MatchCase {
                            pattern: c.pattern.clone(),
                            guard: if binds_var {
                                c.guard.clone()
                            } else {
                                c.guard
                                    .as_ref()
                                    .map(|g| substitute_var(g, var_name, replacement))
                            },
                            body: if binds_var {
                                c.body.clone()
                            } else {
                                substitute_var(&c.body, var_name, replacement)
                            },
                        }
                    })
                    .collect(),
            }
        }
        Expression::List(items) => Expression::List(
            items
                .iter()
                .map(|i| substitute_var(i, var_name, replacement))
                .collect(),
        ),
        Expression::Lambda { params, body } => {
            // Don't substitute in body if lambda binds the same variable
            let shadows = params.iter().any(|p| p.name == var_name);
            if shadows {
                expr.clone()
            } else {
                Expression::Lambda {
                    params: params.clone(),
                    body: Box::new(substitute_var(body, var_name, replacement)),
                }
            }
        }
        Expression::Ascription {
            expr: inner,
            type_annotation,
        } => Expression::Ascription {
            expr: Box::new(substitute_var(inner, var_name, replacement)),
            type_annotation: type_annotation.clone(),
        },
        // Leaf nodes - return as-is
        _ => expr.clone(),
    }
}

/// Check if a pattern binds a variable name (Grammar v0.8: handles As-patterns)
#[cfg(feature = "axiom-verification")]
fn pattern_binds_var(pattern: &kleis::ast::Pattern, var_name: &str) -> bool {
    use kleis::ast::Pattern;
    match pattern {
        Pattern::Variable(name) => name == var_name,
        Pattern::Constructor { args, .. } => args.iter().any(|p| pattern_binds_var(p, var_name)),
        Pattern::Wildcard | Pattern::Constant(_) => false,
        Pattern::As { pattern, binding } => {
            binding == var_name || pattern_binds_var(pattern, var_name)
        }
    }
}

fn load_file(path: &str, evaluator: &mut Evaluator) {
    if path.is_empty() {
        println!("Usage: :load <file.kleis>");
        return;
    }

    let mut loaded_files: HashSet<PathBuf> = HashSet::new();
    let base_path = Path::new(path);

    match load_file_recursive(base_path, evaluator, &mut loaded_files) {
        Ok(stats) => {
            println!(
                "✅ Loaded: {} files, {} functions, {} structures, {} data types, {} type aliases",
                stats.files,
                stats.functions,
                stats.structures,
                stats.data_types,
                stats.type_aliases
            );
        }
        Err(e) => {
            println!("❌ {}", e);
        }
    }
}

/// Stats for reporting what was loaded
struct LoadStats {
    files: usize,
    functions: usize,
    structures: usize,
    data_types: usize,
    type_aliases: usize,
}

impl LoadStats {
    fn new() -> Self {
        LoadStats {
            files: 0,
            functions: 0,
            structures: 0,
            data_types: 0,
            type_aliases: 0,
        }
    }

    fn add(&mut self, other: &LoadStats) {
        self.files += other.files;
        self.functions += other.functions;
        self.structures += other.structures;
        self.data_types += other.data_types;
        self.type_aliases += other.type_aliases;
    }
}

/// Recursively load a .kleis file and its imports
fn load_file_recursive(
    path: &Path,
    evaluator: &mut Evaluator,
    loaded_files: &mut HashSet<PathBuf>,
) -> Result<LoadStats, String> {
    // Resolve to canonical path for circular import detection
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    // Check for circular imports
    if loaded_files.contains(&canonical) {
        // Already loaded, skip (not an error, just avoid reloading)
        return Ok(LoadStats::new());
    }
    loaded_files.insert(canonical.clone());

    // Read file contents
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("File error '{}': {}", path.display(), e))?;

    // Parse the program
    let program = parse_kleis_program(&contents)
        .map_err(|e| format!("Parse error in '{}': {}", path.display(), e))?;

    let mut stats = LoadStats::new();
    stats.files = 1;

    // Get the directory containing this file for resolving relative imports
    let base_dir = path.parent().unwrap_or(Path::new("."));

    // Process imports first (depth-first)
    for item in &program.items {
        if let TopLevel::Import(import_path) = item {
            let resolved_path = resolve_import_path(import_path, base_dir);
            match load_file_recursive(&resolved_path, evaluator, loaded_files) {
                Ok(import_stats) => {
                    stats.add(&import_stats);
                }
                Err(e) => {
                    return Err(format!(
                        "Error loading import '{}' from '{}': {}",
                        import_path,
                        path.display(),
                        e
                    ));
                }
            }
        }
    }

    // Now load this file's definitions into evaluator
    if let Err(e) = evaluator.load_program(&program) {
        return Err(format!(
            "Error loading definitions from '{}': {}",
            path.display(),
            e
        ));
    }

    stats.functions += program.functions().len();
    stats.structures += program.structures().len();
    stats.data_types += program.data_types().len();
    stats.type_aliases += program.type_aliases().len();

    Ok(stats)
}

/// Resolve an import path relative to the base directory
fn resolve_import_path(import_path: &str, base_dir: &Path) -> PathBuf {
    let import = Path::new(import_path);

    if import.is_absolute() {
        // Absolute path: use as-is
        import.to_path_buf()
    } else if import_path.starts_with("stdlib/") {
        // Standard library path: resolve from project root or known stdlib location
        // For now, try relative to current working directory
        // TODO: Support KLEIS_STDLIB_PATH environment variable
        PathBuf::from(import_path)
    } else {
        // Relative path: resolve from base directory
        base_dir.join(import)
    }
}

fn show_env(evaluator: &Evaluator) {
    let functions = evaluator.list_functions();
    if functions.is_empty() {
        println!("No functions defined.");
    } else {
        println!("📋 Defined functions:");
        for name in functions {
            if let Some(closure) = evaluator.get_function(&name) {
                let params = closure.params.join(", ");
                println!("  {} ({}) = ...", name, params);
            }
        }
    }
}

fn define_function(input: &str, evaluator: &mut Evaluator) {
    if input.is_empty() {
        println!("Usage: :define name(params) = expression");
        println!("   or just type: define name(params) = expression");
        return;
    }

    // Prepend "define " if not present
    let full_input = if input.starts_with("define ") {
        input.to_string()
    } else {
        format!("define {}", input)
    };

    match parse_kleis_program(&full_input) {
        Ok(program) => {
            if let Err(e) = evaluator.load_program(&program) {
                println!("❌ Error: {}", e);
            } else if !program.functions().is_empty() {
                let func = &program.functions()[0];
                println!("✅ Defined: {}", func.name);
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

/// Export all defined functions to a .kleis file or stdout
fn export_functions(path: &str, evaluator: &Evaluator) {
    let pp = PrettyPrinter::new();
    let functions = evaluator.list_functions();
    let data_types = evaluator.get_data_types();
    let structures = evaluator.get_structures();

    if functions.is_empty() && data_types.is_empty() && structures.is_empty() {
        println!("No definitions to export.");
        return;
    }

    // Sort functions alphabetically for consistent output
    let mut sorted_functions = functions;
    sorted_functions.sort();

    // Generate the output
    let mut output = String::new();
    output.push_str("// Exported from Kleis REPL\n");

    // Header with counts
    let mut counts = Vec::new();
    if !structures.is_empty() {
        counts.push(format!("{} structure(s)", structures.len()));
    }
    if !data_types.is_empty() {
        counts.push(format!("{} data type(s)", data_types.len()));
    }
    if !sorted_functions.is_empty() {
        counts.push(format!("{} function(s)", sorted_functions.len()));
    }
    output.push_str(&format!("// {}\n\n", counts.join(", ")));

    // Export structures first (they define types and axioms)
    for structure in structures {
        output.push_str(&pp.format_structure(structure));
        output.push_str("\n\n");
    }

    // Export data types (they define constructors used by functions)
    for data_def in data_types {
        output.push_str(&pp.format_data_def(data_def));
        output.push_str("\n\n");
    }

    // Export functions
    for name in &sorted_functions {
        if let Some(closure) = evaluator.get_function(name) {
            output.push_str(&pp.format_function(name, closure));
            output.push_str("\n\n");
        }
    }

    if path.is_empty() {
        // Print to stdout
        println!();
        println!("═══════════════════════════════════════════════════════════════════════════════");
        println!("                         EXPORTED KLEIS DEFINITIONS                            ");
        println!("═══════════════════════════════════════════════════════════════════════════════");
        println!();
        print!("{}", output);
        println!("═══════════════════════════════════════════════════════════════════════════════");
    } else {
        // Write to file
        let file_path = if path.ends_with(".kleis") {
            path.to_string()
        } else {
            format!("{}.kleis", path)
        };

        match std::fs::write(&file_path, &output) {
            Ok(_) => {
                let total = structures.len() + data_types.len() + sorted_functions.len();
                println!("✅ Exported {} definition(s) to {}", total, file_path);
            }
            Err(e) => {
                println!("❌ Error writing file: {}", e);
            }
        }
    }
}
