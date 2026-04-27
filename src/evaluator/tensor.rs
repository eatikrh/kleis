use crate::ast::Expression;
use crate::numerical;

use super::Evaluator;

impl Evaluator {
    /// ndarray_reshape(flat_list, shape_list) → NDArray(shape, data)
    ///
    /// Takes a flat list of numbers and a shape list of integers,
    /// validates dimensions, and returns an NDArray wrapper.
    pub(crate) fn ndarray_reshape(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 2 {
            return Ok(None);
        }

        let data_exprs = match &args[0] {
            Expression::List(items) => items.clone(),
            _ => return Ok(None),
        };

        let shape_exprs = match &args[1] {
            Expression::List(items) => items.clone(),
            _ => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = data_exprs
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "ndarray_reshape: symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let shape: Result<Vec<usize>, _> = shape_exprs
            .iter()
            .map(|e| {
                self.as_integer(e)
                    .map(|v| v as usize)
                    .ok_or_else(|| "ndarray_reshape: shape must be integers".to_string())
            })
            .collect();
        let shape = shape?;

        numerical::tensor_reshape(&data, &shape).map_err(|e| e.to_string())?;

        Ok(Some(Self::make_ndarray(&shape, &data_exprs)))
    }

    /// ndarray_contract(ndarray_expr, matrix_expr, axis) → NDArray(new_shape, new_data)
    ///
    /// Contracts a matrix along one axis of an NDArray tensor.
    /// This is the core of the factorized transfer matrix trick.
    pub(crate) fn ndarray_contract(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 3 {
            return Ok(None);
        }

        let (shape, data_exprs) = match Self::extract_ndarray(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let (mat_rows, mat_cols, mat_elements) = match self.extract_matrix(&args[1]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let axis = match self.as_integer(&args[2]) {
            Some(v) => v as usize,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = data_exprs
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "ndarray_contract: symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let mat_data: Result<Vec<f64>, _> = mat_elements
            .iter()
            .map(|e| {
                self.as_number(e).ok_or_else(|| {
                    "ndarray_contract: symbolic matrix elements not supported".to_string()
                })
            })
            .collect();
        let mat_data = mat_data?;

        let tensor = numerical::tensor_reshape(&data, &shape).map_err(|e| e.to_string())?;
        let result = numerical::tensor_contract_axis(&tensor, &mat_data, mat_rows, mat_cols, axis)
            .map_err(|e| e.to_string())?;

        let new_shape: Vec<usize> = result.shape().to_vec();
        let new_data: Vec<Expression> = numerical::tensor_flatten(&result)
            .iter()
            .map(|&v| Self::const_from_f64(v))
            .collect();

        Ok(Some(Self::make_ndarray(&new_shape, &new_data)))
    }

    /// ndarray_moveaxis(ndarray_expr, from, to) → NDArray(new_shape, new_data)
    ///
    /// Moves a tensor axis from one position to another.
    pub(crate) fn ndarray_moveaxis(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 3 {
            return Ok(None);
        }

        let (shape, data_exprs) = match Self::extract_ndarray(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let from = match self.as_integer(&args[1]) {
            Some(v) => v as usize,
            None => return Ok(None),
        };

        let to = match self.as_integer(&args[2]) {
            Some(v) => v as usize,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = data_exprs
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "ndarray_moveaxis: symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let tensor = numerical::tensor_reshape(&data, &shape).map_err(|e| e.to_string())?;
        let result = numerical::tensor_moveaxis(&tensor, from, to).map_err(|e| e.to_string())?;

        let new_shape: Vec<usize> = result.shape().to_vec();
        let new_data: Vec<Expression> = numerical::tensor_flatten(&result)
            .iter()
            .map(|&v| Self::const_from_f64(v))
            .collect();

        Ok(Some(Self::make_ndarray(&new_shape, &new_data)))
    }

    /// ndarray_flatten(ndarray_expr) → flat list
    ///
    /// Extracts the flat data from an NDArray wrapper.
    pub(crate) fn ndarray_flatten(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (_shape, data_exprs) = match Self::extract_ndarray(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        Ok(Some(Expression::List(data_exprs)))
    }

    /// Extract shape and data from an NDArray(shape_list, data_list) expression.
    fn extract_ndarray(expr: &Expression) -> Option<(Vec<usize>, Vec<Expression>)> {
        match expr {
            Expression::Operation { name, args, .. } if name == "NDArray" && args.len() == 2 => {
                let shape = match &args[0] {
                    Expression::List(items) => {
                        let mut dims = Vec::with_capacity(items.len());
                        for item in items {
                            if let Expression::Const(s) = item {
                                dims.push(s.parse::<usize>().ok()?);
                            } else {
                                return None;
                            }
                        }
                        dims
                    }
                    _ => return None,
                };

                let data = match &args[1] {
                    Expression::List(items) => items.clone(),
                    _ => return None,
                };

                Some((shape, data))
            }
            _ => None,
        }
    }

    // ========================================
    // DFT / FFT builtins
    // ========================================

    /// dft(list) → list of complex numbers
    ///
    /// Discrete Fourier Transform of a real-valued vector.
    pub(crate) fn builtin_dft(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }
        let data = self.extract_real_list(&args[0])?;
        let result = numerical::dft(&data).map_err(|e| e.to_string())?;
        Ok(Some(Self::complex_vec_to_expr(&result)))
    }

    /// fft(list) → list of complex numbers
    ///
    /// Fast Fourier Transform (Cooley-Tukey radix-2, falls back to DFT
    /// for non-power-of-2 sizes).
    pub(crate) fn builtin_fft(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }
        let data = self.extract_real_list(&args[0])?;
        let result = numerical::fft(&data).map_err(|e| e.to_string())?;
        Ok(Some(Self::complex_vec_to_expr(&result)))
    }

    /// idft(list) → list of complex numbers
    ///
    /// Inverse DFT. Input is a list of complex numbers (or reals).
    pub(crate) fn builtin_idft(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }
        let data = self.extract_complex_list(&args[0])?;
        let result = numerical::idft(&data).map_err(|e| e.to_string())?;
        Ok(Some(Self::complex_vec_to_expr(&result)))
    }

    /// ifft(list) → list of complex numbers
    ///
    /// Inverse FFT. Input is a list of complex numbers (or reals).
    pub(crate) fn builtin_ifft(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }
        let data = self.extract_complex_list(&args[0])?;
        let result = numerical::ifft(&data).map_err(|e| e.to_string())?;
        Ok(Some(Self::complex_vec_to_expr(&result)))
    }

    /// Extract a list of f64 from a Kleis list expression.
    fn extract_real_list(&self, expr: &Expression) -> Result<Vec<f64>, String> {
        let items = match expr {
            Expression::List(items) => items,
            _ => return Err("Expected a list".to_string()),
        };
        items
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported for FFT".to_string())
            })
            .collect()
    }

    /// Extract a list of complex (f64, f64) from a Kleis list expression.
    /// Accepts both real numbers (treated as real + 0i) and complex(re, im).
    fn extract_complex_list(&self, expr: &Expression) -> Result<Vec<(f64, f64)>, String> {
        let items = match expr {
            Expression::List(items) => items,
            _ => return Err("Expected a list".to_string()),
        };
        items
            .iter()
            .map(|e| match e {
                Expression::Operation { name, args, .. }
                    if name == "complex" && args.len() == 2 =>
                {
                    let re = self
                        .as_number(&args[0])
                        .ok_or_else(|| "complex re: not a number".to_string())?;
                    let im = self
                        .as_number(&args[1])
                        .ok_or_else(|| "complex im: not a number".to_string())?;
                    Ok((re, im))
                }
                _ => {
                    let re = self
                        .as_number(e)
                        .ok_or_else(|| "Expected number or complex".to_string())?;
                    Ok((re, 0.0))
                }
            })
            .collect()
    }

    /// Convert a Vec<(f64, f64)> of complex numbers to a Kleis list expression.
    /// Real-only values (|im| < 1e-14) are emitted as plain numbers.
    fn complex_vec_to_expr(values: &[(f64, f64)]) -> Expression {
        Expression::List(
            values
                .iter()
                .map(|&(re, im)| {
                    if im.abs() < 1e-14 {
                        Self::const_from_f64(re)
                    } else {
                        Expression::Operation {
                            name: "complex".to_string(),
                            args: vec![Self::const_from_f64(re), Self::const_from_f64(im)],
                            span: None,
                        }
                    }
                })
                .collect(),
        )
    }

    /// Construct an NDArray(shape_list, data_list) expression.
    fn make_ndarray(shape: &[usize], data: &[Expression]) -> Expression {
        Expression::Operation {
            name: "NDArray".to_string(),
            args: vec![
                Expression::List(
                    shape
                        .iter()
                        .map(|&d| Expression::Const(format!("{}", d)))
                        .collect(),
                ),
                Expression::List(data.to_vec()),
            ],
            span: None,
        }
    }
}
