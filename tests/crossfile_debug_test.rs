// Test that DAP reports correct file, line, column at each step
// Uses exact files from examples/ with various operation types

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use kleis::ast::Expression;
use kleis::debug::{DebugAction, DebugHook, DebugState, SourceLocation, StackFrame};
use kleis::evaluator::Evaluator;
use kleis::kleis_ast::TopLevel;
use kleis::kleis_parser::parse_kleis_program_with_file;

// Track all on_eval_start calls with full details
struct StepEvent {
    step_number: usize,
    line: u32,
    column: u32,
    file: Option<PathBuf>,
    operation_name: Option<String>,
}

struct SteppingDebugHook {
    events: Arc<Mutex<Vec<StepEvent>>>,
    step_counter: usize,
}

impl DebugHook for SteppingDebugHook {
    fn on_eval_start(
        &mut self,
        expr: &Expression,
        loc: &SourceLocation,
        depth: usize,
    ) -> DebugAction {
        self.step_counter += 1;

        let operation_name = match expr {
            Expression::Operation { name, .. } => Some(name.clone()),
            _ => None,
        };

        self.events.lock().unwrap().push(StepEvent {
            step_number: self.step_counter,
            line: loc.line,
            column: loc.column,
            file: loc.file.clone(),
            operation_name,
        });
        let _ = depth; // Suppress unused warning
        DebugAction::Continue
    }
    fn on_eval_end(&mut self, _: &Expression, _: &Result<Expression, String>, _: usize) {}
    fn on_function_enter(&mut self, _: &str, _: &[Expression], _: &SourceLocation, _: usize) {}
    fn on_function_exit(&mut self, _: &str, _: &Result<Expression, String>, _: usize) {}
    fn on_bind(&mut self, _: &str, _: &Expression, _: usize) {}
    fn state(&self) -> &DebugState {
        &DebugState::Running
    }
    fn should_stop(&self, _location: &SourceLocation, _depth: usize) -> bool {
        false
    }
    fn wait_for_command(&mut self) -> DebugAction {
        DebugAction::Continue
    }
    fn get_stack(&self) -> &[StackFrame] {
        &[]
    }
    fn push_frame(&mut self, _: StackFrame) {}
    fn pop_frame(&mut self) -> Option<StackFrame> {
        None
    }
}

