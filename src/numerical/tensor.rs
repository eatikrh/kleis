//! Dynamic-rank tensor operations and Fourier transforms.
//!
//! Provides reshape, contract-along-axis, moveaxis, and flatten
//! using ndarray's ArrayD (dynamic-rank arrays). These operations
//! enable the factorized transfer matrix trick from the 3D Ising model:
//! instead of materializing a 2^N × 2^N matrix, reshape the state
//! vector as an N-index tensor and apply N separate 2×2 contractions.
//!
//! Also provides DFT/FFT and their inverses for spectral decomposition
//! of transfer matrices into momentum sectors.

use ndarray::{ArrayD, Axis, IxDyn};
use std::f64::consts::PI;

use super::backend::NumericalError;

/// Reshape flat data into a dynamic-rank tensor.
///
/// Validates that `data.len() == shape.iter().product()`.
pub fn tensor_reshape(data: &[f64], shape: &[usize]) -> Result<ArrayD<f64>, NumericalError> {
    let expected_len: usize = shape.iter().product();
    if data.len() != expected_len {
        return Err(NumericalError::ComputationFailed(format!(
            "tensor_reshape: data length {} does not match shape {:?} (product {})",
            data.len(),
            shape,
            expected_len
        )));
    }
    if shape.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "tensor_reshape: shape must be non-empty".to_string(),
        ));
    }
    ArrayD::from_shape_vec(IxDyn(shape), data.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(format!("tensor_reshape: {}", e)))
}

/// Contract a matrix along one axis of a tensor.
///
/// Given tensor T of shape `[d0, d1, ..., d_{axis}, ..., d_{n-1}]` and
/// matrix M of shape `[mat_rows, mat_cols]` where `mat_cols == d_{axis}`,
/// computes the contraction: for each fiber along `axis`, replace it
/// with M applied to it. The result has `d_{axis}` replaced by `mat_rows`.
///
/// This is the core of the factorized transfer matrix trick:
/// ```text
/// w = w.reshape((2,) * nsites)
/// for k in range(nsites):
///     w = np.tensordot(A, w, axes=([1], [k]))
///     w = np.moveaxis(w, 0, k)
/// ```
pub fn tensor_contract_axis(
    tensor: &ArrayD<f64>,
    matrix: &[f64],
    mat_rows: usize,
    mat_cols: usize,
    axis: usize,
) -> Result<ArrayD<f64>, NumericalError> {
    let ndim = tensor.ndim();
    if axis >= ndim {
        return Err(NumericalError::ComputationFailed(format!(
            "tensor_contract_axis: axis {} out of range for {}-dimensional tensor",
            axis, ndim
        )));
    }
    if matrix.len() != mat_rows * mat_cols {
        return Err(NumericalError::ComputationFailed(format!(
            "tensor_contract_axis: matrix length {} != {} × {}",
            matrix.len(),
            mat_rows,
            mat_cols
        )));
    }
    let axis_dim = tensor.shape()[axis];
    if axis_dim != mat_cols {
        return Err(NumericalError::ComputationFailed(format!(
            "tensor_contract_axis: axis {} has dimension {} but matrix has {} columns",
            axis, axis_dim, mat_cols
        )));
    }

    let mut new_shape: Vec<usize> = tensor.shape().to_vec();
    new_shape[axis] = mat_rows;
    let mut result = ArrayD::zeros(IxDyn(&new_shape));

    // For each lane along the target axis, apply the matrix multiplication.
    // A "lane" is a 1D slice obtained by fixing all indices except `axis`.
    let lanes_in = tensor.lanes(Axis(axis));
    let lanes_out = result.lanes_mut(Axis(axis));

    for (lane_in, mut lane_out) in lanes_in.into_iter().zip(lanes_out.into_iter()) {
        for i in 0..mat_rows {
            let mut sum = 0.0;
            for j in 0..mat_cols {
                sum += matrix[i * mat_cols + j] * lane_in[j];
            }
            lane_out[i] = sum;
        }
    }

    Ok(result)
}

/// Move an axis from position `from` to position `to`.
///
/// Builds a permutation and calls `permuted_axes`.
pub fn tensor_moveaxis(
    tensor: &ArrayD<f64>,
    from: usize,
    to: usize,
) -> Result<ArrayD<f64>, NumericalError> {
    let ndim = tensor.ndim();
    if from >= ndim || to >= ndim {
        return Err(NumericalError::ComputationFailed(format!(
            "tensor_moveaxis: from={} or to={} out of range for {}-dimensional tensor",
            from, to, ndim
        )));
    }
    if from == to {
        return Ok(tensor.clone());
    }

    let mut perm: Vec<usize> = (0..ndim).collect();
    perm.remove(from);
    perm.insert(to, from);

    Ok(tensor.clone().permuted_axes(IxDyn(&perm)))
}

