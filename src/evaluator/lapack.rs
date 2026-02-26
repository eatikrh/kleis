use crate::ast::Expression;
use crate::numerical;

use super::Evaluator;

impl Evaluator {
    pub(crate) fn lapack_eigenvalues(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!(
                "eigenvalues requires a square matrix, got {}×{}",
                m, n
            ));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported for LAPACK".to_string())
            })
            .collect();
        let data = data?;

        let eigvals = numerical::eigenvalues(&data, n).map_err(|e| e.to_string())?;

        let result: Vec<Expression> = eigvals
            .iter()
            .map(|(re, im)| {
                if im.abs() < 1e-14 {
                    Expression::Const(format!("{}", re))
                } else {
                    self.make_complex(
                        Expression::Const(format!("{}", re)),
                        Expression::Const(format!("{}", im)),
                    )
                }
            })
            .collect();

        Ok(Some(Expression::List(result)))
    }

    pub(crate) fn lapack_eig(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("eig requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let (eigvals, eigvecs) = numerical::eig(&data, n).map_err(|e| e.to_string())?;

        let vals: Vec<Expression> = eigvals
            .iter()
            .map(|(re, im)| {
                if im.abs() < 1e-14 {
                    Expression::Const(format!("{}", re))
                } else {
                    self.make_complex(
                        Expression::Const(format!("{}", re)),
                        Expression::Const(format!("{}", im)),
                    )
                }
            })
            .collect();

        let vecs: Vec<Expression> = eigvecs
            .iter()
            .map(|v| {
                Expression::List(
                    v.iter()
                        .map(|(re, im)| {
                            if im.abs() < 1e-14 {
                                Expression::Const(format!("{}", re))
                            } else {
                                self.make_complex(
                                    Expression::Const(format!("{}", re)),
                                    Expression::Const(format!("{}", im)),
                                )
                            }
                        })
                        .collect(),
                )
            })
            .collect();

        Ok(Some(Expression::List(vec![
            Expression::List(vals),
            Expression::List(vecs),
        ])))
    }

    pub(crate) fn lapack_svd(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let (u, s, vt) = numerical::svd(&data, m, n).map_err(|e| e.to_string())?;

        let u_expr = self.make_matrix(m, m, u.iter().map(|&v| Self::const_from_f64(v)).collect());
        let s_expr = Expression::List(s.iter().map(|&v| Self::const_from_f64(v)).collect());
        let vt_expr = self.make_matrix(n, n, vt.iter().map(|&v| Self::const_from_f64(v)).collect());

        Ok(Some(Expression::List(vec![u_expr, s_expr, vt_expr])))
    }

    pub(crate) fn lapack_singular_values(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let s = numerical::singular_values(&data, m, n).map_err(|e| e.to_string())?;

        Ok(Some(Expression::List(
            s.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    pub(crate) fn lapack_solve(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 2 {
            return Ok(None);
        }

        let (m, n, a_elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("solve requires a square matrix A, got {}×{}", m, n));
        }

        let b_elements = match &args[1] {
            Expression::List(items) => items.clone(),
            Expression::Operation {
                name,
                args: op_args,
                span: None,
            } if name == "Vector" => {
                if op_args.len() >= 2 {
                    if let Expression::List(items) = &op_args[1] {
                        items.clone()
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            Expression::Operation {
                name,
                args: op_args,
                span: None,
            } if name == "Matrix" || name == "matrix" => {
                if op_args.len() >= 3 {
                    if let Expression::List(items) = &op_args[2] {
                        items.clone()
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        if b_elements.len() != n {
            return Err(format!(
                "solve: b has {} elements but A is {}×{}",
                b_elements.len(),
                m,
                n
            ));
        }

        let a_data: Result<Vec<f64>, _> = a_elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let a_data = a_data?;

        let b_data: Result<Vec<f64>, _> = b_elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let b_data = b_data?;

        let x = numerical::solve(&a_data, &b_data, n).map_err(|e| e.to_string())?;

        Ok(Some(Expression::List(
            x.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    pub(crate) fn lapack_inv(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("inv requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let inv_data = numerical::inv(&data, n).map_err(|e| e.to_string())?;

        Ok(Some(self.make_matrix(
            n,
            n,
            inv_data.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    pub(crate) fn lapack_qr(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let (q, r) = numerical::qr(&data, m, n).map_err(|e| e.to_string())?;

        let k = m.min(n);
        let q_expr = self.make_matrix(m, k, q.iter().map(|&v| Self::const_from_f64(v)).collect());
        let r_expr = self.make_matrix(k, n, r.iter().map(|&v| Self::const_from_f64(v)).collect());

        Ok(Some(Expression::List(vec![q_expr, r_expr])))
    }

    pub(crate) fn lapack_cholesky(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!(
                "cholesky requires a square matrix, got {}×{}",
                m, n
            ));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let l = numerical::cholesky(&data, n).map_err(|e| e.to_string())?;

        Ok(Some(self.make_matrix(
            n,
            n,
            l.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    pub(crate) fn lapack_rank(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let r = numerical::rank(&data, m, n, None).map_err(|e| e.to_string())?;

        Ok(Some(Expression::Const(format!("{}", r))))
    }

    pub(crate) fn lapack_cond(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let c = numerical::cond(&data, m, n).map_err(|e| e.to_string())?;

        if c.is_infinite() {
            Ok(Some(Expression::Object("Inf".to_string())))
        } else {
            Ok(Some(Expression::Const(format!("{}", c))))
        }
    }

    pub(crate) fn lapack_norm(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.is_empty() || args.len() > 2 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let norm_type = if args.len() == 2 {
            match &args[1] {
                Expression::String(s) => s.as_str(),
                Expression::Object(s) => s.as_str(),
                _ => "fro",
            }
        } else {
            "fro"
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let nval = numerical::norm(&data, m, n, norm_type).map_err(|e| e.to_string())?;

        Ok(Some(Expression::Const(format!("{}", nval))))
    }

    pub(crate) fn lapack_det(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("det requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let d = numerical::det(&data, n).map_err(|e| e.to_string())?;

        Ok(Some(Expression::Const(format!("{}", d))))
    }

    pub(crate) fn lapack_schur(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("schur requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported for LAPACK".to_string())
            })
            .collect();
        let data = data?;

        let result = numerical::schur_lapack(&data, n).map_err(|e| e.to_string())?;

        let u_matrix = self.make_matrix(
            n,
            n,
            result
                .u
                .iter()
                .map(|&x| Expression::Const(format!("{}", x)))
                .collect(),
        );

        let t_matrix = self.make_matrix(
            n,
            n,
            result
                .t
                .iter()
                .map(|&x| Expression::Const(format!("{}", x)))
                .collect(),
        );

        let eigenvalues: Vec<Expression> = result
            .wr
            .iter()
            .zip(result.wi.iter())
            .map(|(&re, &im)| {
                if im.abs() < 1e-14 {
                    Expression::Const(format!("{}", re))
                } else {
                    self.make_complex(
                        Expression::Const(format!("{}", re)),
                        Expression::Const(format!("{}", im)),
                    )
                }
            })
            .collect();

        Ok(Some(Expression::List(vec![
            u_matrix,
            t_matrix,
            Expression::List(eigenvalues),
        ])))
    }

    /// Solve the Continuous Algebraic Riccati Equation (CARE):
    ///   A'P + PA - PBR^-1 B'P + Q = 0
    pub(crate) fn lapack_care(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 4 {
            return Err("care(A, B, Q, R) requires 4 matrix arguments".to_string());
        }

        let (na, ma, a_elems) = self
            .extract_matrix(&args[0])
            .ok_or("care: A must be a matrix")?;
        let (nb, mb, b_elems) = self
            .extract_matrix(&args[1])
            .ok_or("care: B must be a matrix")?;
        let (nq, mq, q_elems) = self
            .extract_matrix(&args[2])
            .ok_or("care: Q must be a matrix")?;
        let (nr, mr, r_elems) = self
            .extract_matrix(&args[3])
            .ok_or("care: R must be a matrix")?;

        let n = na;
        let m = mb;
        if na != ma {
            return Err(format!("care: A must be square, got {}×{}", na, ma));
        }
        if nb != n {
            return Err(format!("care: B must have {} rows, got {}", n, nb));
        }
        if nq != n || mq != n {
            return Err(format!("care: Q must be {}×{}, got {}×{}", n, n, nq, mq));
        }
        if nr != m || mr != m {
            return Err(format!("care: R must be {}×{}, got {}×{}", m, m, nr, mr));
        }

        let a: Vec<f64> = a_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("care: A must be numeric"))
            .collect::<Result<_, _>>()?;
        let b: Vec<f64> = b_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("care: B must be numeric"))
            .collect::<Result<_, _>>()?;
        let q: Vec<f64> = q_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("care: Q must be numeric"))
            .collect::<Result<_, _>>()?;
        let r: Vec<f64> = r_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("care: R must be numeric"))
            .collect::<Result<_, _>>()?;

        let p = numerical::care(&a, &b, &q, &r, n, m).map_err(|e| e.to_string())?;

        let p_exprs: Vec<Expression> = p
            .iter()
            .map(|&x| Expression::Const(format!("{}", x)))
            .collect();
        Ok(Some(self.make_matrix(n, n, p_exprs)))
    }

    /// LQR controller design: K = R^-1 B'P where P solves CARE
    pub(crate) fn lapack_lqr(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 4 {
            return Err("lqr(A, B, Q, R) requires 4 matrix arguments".to_string());
        }

        let (na, ma, a_elems) = self
            .extract_matrix(&args[0])
            .ok_or("lqr: A must be a matrix")?;
        let (nb, mb, b_elems) = self
            .extract_matrix(&args[1])
            .ok_or("lqr: B must be a matrix")?;
        let (nq, mq, q_elems) = self
            .extract_matrix(&args[2])
            .ok_or("lqr: Q must be a matrix")?;
        let (nr, mr, r_elems) = self
            .extract_matrix(&args[3])
            .ok_or("lqr: R must be a matrix")?;

        let n = na;
        let m = mb;
        if na != ma {
            return Err(format!("lqr: A must be square, got {}×{}", na, ma));
        }
        if nb != n {
            return Err(format!("lqr: B must have {} rows, got {}", n, nb));
        }
        if nq != n || mq != n {
            return Err(format!("lqr: Q must be {}×{}, got {}×{}", n, n, nq, mq));
        }
        if nr != m || mr != m {
            return Err(format!("lqr: R must be {}×{}, got {}×{}", m, m, nr, mr));
        }

        let a: Vec<f64> = a_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("lqr: A must be numeric"))
            .collect::<Result<_, _>>()?;
        let b: Vec<f64> = b_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("lqr: B must be numeric"))
            .collect::<Result<_, _>>()?;
        let q: Vec<f64> = q_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("lqr: Q must be numeric"))
            .collect::<Result<_, _>>()?;
        let r: Vec<f64> = r_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("lqr: R must be numeric"))
            .collect::<Result<_, _>>()?;

        let p = numerical::care(&a, &b, &q, &r, n, m).map_err(|e| e.to_string())?;
        let k = numerical::lqr_gain(&b, &r, &p, n, m).map_err(|e| e.to_string())?;

        let mut k_rows: Vec<Expression> = Vec::new();
        for i in 0..m {
            let row: Vec<Expression> = (0..n)
                .map(|j| Expression::Const(format!("{}", k[i * n + j])))
                .collect();
            k_rows.push(Expression::List(row));
        }

        let mut p_rows: Vec<Expression> = Vec::new();
        for i in 0..n {
            let row: Vec<Expression> = (0..n)
                .map(|j| Expression::Const(format!("{}", p[i * n + j])))
                .collect();
            p_rows.push(Expression::List(row));
        }

        Ok(Some(Expression::List(vec![
            Expression::List(k_rows),
            Expression::List(p_rows),
        ])))
    }

    /// Solve the Discrete-time Algebraic Riccati Equation (DARE)
    pub(crate) fn lapack_dare(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 4 {
            return Err("dare(A, B, Q, R) requires 4 matrix arguments".to_string());
        }

        let (na, ma, a_elems) = self
            .extract_matrix(&args[0])
            .ok_or("dare: A must be a matrix")?;
        let (nb, mb, b_elems) = self
            .extract_matrix(&args[1])
            .ok_or("dare: B must be a matrix")?;
        let (nq, mq, q_elems) = self
            .extract_matrix(&args[2])
            .ok_or("dare: Q must be a matrix")?;
        let (nr, mr, r_elems) = self
            .extract_matrix(&args[3])
            .ok_or("dare: R must be a matrix")?;

        let n = na;
        let m = mb;
        if na != ma {
            return Err(format!("dare: A must be square, got {}×{}", na, ma));
        }
        if nb != n {
            return Err(format!("dare: B must have {} rows, got {}", n, nb));
        }
        if nq != n || mq != n {
            return Err(format!("dare: Q must be {}×{}, got {}×{}", n, n, nq, mq));
        }
        if nr != m || mr != m {
            return Err(format!("dare: R must be {}×{}, got {}×{}", m, m, nr, mr));
        }

        let a: Vec<f64> = a_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dare: A must be numeric"))
            .collect::<Result<_, _>>()?;
        let b: Vec<f64> = b_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dare: B must be numeric"))
            .collect::<Result<_, _>>()?;
        let q: Vec<f64> = q_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dare: Q must be numeric"))
            .collect::<Result<_, _>>()?;
        let r: Vec<f64> = r_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dare: R must be numeric"))
            .collect::<Result<_, _>>()?;

        let p = numerical::dare(&a, &b, &q, &r, n, m).map_err(|e| e.to_string())?;

        let mut p_rows: Vec<Expression> = Vec::new();
        for i in 0..n {
            let row: Vec<Expression> = (0..n)
                .map(|j| Expression::Const(format!("{}", p[i * n + j])))
                .collect();
            p_rows.push(Expression::List(row));
        }

        Ok(Some(Expression::List(p_rows)))
    }

    /// Discrete-time LQR: K = (B'PB + R)^-1 B'PA where P solves DARE
    pub(crate) fn lapack_dlqr(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 4 {
            return Err("dlqr(A, B, Q, R) requires 4 matrix arguments".to_string());
        }

        let (na, ma, a_elems) = self
            .extract_matrix(&args[0])
            .ok_or("dlqr: A must be a matrix")?;
        let (nb, mb, b_elems) = self
            .extract_matrix(&args[1])
            .ok_or("dlqr: B must be a matrix")?;
        let (nq, mq, q_elems) = self
            .extract_matrix(&args[2])
            .ok_or("dlqr: Q must be a matrix")?;
        let (nr, mr, r_elems) = self
            .extract_matrix(&args[3])
            .ok_or("dlqr: R must be a matrix")?;

        let n = na;
        let m = mb;
        if na != ma {
            return Err(format!("dlqr: A must be square, got {}×{}", na, ma));
        }
        if nb != n {
            return Err(format!("dlqr: B must have {} rows, got {}", n, nb));
        }
        if nq != n || mq != n {
            return Err(format!("dlqr: Q must be {}×{}, got {}×{}", n, n, nq, mq));
        }
        if nr != m || mr != m {
            return Err(format!("dlqr: R must be {}×{}, got {}×{}", m, m, nr, mr));
        }

        let a: Vec<f64> = a_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dlqr: A must be numeric"))
            .collect::<Result<_, _>>()?;
        let b: Vec<f64> = b_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dlqr: B must be numeric"))
            .collect::<Result<_, _>>()?;
        let q: Vec<f64> = q_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dlqr: Q must be numeric"))
            .collect::<Result<_, _>>()?;
        let r: Vec<f64> = r_elems
            .iter()
            .map(|e| self.as_number(e).ok_or("dlqr: R must be numeric"))
            .collect::<Result<_, _>>()?;

        let p = numerical::dare(&a, &b, &q, &r, n, m).map_err(|e| e.to_string())?;
        let k = numerical::dlqr_gain(&a, &b, &r, &p, n, m).map_err(|e| e.to_string())?;

        let mut k_rows: Vec<Expression> = Vec::new();
        for i in 0..m {
            let row: Vec<Expression> = (0..n)
                .map(|j| Expression::Const(format!("{}", k[i * n + j])))
                .collect();
            k_rows.push(Expression::List(row));
        }

        let mut p_rows: Vec<Expression> = Vec::new();
        for i in 0..n {
            let row: Vec<Expression> = (0..n)
                .map(|j| Expression::Const(format!("{}", p[i * n + j])))
                .collect();
            p_rows.push(Expression::List(row));
        }

        Ok(Some(Expression::List(vec![
            Expression::List(k_rows),
            Expression::List(p_rows),
        ])))
    }
}