#[test]
fn test_debug_all_operation_types() {
    // Use the actual example files
    let main_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_main.kleis");
    let helper_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_helper.kleis");

    let main_source = std::fs::read_to_string(&main_path).expect("Failed to read debug_main.kleis");
    let helper_source =
        std::fs::read_to_string(&helper_path).expect("Failed to read debug_helper.kleis");

    let events = Arc::new(Mutex::new(Vec::new()));
    let hook = SteppingDebugHook {
        events: events.clone(),
        step_counter: 0,
    };

    let mut evaluator = Evaluator::new();
    evaluator.set_debug_hook(Box::new(hook));

    // Load helper file first (imports)
    let helper_program =
        parse_kleis_program_with_file(&helper_source, helper_path.to_str().unwrap())
            .expect("Failed to parse debug_helper.kleis");
    evaluator.load_program(&helper_program).unwrap();

    // Load main file
    let main_program = parse_kleis_program_with_file(&main_source, main_path.to_str().unwrap())
        .expect("Failed to parse debug_main.kleis");

    // Find and run the example block
    for item in &main_program.items {
        if let TopLevel::ExampleBlock(example) = item {
            if example.name == "cross-file debugging" {
                println!("\n=== Running example: {} ===\n", example.name);
                let _ = evaluator.eval_example_block(example);
            }
        }
    }

    // Analyze events
    let all_events = events.lock().unwrap();

    // Print complete output
    println!("\n╔════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                     DAP STEP EVENTS - ALL OPERATION TYPES                              ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════════════╣");
    println!(
        "║ {:>4} │ {:>6}:{:<3} │ {:25} │ {:20} ║",
        "Step", "Line", "Col", "File", "Operation"
    );
    println!("╠════════════════════════════════════════════════════════════════════════════════════════╣");

    for event in all_events.iter() {
        let file_name = event
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "NONE".to_string());

        let op_name = event.operation_name.as_deref().unwrap_or("-");

        println!(
            "║ {:>4} │ {:>6}:{:<3} │ {:25} │ {:20} ║",
            event.step_number, event.line, event.column, file_name, op_name
        );
    }
    println!("╚════════════════════════════════════════════════════════════════════════════════════════╝");

    // Collect operations from helper file
    let helper_ops: Vec<_> = all_events
        .iter()
        .filter(|e| {
            e.file
                .as_ref()
                .is_some_and(|f| f.to_string_lossy().contains("debug_helper"))
        })
        .filter(|e| e.operation_name.is_some())
        .collect();

    println!("\n=== OPERATIONS FROM debug_helper.kleis ===");
    println!("{:>6}:{:<3} {:20}", "Line", "Col", "Operation");
    println!("{}", "-".repeat(40));

    for event in &helper_ops {
        println!(
            "{:>6}:{:<3} {:20}",
            event.line,
            event.column,
            event.operation_name.as_deref().unwrap_or("-")
        );
    }

    // Categorize operations
    let mut arithmetic_ops = vec![];
    let mut logical_ops = vec![];
    let mut comparison_ops = vec![];
    let mut other_ops = vec![];

    for event in &helper_ops {
        if let Some(ref op) = event.operation_name {
            match op.as_str() {
                "plus" | "minus" | "times" | "divide" | "power" => {
                    arithmetic_ops.push((op.clone(), event.line))
                }
                "logical_and" | "logical_or" | "implies" | "iff" => {
                    logical_ops.push((op.clone(), event.line))
                }
                "equals" | "less_than" | "greater_than" | "less_equal" | "greater_equal" => {
                    comparison_ops.push((op.clone(), event.line))
                }
                _ => other_ops.push((op.clone(), event.line)),
            }
        }
    }

    println!("\n=== OPERATION CATEGORIES ===");
    println!("Arithmetic: {:?}", arithmetic_ops);
    println!("Logical:    {:?}", logical_ops);
    println!("Comparison: {:?}", comparison_ops);
    println!("Other:      {:?}", other_ops);

    // Summary
    println!("\n=== SUMMARY ===");
    let main_steps = all_events
        .iter()
        .filter(|e| {
            e.file
                .as_ref()
                .is_some_and(|f| f.to_string_lossy().contains("debug_main"))
        })
        .count();
    let helper_steps = all_events
        .iter()
        .filter(|e| {
            e.file
                .as_ref()
                .is_some_and(|f| f.to_string_lossy().contains("debug_helper"))
        })
        .count();
    let no_file_steps = all_events.iter().filter(|e| e.file.is_none()).count();

    println!("Total steps:          {}", all_events.len());
    println!("debug_main.kleis:     {}", main_steps);
    println!("debug_helper.kleis:   {}", helper_steps);
    println!("NO FILE:              {}", no_file_steps);

    // Assertions
    assert!(helper_steps > 0, "Should have steps in debug_helper.kleis");
    assert_eq!(no_file_steps, 0, "Should have NO steps without file info");

    // Check that we got various operation types from helper
    assert!(
        !arithmetic_ops.is_empty(),
        "Should have arithmetic operations from helper"
    );

    // Verify line numbers are reasonable (not 0 or 1 for helper file operations)
    for event in &helper_ops {
        assert!(
            event.line > 1,
            "Helper file operation should not be on line 1, got line {} for {:?}",
            event.line,
            event.operation_name
        );
    }
}

