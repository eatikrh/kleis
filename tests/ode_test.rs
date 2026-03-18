//! ODE solver integration tests

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::KleisParser;
use kleis::pretty_print::PrettyPrinter;

fn create_evaluator() -> Evaluator {
    Evaluator::new()
}

fn eval(evaluator: &Evaluator, code: &str) -> Result<String, String> {
    let mut parser = KleisParser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let result = evaluator.eval_concrete(&expr)?;
    let pp = PrettyPrinter::new();
    Ok(pp.format_expression(&result))
}

#[test]
fn test_ode45_exponential_decay() {
    let evaluator = create_evaluator();
    // dy/dt = -y, y(0) = 1
    // Lambda: λ t y . [neg(y[0])]
    let result = eval(
        &evaluator,
        "ode45(lambda t y . [neg(index(y, 0))], [1], [0, 1], 0.1)",
    );
    assert!(result.is_ok(), "ODE45 failed: {:?}", result);

    let output = result.unwrap();
    println!("Exponential decay result: {}", output);
    assert!(output.contains("["), "Expected list output");
}

#[test]
fn test_ode45_harmonic_oscillator() {
    let evaluator = create_evaluator();
    // d²x/dt² = -x => [x', v'] = [v, -x]
    // Lambda: λ t y . [y[1], neg(y[0])]
    let result = eval(
        &evaluator,
        "ode45(lambda t y . [index(y, 1), neg(index(y, 0))], [1, 0], [0, 6.28], 0.1)",
    );
    assert!(result.is_ok(), "ODE45 failed: {:?}", result);
    println!("Harmonic oscillator result: {}", result.unwrap());
}

#[test]
fn test_ode45_pendulum() {
    let evaluator = create_evaluator();
    // Pendulum: θ'' = -sin(θ)
    // Lambda: λ t y . [y[1], neg(sin(y[0]))]
    let result = eval(
        &evaluator,
        "ode45(lambda t y . [index(y, 1), neg(sin(index(y, 0)))], [0.1, 0], [0, 5], 0.1)",
    );
    assert!(result.is_ok(), "ODE45 with sin failed: {:?}", result);
    println!("Pendulum result: {}", result.unwrap());
}
