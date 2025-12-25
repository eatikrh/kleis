//! Debug hooks for step-through debugging
//!
//! This module provides the infrastructure for debugging Kleis programs.
//! It uses a callback-based approach where the evaluator calls hook methods
//! at key points during evaluation.

use crate::ast::Expression;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

/// Source location information
#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    /// File path (if known)
    pub file: Option<PathBuf>,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self {
            file: None,
            line,
            column,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    /// Create from a SourceSpan (from the AST)
    pub fn from_span(span: &crate::ast::SourceSpan) -> Self {
        Self {
            file: None,
            line: span.line,
            column: span.column,
        }
    }

    /// Create from a SourceSpan with file path
    pub fn from_span_with_file(span: &crate::ast::SourceSpan, file: PathBuf) -> Self {
        Self {
            file: Some(file),
            line: span.line,
            column: span.column,
        }
    }
}

/// A frame in the call stack
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name (or "<top-level>" for REPL)
    pub name: String,
    /// Source location
    pub location: SourceLocation,
    /// Local bindings in this frame
    pub bindings: HashMap<String, String>,
}

impl StackFrame {
    pub fn new(name: &str, location: SourceLocation) -> Self {
        Self {
            name: name.to_string(),
            location,
            bindings: HashMap::new(),
        }
    }

    pub fn top_level() -> Self {
        Self::new("<top-level>", SourceLocation::default())
    }
}

/// Actions the debugger can take after a hook is called
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    /// Continue normal execution
    Continue,
    /// Step into the next expression
    StepInto,
    /// Step over (continue until we return to this depth)
    StepOver,
    /// Step out (continue until we return to parent)
    StepOut,
}

/// Current debug state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugState {
    /// Not debugging, run at full speed
    Running,
    /// Paused, waiting for user input
    Paused,
    /// Stepping into next expression
    Stepping,
    /// Stepping over (with target depth)
    SteppingOver { target_depth: usize },
    /// Stepping out (with target depth)
    SteppingOut { target_depth: usize },
}

/// Breakpoint definition
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// File path
    pub file: PathBuf,
    /// Line number (1-based)
    pub line: u32,
    /// Whether this breakpoint is active
    pub enabled: bool,
    /// Optional condition expression
    pub condition: Option<String>,
}

impl Breakpoint {
    pub fn new(file: PathBuf, line: u32) -> Self {
        Self {
            file,
            line,
            enabled: true,
            condition: None,
        }
    }
}

/// Trait for debug hooks
///
/// Implement this trait to receive callbacks during evaluation.
/// The debug adapter implements this to pause execution and inspect state.
pub trait DebugHook {
    /// Called before evaluating an expression
    ///
    /// Returns the action to take (continue, step, etc.)
    fn on_eval_start(
        &mut self,
        expr: &Expression,
        location: &SourceLocation,
        depth: usize,
    ) -> DebugAction;

    /// Called after evaluating an expression
    fn on_eval_end(&mut self, expr: &Expression, result: &Result<Expression, String>, depth: usize);

    /// Called when entering a function
    fn on_function_enter(&mut self, name: &str, args: &[Expression], depth: usize);

    /// Called when exiting a function
    fn on_function_exit(&mut self, name: &str, result: &Result<Expression, String>, depth: usize);

    /// Called when a variable is bound
    fn on_bind(&mut self, name: &str, value: &Expression, depth: usize);

    /// Get the current debug state
    fn state(&self) -> &DebugState;

    /// Check if we should stop at the given location
    fn should_stop(&self, location: &SourceLocation, depth: usize) -> bool;

    /// Wait for the user to issue a continue/step command
    /// Returns the action to take
    fn wait_for_command(&mut self) -> DebugAction;

    /// Get the current call stack
    fn get_stack(&self) -> &[StackFrame];

    /// Push a new frame onto the call stack
    fn push_frame(&mut self, frame: StackFrame);

    /// Pop a frame from the call stack
    fn pop_frame(&mut self) -> Option<StackFrame>;
}

/// A no-op debug hook for when debugging is disabled
///
/// This implementation does nothing and always returns Continue,
/// so it has minimal performance impact.
pub struct NoOpDebugHook;

impl DebugHook for NoOpDebugHook {
    fn on_eval_start(
        &mut self,
        _expr: &Expression,
        _location: &SourceLocation,
        _depth: usize,
    ) -> DebugAction {
        DebugAction::Continue
    }