/// Flatten a dynamic-rank tensor to a 1D vector (row-major order).
pub fn tensor_flatten(tensor: &ArrayD<f64>) -> Vec<f64> {
    tensor.iter().copied().collect()
}

// ============================================
// Discrete Fourier Transform / Fast Fourier Transform
// ============================================

/// Discrete Fourier Transform (DFT).
///
/// Input: real-valued vector of length N.
/// Output: complex-valued vector of length N as `Vec<(f64, f64)>` (re, im).
///
/// X[k] = sum_{n=0}^{N-1} x[n] * exp(-2πi·n·k / N)
///
/// O(N²) — correct for any size.
pub fn dft(input: &[f64]) -> Result<Vec<(f64, f64)>, NumericalError> {
    if input.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "dft: input must be non-empty".to_string(),
        ));
    }
    let n = input.len();
    let mut output = Vec::with_capacity(n);
    for k in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;
        for (j, &x) in input.iter().enumerate() {
            let angle = -2.0 * PI * (k as f64) * (j as f64) / (n as f64);
            re += x * angle.cos();
            im += x * angle.sin();
        }
        output.push((re, im));
    }
    Ok(output)
}

/// Complex DFT for complex-valued input.
///
/// Input: complex vector as `&[(f64, f64)]` (re, im).
/// Output: complex vector of same length.
pub fn dft_complex(input: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, NumericalError> {
    if input.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "dft_complex: input must be non-empty".to_string(),
        ));
    }
    let n = input.len();
    let mut output = Vec::with_capacity(n);
    for k in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;
        for (j, &(xr, xi)) in input.iter().enumerate() {
            let angle = -2.0 * PI * (k as f64) * (j as f64) / (n as f64);
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            // (xr + i·xi) * (cos_a + i·sin_a)
            re += xr * cos_a - xi * sin_a;
            im += xr * sin_a + xi * cos_a;
        }
        output.push((re, im));
    }
    Ok(output)
}

/// Fast Fourier Transform (Cooley-Tukey radix-2).
///
/// Input: real-valued vector whose length must be a power of 2.
/// Output: complex-valued vector of same length.
///
/// O(N log N). Falls back to DFT for non-power-of-2 sizes.
pub fn fft(input: &[f64]) -> Result<Vec<(f64, f64)>, NumericalError> {
    if input.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "fft: input must be non-empty".to_string(),
        ));
    }
    let n = input.len();
    if !n.is_power_of_two() {
        return dft(input);
    }
    let complex_input: Vec<(f64, f64)> = input.iter().map(|&x| (x, 0.0)).collect();
    fft_complex(&complex_input)
}

/// Complex FFT (Cooley-Tukey radix-2).
///
/// Input: complex vector whose length must be a power of 2.
/// Falls back to complex DFT for non-power-of-2 sizes.
pub fn fft_complex(input: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, NumericalError> {
    if input.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "fft_complex: input must be non-empty".to_string(),
        ));
    }
    let n = input.len();
    if !n.is_power_of_two() {
        return dft_complex(input);
    }
    if n == 1 {
        return Ok(input.to_vec());
    }

    let even: Vec<(f64, f64)> = input.iter().step_by(2).copied().collect();
    let odd: Vec<(f64, f64)> = input.iter().skip(1).step_by(2).copied().collect();

    let even_fft = fft_complex(&even)?;
    let odd_fft = fft_complex(&odd)?;

    let mut output = vec![(0.0, 0.0); n];
    let half = n / 2;
    for k in 0..half {
        let angle = -2.0 * PI * (k as f64) / (n as f64);
        let twiddle_re = angle.cos();
        let twiddle_im = angle.sin();
        // twiddle * odd[k]
        let (or, oi) = odd_fft[k];
        let tr = twiddle_re * or - twiddle_im * oi;
        let ti = twiddle_re * oi + twiddle_im * or;
        output[k] = (even_fft[k].0 + tr, even_fft[k].1 + ti);
        output[k + half] = (even_fft[k].0 - tr, even_fft[k].1 - ti);
    }
    Ok(output)
}