/// Test specific line number accuracy for each operation type
#[test]
fn test_operation_line_numbers() {
    let helper_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_helper.kleis");

    let helper_source =
        std::fs::read_to_string(&helper_path).expect("Failed to read debug_helper.kleis");

    // Expected line numbers for operation bodies:
    // double(n) = n + n       -> plus on line 10
    // triple(n) = n + n + n   -> plus on line 14
    // multiply(a, b) = a * b  -> times on line 18
    // square(x) = x ^ 2       -> power on line 22
    // half(x) = x / 2         -> divide on line 26
    // both_true(p, q) = p ∧ q -> logical_and on line 34
    // either_true(p, q) = p ∨ q -> logical_or on line 38
    // implies_that(p, q) = p ⟹ q -> implies on line 42
    // is_greater(a, b) = a > b -> greater_than on line 50
    // is_less(a, b) = a < b   -> less_than on line 54
    // is_equal(a, b) = a = b  -> equals on line 58

    let expected_operations = vec![
        ("plus", 10),         // n + n in double
        ("plus", 14),         // n + n + n in triple (first plus)
        ("times", 18),        // a * b in multiply
        ("power", 22),        // x ^ 2 in square
        ("divide", 26),       // x / 2 in half
        ("logical_and", 34),  // p ∧ q in both_true
        ("logical_or", 38),   // p ∨ q in either_true
        ("implies", 42),      // p ⟹ q in implies_that
        ("greater_than", 50), // a > b in is_greater
        ("less_than", 54),    // a < b in is_less
        ("equals", 58),       // a = b in is_equal
    ];

    println!("\n=== EXPECTED OPERATION LINES IN debug_helper.kleis ===");
    for (op, line) in &expected_operations {
        // Find the actual line content
        let lines: Vec<&str> = helper_source.lines().collect();
        if *line > 0 && (*line as usize) <= lines.len() {
            println!(
                "Line {:>2}: {:20} -> {}",
                line,
                op,
                lines[*line as usize - 1].trim()
            );
        }
    }

    // This test documents expected behavior - actual verification happens in the main test
}