    fn on_eval_end(
        &mut self,
        _expr: &Expression,
        _result: &Result<Expression, String>,
        _depth: usize,
    ) {
    }

    fn on_function_enter(&mut self, _name: &str, _args: &[Expression], _depth: usize) {}

    fn on_function_exit(
        &mut self,
        _name: &str,
        _result: &Result<Expression, String>,
        _depth: usize,
    ) {
    }

    fn on_bind(&mut self, _name: &str, _value: &Expression, _depth: usize) {}

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

    fn push_frame(&mut self, _frame: StackFrame) {}

    fn pop_frame(&mut self) -> Option<StackFrame> {
        None
    }
}

/// A debug hook that actually tracks state and handles breakpoints
pub struct InteractiveDebugHook {
    /// Current state
    state: DebugState,
    /// Call stack
    stack: Vec<StackFrame>,
    /// Breakpoints
    breakpoints: Vec<Breakpoint>,
    /// Channel to receive commands from the debug adapter
    /// (For now, we'll use a simpler callback mechanism)
    command_callback: Option<Box<dyn FnMut() -> DebugAction + Send>>,
    /// Current depth for step over/out
    current_depth: usize,
}

impl InteractiveDebugHook {
    pub fn new() -> Self {
        Self {
            state: DebugState::Paused, // Start paused at entry
            stack: vec![StackFrame::top_level()],
            breakpoints: Vec::new(),
            command_callback: None,
            current_depth: 0,
        }
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, bp: Breakpoint) {
        self.breakpoints.push(bp);
    }

    /// Remove all breakpoints for a file
    pub fn clear_breakpoints(&mut self, file: &PathBuf) {
        self.breakpoints.retain(|bp| &bp.file != file);
    }

    /// Set the callback for getting commands
    pub fn set_command_callback<F>(&mut self, callback: F)
    where
        F: FnMut() -> DebugAction + Send + 'static,
    {
        self.command_callback = Some(Box::new(callback));
    }

    /// Check if a location matches a breakpoint
    fn matches_breakpoint(&self, location: &SourceLocation) -> bool {
        if let Some(ref file) = location.file {
            self.breakpoints
                .iter()
                .any(|bp| bp.enabled && &bp.file == file && bp.line == location.line)
        } else {
            false
        }
    }

    /// Resume execution with a new state
    pub fn resume(&mut self, action: DebugAction) {
        match action {
            DebugAction::Continue => {
                self.state = DebugState::Running;
            }
            DebugAction::StepInto => {
                self.state = DebugState::Stepping;
            }
            DebugAction::StepOver => {
                self.state = DebugState::SteppingOver {
                    target_depth: self.current_depth,
                };
            }
            DebugAction::StepOut => {
                self.state = DebugState::SteppingOut {
                    target_depth: self.current_depth.saturating_sub(1),
                };
            }
        }
    }
}

impl Default for InteractiveDebugHook {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugHook for InteractiveDebugHook {
    fn on_eval_start(
        &mut self,
        _expr: &Expression,
        location: &SourceLocation,
        depth: usize,
    ) -> DebugAction {
        self.current_depth = depth;

        // Check if we should stop
        if self.should_stop(location, depth) {
            self.state = DebugState::Paused;
            return self.wait_for_command();
        }

        DebugAction::Continue
    }

    fn on_eval_end(
        &mut self,
        _expr: &Expression,
        _result: &Result<Expression, String>,
        _depth: usize,
    ) {
        // Could be used for step over logic
    }

    fn on_function_enter(&mut self, name: &str, _args: &[Expression], depth: usize) {
        self.push_frame(StackFrame::new(name, SourceLocation::default()));
        self.current_depth = depth;
    }

    fn on_function_exit(
        &mut self,
        _name: &str,
        _result: &Result<Expression, String>,
        depth: usize,
    ) {
        self.pop_frame();
        self.current_depth = depth;
    }

    fn on_bind(&mut self, name: &str, value: &Expression, _depth: usize) {
        if let Some(frame) = self.stack.last_mut() {
            frame
                .bindings
                .insert(name.to_string(), format!("{:?}", value));
        }
    }

    fn state(&self) -> &DebugState {
        &self.state
    }

