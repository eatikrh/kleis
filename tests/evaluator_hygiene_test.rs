use kleis::ast::{Expression, MatchCase, Pattern, QuantifiedVar, SourceSpan};
use kleis::evaluator::Evaluator;
use kleis::kleis_ast::FunctionDef;

#[test]
fn test_substitute_respects_quantifier_binder() {
    let mut evaluator = Evaluator::new();

    let body = Expression::Quantifier {
        quantifier: kleis::ast::QuantifierKind::ForAll,
        variables: vec![QuantifiedVar::new("x", Some("M"))],
        where_clause: Some(Box::new(Expression::Object("x".to_string()))),
        body: Box::new(Expression::Object("x".to_string())),
    };

    let func_def = FunctionDef {
        name: "f".to_string(),
        params: vec!["x".to_string()],
        type_annotation: None,
        body,
        span: None,
    };
    evaluator.load_function_def(&func_def).unwrap();

    let result = evaluator
        .apply_function("f", vec![Expression::Object("y".to_string())])
        .unwrap();

    match result {
        Expression::Quantifier {
            variables,
            where_clause,
            body,
            ..
        } => {
            assert_eq!(variables.len(), 1);
            assert_eq!(variables[0].name, "x");
            match where_clause.as_deref() {
                Some(Expression::Object(name)) => assert_eq!(name, "x"),
                other => panic!("Expected where clause Object('x'), got {:?}", other),
            }
            match *body {
                Expression::Object(ref name) => assert_eq!(name, "x"),
                other => panic!("Expected body Object('x'), got {:?}", other),
            }
        }
        other => panic!("Expected Quantifier result, got {:?}", other),
    }
}

#[test]
fn test_substitute_respects_match_binder() {
    let mut evaluator = Evaluator::new();

    let match_expr = Expression::match_expr(
        Expression::Object("z".to_string()),
        vec![MatchCase::new(
            Pattern::variable("x"),
            Expression::Object("x".to_string()),
        )],
    );

    let func_def = FunctionDef {
        name: "f".to_string(),
        params: vec!["x".to_string()],
        type_annotation: None,
        body: match_expr,
        span: None,
    };
    evaluator.load_function_def(&func_def).unwrap();

    let result = evaluator
        .apply_function("f", vec![Expression::Object("y".to_string())])
        .unwrap();

    match result {
        Expression::Match {
            scrutinee, cases, ..
        } => {
            assert!(matches!(*scrutinee, Expression::Object(ref s) if s == "z"));
            assert_eq!(cases.len(), 1);
            match &cases[0].body {
                Expression::Object(name) => assert_eq!(name, "x"),
                other => panic!("Expected body Object('x'), got {:?}", other),
            }
        }
        other => panic!("Expected Match result, got {:?}", other),
    }
}

#[test]
fn test_eval_preserves_builtin_span() {
    let evaluator = Evaluator::new();
    let span = SourceSpan::new(10, 5);

    let expr = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
        ],
        span: Some(span.clone()),
    };

    let result = evaluator.eval(&expr).unwrap();
    match result {
        Expression::Operation {
            span: result_span, ..
        } => {
            assert_eq!(result_span, Some(span));
        }
        other => panic!("Expected Operation result, got {:?}", other),
    }
}

#[test]
fn test_eval_conditional_true_does_not_eval_else_branch() {
    let evaluator = Evaluator::new();

    let error_branch = Expression::Let {
        pattern: Pattern::constant("0"),
        type_annotation: None,
        value: Box::new(Expression::Const("1".to_string())),
        body: Box::new(Expression::Object("unused".to_string())),
        span: None,
    };

    let expr = Expression::conditional(
        Expression::Object("True".to_string()),
        Expression::Const("1".to_string()),
        error_branch,
    );

    let result = evaluator.eval(&expr).unwrap();
    assert_eq!(result, Expression::Const("1".to_string()));
}

#[test]
fn test_eval_conditional_symbolic_keeps_branches() {
    let evaluator = Evaluator::new();

    let error_branch = Expression::Let {
        pattern: Pattern::constant("0"),
        type_annotation: None,
        value: Box::new(Expression::Const("1".to_string())),
        body: Box::new(Expression::Object("unused".to_string())),
        span: None,
    };

    let expr = Expression::conditional(
        Expression::Object("cond".to_string()),
        Expression::Const("1".to_string()),
        error_branch,
    );

    let result = evaluator.eval(&expr).unwrap();
    match result {
        Expression::Conditional {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            assert!(matches!(*condition, Expression::Object(ref s) if s == "cond"));
            assert!(matches!(*then_branch, Expression::Const(ref s) if s == "1"));
            assert!(matches!(*else_branch, Expression::Let { .. }));
        }
        other => panic!("Expected Conditional result, got {:?}", other),
    }
}
