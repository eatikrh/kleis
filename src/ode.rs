//! ODE solver integration using ode_solvers crate.
//!
//! Provides Runge-Kutta integration for Kleis.

use ode_solvers::dopri5::Dopri5;
use ode_solvers::DVector;

/// State vector type
type State = DVector<f64>;

/// ODE system wrapper for a closure
struct OdeSystem<F>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    dynamics: F,
}

impl<F> ode_solvers::System<f64, State> for OdeSystem<F>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    fn system(&self, t: f64, y: &State, dydt: &mut State) {
        let result = (self.dynamics)(t, y.as_slice());
        for (i, &v) in result.iter().enumerate() {
            dydt[i] = v;
        }
    }
}

/// Integrate ODE using Dormand-Prince 5(4) method.
///
/// # Arguments
/// * `f` - Dynamics function f(t, y) -> dy/dt
/// * `y0` - Initial state
/// * `t_span` - (t_start, t_end)
/// * `dt` - Initial step size
///
/// # Returns
/// Vector of (t, y) pairs
pub fn integrate_dopri5<F>(
    f: F,
    y0: &[f64],
    t_span: (f64, f64),
    dt: f64,
) -> Result<Vec<(f64, Vec<f64>)>, String>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let system = OdeSystem { dynamics: f };

    let y0_state: State = State::from_vec(y0.to_vec());

    let mut stepper = Dopri5::new(system, t_span.0, t_span.1, dt, y0_state, 1e-6, 1e-6);

    let res = stepper.integrate();

    match res {
        Ok(_stats) => {
            let t_out = stepper.x_out();
            let y_out = stepper.y_out();

            Ok(t_out
                .iter()
                .zip(y_out.iter())
                .map(|(&t, y): (&f64, &State)| (t, y.as_slice().to_vec()))
                .collect())
        }
        Err(e) => Err(format!("Integration failed: {:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_decay() {
        // dy/dt = -y, y(0) = 1 => y(t) = e^(-t)
        let result = integrate_dopri5(|_t, y| vec![-y[0]], &[1.0], (0.0, 1.0), 0.1).unwrap();

        let (t_final, y_final) = result.last().unwrap();
        let expected = (-1.0_f64).exp();

        assert!(
            (y_final[0] - expected).abs() < 1e-4,
            "Expected {}, got {}",
            expected,
            y_final[0]
        );
    }

    #[test]
    fn test_harmonic_oscillator() {
        // d²x/dt² = -x => [x', v'] = [v, -x]
        let result =
            integrate_dopri5(|_t, y| vec![y[1], -y[0]], &[1.0, 0.0], (0.0, 6.28), 0.1).unwrap();

        // After one period, should return to initial state
        let (_, y_final) = result.last().unwrap();

        assert!(
            (y_final[0] - 1.0).abs() < 0.01,
            "x should be ~1, got {}",
            y_final[0]
        );
    }
}

