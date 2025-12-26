// Test that on_eval_start is called with correct file from imported expression's span

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use kleis::ast::Expression;
use kleis::debug::{DebugAction, DebugHook, DebugState, SourceLocation, StackFrame};
use kleis::evaluator::Evaluator;
use kleis::kleis_ast::TopLevel;
use kleis::kleis_parser::parse_kleis_program_with_file;

// Track all on_eval_start calls
struct Tracker {
    calls: Arc<Mutex<Vec<(u32, Option<PathBuf>)>>>, // (line, file)
}

impl DebugHook for Tracker {
    fn on_eval_start(
        &mut self,
        _expr: &Expression,
        loc: &SourceLocation,
        _depth: usize,
    ) -> DebugAction {
        self.calls
            .lock()
            .unwrap()
            .push((loc.line, loc.file.clone()));
        DebugAction::Continue
    }
    fn on_eval_end(&mut self, _: &Expression, _: &Result<Expression, String>, _: usize) {}
    fn on_function_enter(
        &mut self,
        _: &str,
        _: &[Expression],
        _: &SourceLocation,
        _: usize,
    ) {
    }
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
fn test_on_eval_start_reports_imported_file() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let tracker = Tracker {
        calls: calls.clone(),
    };

    let mut evaluator = Evaluator::new();
    evaluator.set_debug_hook(Box::new(tracker));

    // Load helper file - the function body's span will have this file
    let helper_source = r#"
define double(n) = n + n
"#;
    let helper_program =
        parse_kleis_program_with_file(helper_source, "/test/helper.kleis").unwrap();
    evaluator.load_program(&helper_program).unwrap();

    // Load main file
    let main_source = r#"
example "test" {
    let x = double(5)
}
"#;
    let main_program = parse_kleis_program_with_file(main_source, "/test/main.kleis").unwrap();
    
    // Find and run the example block
    for item in &main_program.items {
        if let TopLevel::ExampleBlock(example) = item {
            let _ = evaluator.eval_example_block(example);
        }
    }

    // Check if any call had helper.kleis as the file
    let all_calls = calls.lock().unwrap();
    println!("Total on_eval_start calls: {}", all_calls.len());

    let helper_calls: Vec<_> = all_calls
        .iter()
        .filter(|(_, file)| {
            file.as_ref()
                .map(|f| f.to_string_lossy().contains("helper"))
                .unwrap_or(false)
        })
        .collect();

    println!("Calls with helper.kleis: {}", helper_calls.len());
    for (line, file) in &helper_calls {
        println!("  line={} file={:?}", line, file);
    }

    assert!(
        !helper_calls.is_empty(),
        "on_eval_start should be called with helper.kleis file"
    );
}