    fn should_stop(&self, location: &SourceLocation, depth: usize) -> bool {
        match &self.state {
            DebugState::Paused => true,
            DebugState::Running => self.matches_breakpoint(location),
            DebugState::Stepping => true,
            DebugState::SteppingOver { target_depth } => depth <= *target_depth,
            DebugState::SteppingOut { target_depth } => depth <= *target_depth,
        }
    }

    fn wait_for_command(&mut self) -> DebugAction {
        if let Some(ref mut callback) = self.command_callback {
            let action = callback();
            self.resume(action);
            action
        } else {
            // No callback set, just continue
            DebugAction::Continue
        }
    }

    fn get_stack(&self) -> &[StackFrame] {
        &self.stack
    }

    fn push_frame(&mut self, frame: StackFrame) {
        self.stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<StackFrame> {
        // Keep at least the top-level frame
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }
}

// ============================================================================
// DAP-Compatible Debug Hook (uses channels for blocking communication)
// ============================================================================

/// Event sent from evaluator to DAP server when execution stops
#[derive(Debug, Clone)]
pub struct StopEvent {
    pub reason: StopReason,
    pub location: SourceLocation,
    pub stack: Vec<StackFrame>,
}

/// Why execution stopped
#[derive(Debug, Clone)]
pub enum StopReason {
    Entry,
    Step,
    Breakpoint,
    Pause,
}

/// A debug hook designed for DAP integration
///
/// Uses channels for thread-safe communication:
/// - `command_rx`: Receives commands from DAP (Continue, StepOver, etc.)
/// - `event_tx`: Sends stop events to DAP
pub struct DapDebugHook {
    state: DebugState,
    stack: Vec<StackFrame>,
    breakpoints: Vec<Breakpoint>,
    current_depth: usize,
    current_file: Option<PathBuf>,
    /// Receive commands from DAP server
    command_rx: Receiver<DebugAction>,
    /// Send stop events to DAP server  
    event_tx: Sender<StopEvent>,
}

/// Handle for DAP server to control the debug hook
pub struct DapDebugController {
    /// Send commands to the evaluator
    pub command_tx: Sender<DebugAction>,
    /// Receive stop events from the evaluator
    pub event_rx: Receiver<StopEvent>,
}

impl DapDebugHook {
    /// Create a new DAP debug hook with its controller
    pub fn new() -> (Self, DapDebugController) {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        let hook = Self {
            state: DebugState::Paused, // Start paused
            stack: vec![StackFrame::top_level()],
            breakpoints: Vec::new(),
            current_depth: 0,
            current_file: None,
            command_rx,
            event_tx,
        };

        let controller = DapDebugController {
            command_tx,
            event_rx,
        };

        (hook, controller)
    }

    /// Set the current file being debugged
    pub fn set_file(&mut self, file: PathBuf) {
        self.current_file = Some(file.clone());
        if let Some(frame) = self.stack.first_mut() {
            frame.location.file = Some(file);
        }
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, bp: Breakpoint) {
        self.breakpoints.push(bp);
    }

    /// Clear breakpoints for a file
    pub fn clear_breakpoints(&mut self, file: &PathBuf) {
        self.breakpoints.retain(|bp| &bp.file != file);
    }

    /// Check if location matches a breakpoint
    fn matches_breakpoint(&self, location: &SourceLocation) -> bool {
        if let Some(ref file) = location.file {
            self.breakpoints
                .iter()
                .any(|bp| bp.enabled && &bp.file == file && bp.line == location.line)
        } else {
            false
        }
    }