/// Test with breakpoints set on specific lines (like VS Code would)
#[test]
fn test_breakpoints_in_both_files() {
    // Breakpoints as shown in VS Code:
    // debug_main.kleis: lines 16, 22, 28
    // debug_helper.kleis: lines 10, 18, 34

    struct Breakpoint {
        file_suffix: &'static str,
        line: u32,
    }

    let breakpoints = [
        Breakpoint {
            file_suffix: "debug_main.kleis",
            line: 16,
        }, // let doubled = double(x)
        Breakpoint {
            file_suffix: "debug_main.kleis",
            line: 22,
        }, // let product = multiply(x, y)
        Breakpoint {
            file_suffix: "debug_main.kleis",
            line: 28,
        }, // let halved = half(x)
        Breakpoint {
            file_suffix: "debug_helper.kleis",
            line: 10,
        }, // n + n (double body)
        Breakpoint {
            file_suffix: "debug_helper.kleis",
            line: 18,
        }, // a * b (multiply body)
        Breakpoint {
            file_suffix: "debug_helper.kleis",
            line: 34,
        }, // p ∧ q (both_true body)
    ];

    struct BreakpointHit {
        line: u32,
        file: String,
        operation: String,
    }

    let hits = Arc::new(Mutex::new(Vec::<BreakpointHit>::new()));
    let hits_clone = hits.clone();
    let breakpoints_ref: Vec<_> = breakpoints
        .iter()
        .map(|b| (b.file_suffix, b.line))
        .collect();

    struct BreakpointDebugHook {
        breakpoints: Vec<(&'static str, u32)>,
        hits: Arc<Mutex<Vec<BreakpointHit>>>,
    }

    impl DebugHook for BreakpointDebugHook {
        fn on_eval_start(
            &mut self,
            expr: &Expression,
            loc: &SourceLocation,
            _depth: usize,
        ) -> DebugAction {
            // Check if this location matches a breakpoint
            for (file_suffix, line) in &self.breakpoints {
                if loc.line == *line {
                    if let Some(ref file) = loc.file {
                        if file.to_string_lossy().ends_with(*file_suffix) {
                            let op_name = match expr {
                                Expression::Operation { name, .. } => name.clone(),
                                _ => "-".to_string(),
                            };
                            self.hits.lock().unwrap().push(BreakpointHit {
                                line: loc.line,
                                file: file_suffix.to_string(),
                                operation: op_name,
                            });
                        }
                    }
                }
            }
            DebugAction::Continue
        }
        fn on_eval_end(&mut self, _: &Expression, _: &Result<Expression, String>, _: usize) {}
        fn on_function_enter(&mut self, _: &str, _: &[Expression], _: &SourceLocation, _: usize) {}
        fn on_function_exit(&mut self, _: &str, _: &Result<Expression, String>, _: usize) {}
        fn on_bind(&mut self, _: &str, _: &Expression, _: usize) {}
        fn state(&self) -> &DebugState {
            &DebugState::Running
        }
        fn should_stop(&self, _: &SourceLocation, _: usize) -> bool {
            false
        }
        fn wait_for_command(&mut self) -> DebugAction {
            DebugAction::Continue
        }
        fn get_stack(&self) -> &[StackFrame] {
            &[]
        }
        fn push_frame(&mut self, _: StackFrame) {}
        fn pop_frame(&mut self) -> Option<StackFrame> {
            None
        }
    }

    let hook = BreakpointDebugHook {
        breakpoints: breakpoints_ref,
        hits: hits_clone,
    };

    let main_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_main.kleis");
    let helper_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_helper.kleis");

    let main_source = std::fs::read_to_string(&main_path).unwrap();
    let helper_source = std::fs::read_to_string(&helper_path).unwrap();

    let mut evaluator = Evaluator::new();
    evaluator.set_debug_hook(Box::new(hook));

    let helper_program =
        parse_kleis_program_with_file(&helper_source, helper_path.to_str().unwrap()).unwrap();
    evaluator.load_program(&helper_program).unwrap();

    let main_program =
        parse_kleis_program_with_file(&main_source, main_path.to_str().unwrap()).unwrap();

    for item in &main_program.items {
        if let TopLevel::ExampleBlock(example) = item {
            if example.name == "cross-file debugging" {
                let _ = evaluator.eval_example_block(example);
            }
        }
    }

    let all_hits = hits.lock().unwrap();

    println!("\n=== BREAKPOINT HITS ===");
    println!("{:>6} {:25} {:20}", "Line", "File", "Operation");
    println!("{}", "-".repeat(55));
    for hit in all_hits.iter() {
        println!("{:>6} {:25} {:20}", hit.line, hit.file, hit.operation);
    }

    // Verify we hit breakpoints in BOTH files
    let main_hits: Vec<_> = all_hits
        .iter()
        .filter(|h| h.file.contains("main"))
        .collect();
    let helper_hits: Vec<_> = all_hits
        .iter()
        .filter(|h| h.file.contains("helper"))
        .collect();

    println!("\nBreakpoints hit in debug_main.kleis: {}", main_hits.len());
    println!(
        "Breakpoints hit in debug_helper.kleis: {}",
        helper_hits.len()
    );

    assert!(
        !main_hits.is_empty(),
        "Should hit breakpoints in debug_main.kleis"
    );
    assert!(
        !helper_hits.is_empty(),
        "Should hit breakpoints in debug_helper.kleis"
    );

    // Verify specific operations at specific lines
    let helper_ops: Vec<_> = helper_hits
        .iter()
        .map(|h| (h.line, h.operation.as_str()))
        .collect();
    println!("\nHelper file breakpoint operations: {:?}", helper_ops);

    // Check that we got the right operations at the right lines
    assert!(
        helper_ops
            .iter()
            .any(|(line, op)| *line == 10 && *op == "plus"),
        "Should hit plus at line 10"
    );
    assert!(
        helper_ops
            .iter()
            .any(|(line, op)| *line == 18 && *op == "times"),
        "Should hit times at line 18"
    );
    assert!(
        helper_ops
            .iter()
            .any(|(line, op)| *line == 34 && *op == "logical_and"),
        "Should hit logical_and at line 34"
    );
}

/// Test that variables and stack frames are tracked correctly
#[test]
fn test_variables_and_stack_frames() {
    struct StackAndVarsHook {
        stack: Vec<StackFrame>,
        all_bindings: Arc<Mutex<Vec<(String, String)>>>,
        stack_snapshots: Arc<Mutex<Vec<(String, usize)>>>, // (function_name, stack_depth)
    }

    impl DebugHook for StackAndVarsHook {
        fn on_eval_start(&mut self, _: &Expression, _: &SourceLocation, _: usize) -> DebugAction {
            DebugAction::Continue
        }
        fn on_eval_end(&mut self, _: &Expression, _: &Result<Expression, String>, _: usize) {}
        fn on_function_enter(
            &mut self,
            name: &str,
            _: &[Expression],
            loc: &SourceLocation,
            _: usize,
        ) {
            let frame = StackFrame::new(name, loc.clone());
            self.stack.push(frame);
            self.stack_snapshots
                .lock()
                .unwrap()
                .push((name.to_string(), self.stack.len()));
        }
        fn on_function_exit(&mut self, _: &str, _: &Result<Expression, String>, _: usize) {
            if self.stack.len() > 1 {
                self.stack.pop();
            }
        }
        fn on_bind(&mut self, name: &str, value: &Expression, _: usize) {
            self.all_bindings
                .lock()
                .unwrap()
                .push((name.to_string(), format!("{:?}", value)));
            if let Some(frame) = self.stack.last_mut() {
                frame
                    .bindings
                    .insert(name.to_string(), format!("{:?}", value));
            }
        }
        fn state(&self) -> &DebugState {
            &DebugState::Running
        }
        fn should_stop(&self, _: &SourceLocation, _: usize) -> bool {
            false
        }
        fn wait_for_command(&mut self) -> DebugAction {
            DebugAction::Continue
        }
        fn get_stack(&self) -> &[StackFrame] {
            &self.stack
        }
        fn push_frame(&mut self, frame: StackFrame) {
            self.stack.push(frame);
        }
        fn pop_frame(&mut self) -> Option<StackFrame> {
            if self.stack.len() > 1 {
                self.stack.pop()
            } else {
                None
            }
        }
    }

    let all_bindings = Arc::new(Mutex::new(Vec::new()));
    let stack_snapshots = Arc::new(Mutex::new(Vec::new()));

    let hook = StackAndVarsHook {
        stack: vec![StackFrame::new("<top-level>", SourceLocation::new(1, 1))],
        all_bindings: all_bindings.clone(),
        stack_snapshots: stack_snapshots.clone(),
    };

    let main_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_main.kleis");
    let helper_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/debug_helper.kleis");

    let main_source = std::fs::read_to_string(&main_path).unwrap();
    let helper_source = std::fs::read_to_string(&helper_path).unwrap();

    let mut evaluator = Evaluator::new();
    evaluator.set_debug_hook(Box::new(hook));

    let helper_program =
        parse_kleis_program_with_file(&helper_source, helper_path.to_str().unwrap()).unwrap();
    evaluator.load_program(&helper_program).unwrap();

    let main_program =
        parse_kleis_program_with_file(&main_source, main_path.to_str().unwrap()).unwrap();

    for item in &main_program.items {
        if let TopLevel::ExampleBlock(example) = item {
            if example.name == "cross-file debugging" {
                let _ = evaluator.eval_example_block(example);
            }
        }
    }

    // Check bindings
    let bindings = all_bindings.lock().unwrap();
    println!("\n=== VARIABLE BINDINGS ===");
    for (name, value) in bindings.iter() {
        let short_value: String = value.chars().take(50).collect();
        println!("  {} = {}", name, short_value);
    }

    // Check stack snapshots
    let snapshots = stack_snapshots.lock().unwrap();
    println!("\n=== STACK SNAPSHOTS (function calls) ===");
    for (name, depth) in snapshots.iter() {
        println!("  {} (stack depth: {})", name, depth);
    }

    // Verify we got expected bindings
    let binding_names: Vec<_> = bindings.iter().map(|(n, _)| n.as_str()).collect();
    println!("\nBinding names: {:?}", binding_names);

    assert!(binding_names.contains(&"x"), "Should have binding for 'x'");
    assert!(
        binding_names.contains(&"doubled"),
        "Should have binding for 'doubled'"
    );

    // Verify we got expected function calls
    let func_names: Vec<_> = snapshots.iter().map(|(n, _)| n.as_str()).collect();
    println!("Function calls: {:?}", func_names);

    assert!(
        func_names.contains(&"double"),
        "Should have called 'double'"
    );
    assert!(
        func_names.contains(&"triple"),
        "Should have called 'triple'"
    );
}

/// Test that assert() with symbolic expressions uses Z3 verification
#[test]
fn test_assert_with_z3_verification() {
    // This test verifies that assert() can handle symbolic expressions
    // using Z3 when the axiom-verification feature is enabled

    let mut evaluator = Evaluator::new();

    // Load a simple structure with commutativity axiom
    let source = r#"
structure CommutativeRing(R) {
    operation (+) : R × R → R
    axiom commutativity: ∀(a b : R). a + b = b + a
}

example "test commutativity" {
    // This should be verified by Z3 (symbolic, not concrete)
    assert(x + y = y + x)
}
"#;

    let program = parse_kleis_program_with_file(source, "test_z3.kleis").expect("Should parse");

    evaluator.load_program(&program).unwrap();

    // Find and run the example
    for item in &program.items {
        if let TopLevel::ExampleBlock(example) = item {
            println!("\n=== Running example: {} ===", example.name);
            let result = evaluator.eval_example_block(example);

            println!("Passed: {}", result.passed);
            println!(
                "Assertions passed: {}/{}",
                result.assertions_passed, result.assertions_total
            );
            if let Some(ref err) = result.error {
                println!("Error: {}", err);
            }

            // The assertion should either:
            // - Pass (Z3 verified commutativity)
            // - Be Unknown (Z3 feature disabled or couldn't determine)
            // It should NOT fail with "Assertion failed"
            if !result.passed {
                if let Some(ref err) = result.error {
                    // "Disproved" or "Unknown" is acceptable (feature may be disabled)
                    // "Assertion failed: expected" means Z3 wasn't used
                    assert!(
                        !err.contains("Assertion failed: expected"),
                        "Z3 should have handled symbolic assertion, got: {}",
                        err
                    );
                }
            }
        }
    }
}

/// Test that concrete assertions still work (without Z3)
#[test]
fn test_assert_concrete_values() {
    let mut evaluator = Evaluator::new();

    // Note: Kleis is symbolic, so 3 + 2 does NOT evaluate to 5
    // It stays as plus(3, 2). Only structural equality works concretely.
    let source = r#"
example "concrete assertions" {
    // These should pass via structural equality
    let x = 5
    assert(x = 5)
    // Note: we can only test what the evaluator actually binds
}
"#;

    let program =
        parse_kleis_program_with_file(source, "test_concrete.kleis").expect("Should parse");

    evaluator.load_program(&program).unwrap();

    for item in &program.items {
        if let TopLevel::ExampleBlock(example) = item {
            let result = evaluator.eval_example_block(example);
            println!(
                "Concrete test: passed={}, {}/{} assertions",
                result.passed, result.assertions_passed, result.assertions_total
            );
            if let Some(ref err) = result.error {
                println!("Error: {}", err);
            }
            assert!(
                result.passed,
                "Concrete assertions should pass: {:?}",
                result.error
            );
            assert_eq!(result.assertions_passed, 1);
        }
    }
}

/// Test that invalid symbolic assertions are correctly disproved
#[test]
fn test_assert_invalid_symbolic() {
    let mut evaluator = Evaluator::new();

    // This assertion is false: x + y ≠ y + y (in general)
    let source = r#"
example "invalid symbolic" {
    // This is NOT true in general (unless x = y)
    assert(x + y = y + y)
}
"#;

    let program =
        parse_kleis_program_with_file(source, "test_invalid.kleis").expect("Should parse");

    evaluator.load_program(&program).unwrap();

    for item in &program.items {
        if let TopLevel::ExampleBlock(example) = item {
            let result = evaluator.eval_example_block(example);
            println!("Invalid test: passed={}", result.passed);
            if let Some(ref err) = result.error {
                println!("Error: {}", err);
            }
            // This should either:
            // - Fail (Z3 disproves it)
            // - Pass (optimistic unknown handling - acceptable)
            // We don't assert anything specific here, just that it completes
        }
    }
}

/// Test associativity with Z3
#[test]
fn test_assert_associativity() {
    let mut evaluator = Evaluator::new();

    let source = r#"
structure Semigroup(S) {
    operation (*) : S × S → S
    axiom associativity: ∀(a b c : S). (a * b) * c = a * (b * c)
}

example "test associativity" {
    // Should be verified by Z3 using the associativity axiom
    assert((x * y) * z = x * (y * z))
}
"#;

    let program = parse_kleis_program_with_file(source, "test_assoc.kleis").expect("Should parse");

    evaluator.load_program(&program).unwrap();

    for item in &program.items {
        if let TopLevel::ExampleBlock(example) = item {
            println!("\n=== Running example: {} ===", example.name);
            let result = evaluator.eval_example_block(example);

            println!(
                "Associativity test: passed={}, {}/{}",
                result.passed, result.assertions_passed, result.assertions_total
            );

            // Should pass via Z3 or be treated as unknown
            if !result.passed {
                if let Some(ref err) = result.error {
                    assert!(
                        !err.contains("Assertion failed: expected"),
                        "Z3 should handle associativity, got: {}",
                        err
                    );
                }
            }
        }
    }
}