/// Inverse Discrete Fourier Transform.
///
/// Input: complex vector as `&[(f64, f64)]`.
/// Output: complex vector (for real input, imaginary parts will be ~0).
///
/// x[n] = (1/N) sum_{k=0}^{N-1} X[k] * exp(+2πi·n·k / N)
pub fn idft(input: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, NumericalError> {
    if input.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "idft: input must be non-empty".to_string(),
        ));
    }
    let n = input.len();
    // IDFT = conjugate, DFT, conjugate, divide by N
    let conjugated: Vec<(f64, f64)> = input.iter().map(|&(r, i)| (r, -i)).collect();
    let transformed = dft_complex(&conjugated)?;
    let scale = 1.0 / (n as f64);
    Ok(transformed
        .iter()
        .map(|&(r, i)| (r * scale, -i * scale))
        .collect())
}

/// Inverse FFT (Cooley-Tukey).
///
/// Input: complex vector (power-of-2 length preferred).
/// Output: complex vector.
pub fn ifft(input: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, NumericalError> {
    if input.is_empty() {
        return Err(NumericalError::ComputationFailed(
            "ifft: input must be non-empty".to_string(),
        ));
    }
    let n = input.len();
    let conjugated: Vec<(f64, f64)> = input.iter().map(|&(r, i)| (r, -i)).collect();
    let transformed = fft_complex(&conjugated)?;
    let scale = 1.0 / (n as f64);
    Ok(transformed
        .iter()
        .map(|&(r, i)| (r * scale, -i * scale))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reshape_roundtrip() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let t = tensor_reshape(&data, &[2, 2, 2]).unwrap();
        assert_eq!(t.shape(), &[2, 2, 2]);
        let flat = tensor_flatten(&t);
        assert_eq!(flat, data);
    }

    #[test]
    fn test_reshape_bad_size() {
        let data = vec![1.0, 2.0, 3.0];
        assert!(tensor_reshape(&data, &[2, 2]).is_err());
    }

    #[test]
    fn test_reshape_empty_shape() {
        let data = vec![1.0];
        assert!(tensor_reshape(&data, &[]).is_err());
    }

    #[test]
    fn test_contract_identity_matrix() {
        // Contracting with the identity matrix should leave tensor unchanged
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let t = tensor_reshape(&data, &[2, 2]).unwrap();
        let identity = vec![1.0, 0.0, 0.0, 1.0];

        let result = tensor_contract_axis(&t, &identity, 2, 2, 0).unwrap();
        let flat = tensor_flatten(&result);
        assert_eq!(flat, data);

        let result = tensor_contract_axis(&t, &identity, 2, 2, 1).unwrap();
        let flat = tensor_flatten(&result);
        assert_eq!(flat, data);
    }

    #[test]
    fn test_contract_known_matrix() {
        // [[1, 2], [3, 4]] contracted along axis 0 with [[1, 1], [0, 1]]
        // Axis 0 fibers: [1, 3] and [2, 4]
        // M * [1, 3] = [1+3, 3] = [4, 3]
        // M * [2, 4] = [2+4, 4] = [6, 4]
        // Result: [[4, 6], [3, 4]]
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let t = tensor_reshape(&data, &[2, 2]).unwrap();
        let m = vec![1.0, 1.0, 0.0, 1.0];

        let result = tensor_contract_axis(&t, &m, 2, 2, 0).unwrap();
        let flat = tensor_flatten(&result);
        assert_eq!(flat, vec![4.0, 6.0, 3.0, 4.0]);
    }

    #[test]
    fn test_contract_ising_style() {
        // Simulate the Ising trick for N=1 (single site, dim=2)
        // v = [1, 0] (spin-up state)
        // A = [[2, 0], [0, 0.5]] (asymmetric coupling)
        // After reshape to (2,) tensor, contract with A along axis 0, flatten
        // Result: A * [1, 0] = [2, 0]
        let v = vec![1.0, 0.0];
        let t = tensor_reshape(&v, &[2]).unwrap();
        let a = vec![2.0, 0.0, 0.0, 0.5];

        let result = tensor_contract_axis(&t, &a, 2, 2, 0).unwrap();
        let flat = tensor_flatten(&result);
        assert!((flat[0] - 2.0).abs() < 1e-14);
        assert!((flat[1] - 0.0).abs() < 1e-14);
    }

    #[test]
    fn test_contract_axis_mismatch() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let t = tensor_reshape(&data, &[2, 2]).unwrap();
        let m = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]; // 3x3
        assert!(tensor_contract_axis(&t, &m, 3, 3, 0).is_err());
    }

    #[test]
    fn test_moveaxis_transpose() {
        // Moving axis 0 to 1 is a matrix transpose
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let t = tensor_reshape(&data, &[2, 2]).unwrap();
        let transposed = tensor_moveaxis(&t, 0, 1).unwrap();
        let flat = tensor_flatten(&transposed);
        assert_eq!(flat, vec![1.0, 3.0, 2.0, 4.0]);
    }

    #[test]
    fn test_moveaxis_3d() {
        // Shape [2, 3, 4], move axis 2 to 0 → [4, 2, 3]
        let data: Vec<f64> = (0..24).map(|x| x as f64).collect();
        let t = tensor_reshape(&data, &[2, 3, 4]).unwrap();
        let moved = tensor_moveaxis(&t, 2, 0).unwrap();
        assert_eq!(moved.shape(), &[4, 2, 3]);
    }

    #[test]
    fn test_moveaxis_noop() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let t = tensor_reshape(&data, &[2, 2]).unwrap();
        let same = tensor_moveaxis(&t, 0, 0).unwrap();
        assert_eq!(tensor_flatten(&same), data);
    }

    #[test]
    fn test_moveaxis_out_of_range() {
        let data = vec![1.0, 2.0];
        let t = tensor_reshape(&data, &[2]).unwrap();
        assert!(tensor_moveaxis(&t, 0, 5).is_err());
    }

    #[test]
    fn test_flatten() {
        let data: Vec<f64> = (0..24).map(|x| x as f64).collect();
        let t = tensor_reshape(&data, &[2, 3, 4]).unwrap();
        assert_eq!(tensor_flatten(&t), data);
    }

    #[test]
    fn test_full_ising_loop() {
        // Full apply_transfer simulation for N=1 (nsites=1, dim=2)
        // v = [0.6, 0.4], beta=0.5
        // A = [[1+tanh(0.5), 1-tanh(0.5)], [1-tanh(0.5), 1+tanh(0.5)]]
        let beta: f64 = 0.5;
        let t = beta.tanh();
        let a = vec![1.0 + t, 1.0 - t, 1.0 - t, 1.0 + t];
        let v = vec![0.6, 0.4];

        let tensor = tensor_reshape(&v, &[2]).unwrap();
        let contracted = tensor_contract_axis(&tensor, &a, 2, 2, 0).unwrap();
        let result = tensor_flatten(&contracted);

        // Manual: A * v = [(1+t)*0.6 + (1-t)*0.4, (1-t)*0.6 + (1+t)*0.4]
        let expected_0 = (1.0 + t) * 0.6 + (1.0 - t) * 0.4;
        let expected_1 = (1.0 - t) * 0.6 + (1.0 + t) * 0.4;
        assert!((result[0] - expected_0).abs() < 1e-14);
        assert!((result[1] - expected_1).abs() < 1e-14);
    }

    #[test]
    fn test_multi_axis_ising() {
        // N=2 case: nsites=4, dim=16
        // Reshape 16-vector as (2,2,2,2) tensor, contract with A along each axis
        let beta: f64 = 0.22;
        let t = beta.tanh();
        let a = vec![1.0 + t, 1.0 - t, 1.0 - t, 1.0 + t];

        // Start with uniform state
        let v: Vec<f64> = vec![1.0; 16];
        let mut tensor = tensor_reshape(&v, &[2, 2, 2, 2]).unwrap();

        for k in 0..4 {
            tensor = tensor_contract_axis(&tensor, &a, 2, 2, k).unwrap();
        }

        let result = tensor_flatten(&tensor);
        assert_eq!(result.len(), 16);
        // With uniform input and symmetric A, all outputs should be equal
        let first = result[0];
        for &val in &result[1..] {
            assert!(
                (val - first).abs() < 1e-10,
                "Expected uniform output, got {} vs {}",
                val,
                first
            );
        }
    }

    // ========================================
    // DFT / FFT tests
    // ========================================

    #[test]
    fn test_dft_dc_signal() {
        // Constant signal [1, 1, 1, 1]: DFT should be [4, 0, 0, 0]
        let input = vec![1.0, 1.0, 1.0, 1.0];
        let result = dft(&input).unwrap();
        assert!((result[0].0 - 4.0).abs() < 1e-10);
        assert!(result[0].1.abs() < 1e-10);
        for &(re, im) in &result[1..] {
            assert!(re.abs() < 1e-10, "Expected 0, got re={}", re);
            assert!(im.abs() < 1e-10, "Expected 0, got im={}", im);
        }
    }

    #[test]
    fn test_dft_impulse() {
        // Impulse [1, 0, 0, 0]: DFT should be [1, 1, 1, 1] (all ones)
        let input = vec![1.0, 0.0, 0.0, 0.0];
        let result = dft(&input).unwrap();
        for &(re, im) in &result {
            assert!((re - 1.0).abs() < 1e-10);
            assert!(im.abs() < 1e-10);
        }
    }

    #[test]
    fn test_dft_sine() {
        // [0, 1, 0, -1]: single-frequency sine at k=1
        let input = vec![0.0, 1.0, 0.0, -1.0];
        let result = dft(&input).unwrap();
        // X[0] = 0
        assert!(result[0].0.abs() < 1e-10);
        // X[1] = -2i (imaginary part = -2)
        assert!(result[1].0.abs() < 1e-10);
        assert!((result[1].1 - (-2.0)).abs() < 1e-10);
        // X[2] = 0
        assert!(result[2].0.abs() < 1e-10);
        assert!(result[2].1.abs() < 1e-10);
        // X[3] = 2i
        assert!(result[3].0.abs() < 1e-10);
        assert!((result[3].1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_fft_matches_dft() {
        // FFT and DFT should produce identical results for power-of-2 input
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let dft_result = dft(&input).unwrap();
        let fft_result = fft(&input).unwrap();
        assert_eq!(dft_result.len(), fft_result.len());
        for (d, f) in dft_result.iter().zip(fft_result.iter()) {
            assert!(
                (d.0 - f.0).abs() < 1e-10,
                "Real mismatch: {} vs {}",
                d.0,
                f.0
            );
            assert!(
                (d.1 - f.1).abs() < 1e-10,
                "Imag mismatch: {} vs {}",
                d.1,
                f.1
            );
        }
    }

    #[test]
    fn test_fft_non_power_of_2() {
        // FFT should fall back to DFT for non-power-of-2
        let input = vec![1.0, 2.0, 3.0];
        let result = fft(&input).unwrap();
        let dft_result = dft(&input).unwrap();
        for (f, d) in result.iter().zip(dft_result.iter()) {
            assert!((f.0 - d.0).abs() < 1e-10);
            assert!((f.1 - d.1).abs() < 1e-10);
        }
    }

    #[test]
    fn test_idft_roundtrip() {
        // DFT then IDFT should recover original signal
        let input = vec![3.0, 1.0, 4.0, 1.0];
        let spectrum = dft(&input).unwrap();
        let recovered = idft(&spectrum).unwrap();
        for (i, &(re, im)) in recovered.iter().enumerate() {
            assert!(
                (re - input[i]).abs() < 1e-10,
                "Real mismatch at {}: {} vs {}",
                i,
                re,
                input[i]
            );
            assert!(im.abs() < 1e-10, "Imaginary should be ~0 at {}: {}", i, im);
        }
    }

    #[test]
    fn test_ifft_roundtrip() {
        // FFT then IFFT should recover original signal
        let input = vec![1.0, 0.0, -1.0, 0.0, 2.0, 3.0, -2.0, 1.0];
        let spectrum = fft(&input).unwrap();
        let recovered = ifft(&spectrum).unwrap();
        for (i, &(re, im)) in recovered.iter().enumerate() {
            assert!(
                (re - input[i]).abs() < 1e-10,
                "Real mismatch at {}: {} vs {}",
                i,
                re,
                input[i]
            );
            assert!(im.abs() < 1e-10, "Imaginary should be ~0 at {}: {}", i, im);
        }
    }

    #[test]
    fn test_parsevals_theorem() {
        // Sum of |x[n]|^2 = (1/N) * Sum of |X[k]|^2
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let spectrum = dft(&input).unwrap();
        let time_energy: f64 = input.iter().map(|x| x * x).sum();
        let freq_energy: f64 = spectrum.iter().map(|(r, i)| r * r + i * i).sum();
        let n = input.len() as f64;
        assert!(
            (time_energy - freq_energy / n).abs() < 1e-10,
            "Parseval's theorem: {} vs {}",
            time_energy,
            freq_energy / n
        );
    }

    #[test]
    fn test_dft_empty_error() {
        assert!(dft(&[]).is_err());
    }

    #[test]
    fn test_fft_empty_error() {
        assert!(fft(&[]).is_err());
    }
}