    /// Send stop event to DAP and wait for command
    fn stop_and_wait(&mut self, reason: StopReason, location: &SourceLocation) -> DebugAction {
        // Update current location in top frame
        if let Some(frame) = self.stack.first_mut() {
            frame.location = location.clone();
        }

        // Send stop event
        let event = StopEvent {
            reason,
            location: location.clone(),
            stack: self.stack.clone(),
        };
        let _ = self.event_tx.send(event);

        // Block waiting for command
        match self.command_rx.recv() {
            Ok(action) => {
                // Update state based on action
                match action {
                    DebugAction::Continue => self.state = DebugState::Running,
                    DebugAction::StepInto => self.state = DebugState::Stepping,
                    DebugAction::StepOver => {
                        self.state = DebugState::SteppingOver {
                            target_depth: self.current_depth,
                        }
                    }
                    DebugAction::StepOut => {
                        self.state = DebugState::SteppingOut {
                            target_depth: self.current_depth.saturating_sub(1),
                        }
                    }
                }
                action
            }
            Err(_) => {
                // Channel closed, just continue
                DebugAction::Continue
            }
        }
    }
}

impl DebugHook for DapDebugHook {
    fn on_eval_start(
        &mut self,
        _expr: &Expression,
        location: &SourceLocation,
        depth: usize,
    ) -> DebugAction {
        self.current_depth = depth;

        // Determine if we should stop
        let should_stop = match &self.state {
            DebugState::Paused => true,
            DebugState::Running => self.matches_breakpoint(location),
            DebugState::Stepping => true,
            DebugState::SteppingOver { target_depth } => depth <= *target_depth,
            DebugState::SteppingOut { target_depth } => depth <= *target_depth,
        };

        if should_stop {
            let reason = if self.matches_breakpoint(location) {
                StopReason::Breakpoint
            } else {
                StopReason::Step
            };
            self.state = DebugState::Paused;
            return self.stop_and_wait(reason, location);
        }

        DebugAction::Continue
    }

    fn on_eval_end(
        &mut self,
        _expr: &Expression,
        _result: &Result<Expression, String>,
        _depth: usize,
    ) {
    }

    fn on_function_enter(&mut self, name: &str, _args: &[Expression], depth: usize) {
        let mut frame = StackFrame::new(name, SourceLocation::default());
        if let Some(ref file) = self.current_file {
            frame.location.file = Some(file.clone());
        }
        self.push_frame(frame);
        self.current_depth = depth;
    }

    fn on_function_exit(
        &mut self,
        _name: &str,
        _result: &Result<Expression, String>,
        depth: usize,
    ) {
        self.pop_frame();
        self.current_depth = depth;
    }

    fn on_bind(&mut self, name: &str, value: &Expression, _depth: usize) {
        if let Some(frame) = self.stack.last_mut() {
            frame
                .bindings
                .insert(name.to_string(), format!("{:?}", value));
        }
    }

    fn state(&self) -> &DebugState {
        &self.state
    }

    fn should_stop(&self, location: &SourceLocation, depth: usize) -> bool {
        match &self.state {
            DebugState::Paused => true,
            DebugState::Running => self.matches_breakpoint(location),
            DebugState::Stepping => true,
            DebugState::SteppingOver { target_depth } => depth <= *target_depth,
            DebugState::SteppingOut { target_depth } => depth <= *target_depth,
        }
    }

    fn wait_for_command(&mut self) -> DebugAction {
        match self.command_rx.recv() {
            Ok(action) => action,
            Err(_) => DebugAction::Continue,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_hook() {
        let mut hook = NoOpDebugHook;
        let expr = Expression::Const("42".to_string());
        let loc = SourceLocation::new(1, 1);

        assert_eq!(hook.on_eval_start(&expr, &loc, 0), DebugAction::Continue);
        assert!(!hook.should_stop(&loc, 0));
    }

    #[test]
    fn test_interactive_hook_breakpoint() {
        let mut hook = InteractiveDebugHook::new();

        // Add a breakpoint
        hook.add_breakpoint(Breakpoint::new(PathBuf::from("test.kleis"), 5));

        // Check breakpoint matching
        let loc_no_match = SourceLocation::new(3, 1).with_file(PathBuf::from("test.kleis"));
        let loc_match = SourceLocation::new(5, 1).with_file(PathBuf::from("test.kleis"));

        hook.state = DebugState::Running;
        assert!(!hook.should_stop(&loc_no_match, 0));
        assert!(hook.should_stop(&loc_match, 0));
    }

    #[test]
    fn test_stack_frames() {
        let mut hook = InteractiveDebugHook::new();

        assert_eq!(hook.get_stack().len(), 1); // Top-level frame

        hook.push_frame(StackFrame::new("fib", SourceLocation::new(10, 1)));
        assert_eq!(hook.get_stack().len(), 2);

        hook.pop_frame();
        assert_eq!(hook.get_stack().len(), 1);

        // Can't pop the top-level frame
        hook.pop_frame();
        assert_eq!(hook.get_stack().len(), 1);
    }
}
