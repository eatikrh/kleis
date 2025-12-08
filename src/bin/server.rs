use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

// For now, we'll work with LaTeX strings directly
// Later we'll add proper Expression parsing

#[derive(Debug, Serialize, Deserialize)]
struct RenderRequest {
    latex: String,
    format: Option<String>, // "latex", "unicode", "svg"
}

#[derive(Debug, Serialize)]
struct RenderResponse {
    output: String,
    format: String,
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct OperationInfo {
    name: String,
    description: String,
    example_latex: String,
}

#[derive(Debug, Serialize)]
struct GalleryResponse {
    examples: Vec<GalleryExample>,
}

#[derive(Debug, Serialize)]
struct GalleryExample {
    title: String,
    latex: String,
}

#[derive(Debug, Deserialize)]
struct ParseRequest {
    latex: String,
}

#[derive(Debug, Serialize)]
struct ParseResponse {
    ast: serde_json::Value,
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenderASTRequest {
    ast: serde_json::Value,
    format: Option<String>, // "latex", "unicode", "html"
}

#[derive(Debug, Serialize)]
struct RenderASTResponse {
    output: String,
    format: String,
    success: bool,
    error: Option<String>,
}

// Type check request/response
#[derive(Debug, Deserialize)]
struct TypeCheckRequest {
    ast: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct TypeCheckResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

// Shared application state
#[derive(Clone)]
struct AppState {
    // TypeChecker loaded from stdlib/matrices.kleis
    type_checker: Arc<std::sync::Mutex<Option<kleis::type_checker::TypeChecker>>>,
}

#[tokio::main]
async fn main() {
    // Initialize TypeChecker with stdlib (includes minimal_prelude + matrices)
    let type_checker = match kleis::type_checker::TypeChecker::with_stdlib() {
        Ok(checker) => {
            println!("✅ TypeChecker initialized with stdlib (minimal_prelude + matrices)");
            Some(checker)
        }
        Err(e) => {
            eprintln!("⚠️  Failed to initialize TypeChecker with stdlib: {}", e);
            None
        }
    };

    let state = Arc::new(AppState {
        type_checker: Arc::new(std::sync::Mutex::new(type_checker)),
    });

    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/render", post(render_handler))
        .route("/api/render_ast", post(render_ast_handler))
        .route("/api/render_typst", post(render_typst_handler))
        .route("/api/parse", post(parse_handler))
        .route("/api/type_check", post(type_check_handler))
        .route("/api/operations", get(operations_handler))
        .route("/api/gallery", get(gallery_handler))
        .route("/health", get(health_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("🚀 Kleis Server starting...");
    println!("📡 Server running at: http://localhost:3000");
    println!("📚 Gallery available at: http://localhost:3000/api/gallery");
    println!("🧪 Health check: http://localhost:3000/health");
    println!();
    println!("Press Ctrl+C to stop");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

// Handler for root path - serves a simple web UI
async fn index_handler() -> impl IntoResponse {
    // Serve dynamically so changes are picked up without recompiling
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => (StatusCode::OK, Html(content)),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Html("<h1>index.html not found. Make sure static/ directory exists.</h1>".to_string()),
        ),
    }
}

// Handler for rendering equations
async fn render_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RenderRequest>,
) -> impl IntoResponse {
    // Parse LaTeX → Expression → Render
    match kleis::parser::parse_latex(&req.latex) {
        Ok(expr) => {
            let format = req.format.as_deref().unwrap_or("latex");
            let target = match format {
                "unicode" => kleis::render::RenderTarget::Unicode,
                "html" => kleis::render::RenderTarget::HTML,
                _ => kleis::render::RenderTarget::LaTeX,
            };

            let ctx = kleis::render::build_default_context();
            let output = kleis::render::render_expression(&expr, &ctx, &target);

            let response = RenderResponse {
                output,
                format: format.to_string(),
                success: true,
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let response = RenderResponse {
                output: String::new(),
                format: req.format.unwrap_or_else(|| "latex".to_string()),
                success: false,
                error: Some(format!("Parse error: {:?}", e)),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

// Handler for parsing LaTeX into AST
async fn parse_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ParseRequest>,
) -> impl IntoResponse {
    match kleis::parser::parse_latex(&req.latex) {
        Ok(expr) => {
            // Convert Expression to JSON-serializable format
            let ast_json = expression_to_json(&expr);
            let response = ParseResponse {
                ast: ast_json,
                success: true,
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let response = ParseResponse {
                ast: serde_json::Value::Null,
                success: false,
                error: Some(format!("Parse error: {:?}", e)),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

// Convert Expression to JSON (simplified serialization)
fn expression_to_json(expr: &kleis::ast::Expression) -> serde_json::Value {
    use kleis::ast::Expression;
    use serde_json::json;

    match expr {
        Expression::Const(s) => json!({"Const": s}),
        Expression::Object(s) => json!({"Object": s}),
        Expression::Placeholder { id, hint } => json!({"Placeholder": {"id": id, "hint": hint}}),
        Expression::Operation { name, args } => {
            let args_json: Vec<serde_json::Value> = args.iter().map(expression_to_json).collect();
            json!({"Operation": {"name": name, "args": args_json}})
        }
        Expression::Match { .. } => {
            // TODO: Implement match expression JSON serialization
            json!({"Match": "not yet implemented"})
        }
    }
}

// Convert JSON back to Expression
fn json_to_expression(json: &serde_json::Value) -> Result<kleis::ast::Expression, String> {
    use kleis::ast::Expression;

    if let Some(obj) = json.as_object() {
        if let Some(const_val) = obj.get("Const") {
            if let Some(s) = const_val.as_str() {
                return Ok(Expression::Const(s.to_string()));
            }
        } else if let Some(obj_val) = obj.get("Object") {
            if let Some(s) = obj_val.as_str() {
                return Ok(Expression::Object(s.to_string()));
            }
        } else if let Some(placeholder_val) = obj.get("Placeholder") {
            if let Some(placeholder_obj) = placeholder_val.as_object() {
                let id = placeholder_obj
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing or invalid placeholder id")? as usize;
                let hint = placeholder_obj
                    .get("hint")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid placeholder hint")?
                    .to_string();
                return Ok(Expression::Placeholder { id, hint });
            }
        } else if let Some(op_val) = obj.get("Operation") {
            if let Some(op_obj) = op_val.as_object() {
                let name = op_obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid operation name")?
                    .to_string();
                let args_json = op_obj
                    .get("args")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing or invalid operation args")?;
                let args: Result<Vec<Expression>, String> =
                    args_json.iter().map(json_to_expression).collect();
                return Ok(Expression::Operation { name, args: args? });
            }
        }
    }

    Err(format!("Invalid expression JSON: {:?}", json))
}

// Handler for rendering from AST
async fn render_ast_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RenderASTRequest>,
) -> impl IntoResponse {
    match json_to_expression(&req.ast) {
        Ok(expr) => {
            let format = req.format.as_deref().unwrap_or("html");
            let target = match format {
                "unicode" => kleis::render::RenderTarget::Unicode,
                "latex" => kleis::render::RenderTarget::LaTeX,
                _ => kleis::render::RenderTarget::HTML,
            };

            let ctx = kleis::render::build_default_context();
            let output = kleis::render::render_expression(&expr, &ctx, &target);

            let response = RenderASTResponse {
                output,
                format: format.to_string(),
                success: true,
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let response = RenderASTResponse {
                output: String::new(),
                format: req.format.unwrap_or_else(|| "html".to_string()),
                success: false,
                error: Some(format!("Invalid AST: {}", e)),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

// Handler for listing available operations
async fn operations_handler() -> impl IntoResponse {
    let operations = vec![
        OperationInfo {
            name: "Fraction".to_string(),
            description: "Creates a fraction a/b".to_string(),
            example_latex: "\\frac{a}{b}".to_string(),
        },
        OperationInfo {
            name: "Square Root".to_string(),
            description: "Square root of x".to_string(),
            example_latex: "\\sqrt{x}".to_string(),
        },
        OperationInfo {
            name: "Integral".to_string(),
            description: "Definite integral".to_string(),
            example_latex: "\\int_{a}^{b} f(x) \\, dx".to_string(),
        },
        OperationInfo {
            name: "Sum".to_string(),
            description: "Summation with bounds".to_string(),
            example_latex: "\\sum_{i=1}^{n} i".to_string(),
        },
        OperationInfo {
            name: "Matrix".to_string(),
            description: "2x2 matrix".to_string(),
            example_latex: "\\begin{bmatrix}a&b\\\\c&d\\end{bmatrix}".to_string(),
        },
    ];

    (StatusCode::OK, Json(operations))
}

// Handler for gallery examples
async fn gallery_handler() -> impl IntoResponse {
    let samples = kleis::render::collect_samples_for_gallery();

    let examples: Vec<GalleryExample> = samples
        .into_iter()
        .map(|(title, latex)| GalleryExample { title, latex })
        .collect();

    (StatusCode::OK, Json(GalleryResponse { examples }))
}

// Handler for rendering with Typst (returns SVG with placeholder positions)
async fn render_typst_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RenderASTRequest>,
) -> impl IntoResponse {
    eprintln!("=== render_typst_handler called ===");
    eprintln!("Received AST JSON: {:?}", req.ast);

    match json_to_expression(&req.ast) {
        Ok(expr) => {
            eprintln!("Parsed Expression: {:#?}", expr);

            // Collect ALL argument slots with their info (empty or filled)
            let arg_slots = collect_argument_slots(&expr);
            eprintln!("Argument slots: {} total", arg_slots.len());

            for (i, slot) in arg_slots.iter().enumerate() {
                eprintln!(
                    "  Slot {}: id={}, is_placeholder={}, hint='{}'",
                    i, slot.id, slot.is_placeholder, slot.hint
                );
            }

            // Get unfilled placeholder IDs for Typst square rendering
            // Parse "ph{number}" format back to usize
            let unfilled_ids: Vec<usize> = arg_slots
                .iter()
                .filter(|s| s.is_placeholder)
                .filter_map(|s| {
                    if s.id.starts_with("ph") {
                        s.id[2..].parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .collect();

            // For all_slot_ids, we only care about placeholders (which have numeric IDs)
            // Filled slots have UUIDs which don't map to Typst placeholder positions
            let all_slot_ids = unfilled_ids.clone();

            // Build node_id -> UUID map from argument slots
            // Truncate UUIDs with collision detection and regeneration
            let mut node_id_to_uuid = std::collections::HashMap::new();
            let mut used_truncated = std::collections::HashSet::new();

            for slot in &arg_slots {
                let node_id = slot
                    .path
                    .iter()
                    .enumerate()
                    .map(|(depth, &idx)| {
                        if depth == 0 {
                            format!("{}", idx)
                        } else {
                            format!(".{}", idx)
                        }
                    })
                    .collect::<String>();
                let node_id = if node_id.is_empty() {
                    "0".to_string()
                } else {
                    format!("0.{}", node_id)
                };

                // Only process filled slots (not placeholders)
                if !slot.is_placeholder {
                    // Truncate UUID to first 8 chars
                    let mut truncated = slot.id.chars().take(8).collect::<String>();

                    // Check for collision - regenerate if needed
                    let mut attempts = 0;
                    while used_truncated.contains(&truncated) && attempts < 100 {
                        eprintln!(
                            "⚠️  UUID collision detected for {}, regenerating...",
                            truncated
                        );
                        let new_uuid = uuid::Uuid::new_v4().to_string().replace("-", "");
                        truncated = new_uuid.chars().take(8).collect::<String>();
                        attempts += 1;
                    }

                    if attempts >= 100 {
                        eprintln!(
                            "❌ Failed to generate unique 8-char UUID after 100 attempts, using full UUID"
                        );
                        truncated = uuid::Uuid::new_v4().to_string().replace("-", "");
                    }

                    used_truncated.insert(truncated.clone());
                    node_id_to_uuid.insert(node_id, truncated);
                }
            }

            eprintln!(
                "Built node_id->UUID map with {} entries",
                node_id_to_uuid.len()
            );

            // Compile with Typst using semantic bounding box extraction
            // Pass both unfilled_ids (for placeholder squares) and all_slot_ids (for filled content)
            match kleis::math_layout::compile_with_semantic_boxes_and_slots(
                &expr,
                &unfilled_ids,
                &all_slot_ids,
                &node_id_to_uuid,
            ) {
                Ok(output) => {
                    let response = serde_json::json!({
                        "svg": output.svg,
                        "placeholders": output.placeholder_positions.iter().map(|p| {
                            serde_json::json!({
                                "id": p.id,
                                "x": p.x,
                                "y": p.y,
                                "width": p.width,
                                "height": p.height,
                            })
                        }).collect::<Vec<_>>(),
                        "argument_bounding_boxes": output.argument_bounding_boxes.iter().map(|b| {
                            serde_json::json!({
                                "arg_index": b.arg_index,
                                "node_id": b.node_id,
                                "x": b.x,
                                "y": b.y,
                                "width": b.width,
                                "height": b.height,
                            })
                        }).collect::<Vec<_>>(),
                        "argument_slots": arg_slots,  // Return ALL slots (for frontend to make clickable)
                        "success": true,
                    });
                    (StatusCode::OK, Json(response))
                }
                Err(e) => {
                    let response = serde_json::json!({
                        "error": format!("Typst compilation failed: {}", e),
                        "success": false,
                    });
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
                }
            }
        }
        Err(e) => {
            let response = serde_json::json!({
                "error": format!("Invalid AST: {}", e),
                "success": false,
            });
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

// Argument slot info (for tracking editable regions)
#[derive(Debug, Clone, serde::Serialize)]
struct ArgumentSlot {
    id: String,           // UUID for filled values, or placeholder ID as string
    path: Vec<usize>,     // Path in AST (e.g., [0] = first arg of root operation)
    hint: String,         // Description of this slot
    is_placeholder: bool, // True if empty placeholder, false if filled
    role: Option<String>, // Semantic role (e.g., superscript, subscript, base)
}

// Collect ALL argument slots from expression (both empty and filled)
fn collect_argument_slots(expr: &kleis::ast::Expression) -> Vec<ArgumentSlot> {
    let mut slots = Vec::new();
    collect_slots_recursive(expr, &mut slots, vec![], None);
    slots
}

fn collect_slots_recursive(
    expr: &kleis::ast::Expression,
    slots: &mut Vec<ArgumentSlot>,
    path: Vec<usize>,
    role: Option<String>,
) {
    use kleis::ast::Expression;

    match expr {
        Expression::Placeholder { id, hint } => {
            // Empty placeholder - convert ID to string
            slots.push(ArgumentSlot {
                id: format!("ph{}", id), // Prefix with "ph" to distinguish from UUIDs
                path: path.clone(),
                hint: hint.clone(),
                is_placeholder: true,
                role: role.clone(),
            });
        }
        Expression::Const(value) | Expression::Object(value) => {
            // Filled value - generate UUID without dashes
            let uuid = uuid::Uuid::new_v4().to_string().replace("-", "");
            slots.push(ArgumentSlot {
                id: uuid,
                path: path.clone(),
                hint: format!("value: {}", value),
                is_placeholder: false,
                role: role.clone(),
            });
        }
        Expression::Operation { name, args } => {
            // Create a slot for the operation itself (for bounding box positioning)
            let uuid = uuid::Uuid::new_v4().to_string().replace("-", "");
            slots.push(ArgumentSlot {
                id: uuid,
                path: path.clone(),
                hint: format!("operation: {}", name),
                is_placeholder: false,
                role: role.clone(),
            });

            // Recursively process each argument
            for (i, arg) in args.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                let child_role = determine_arg_role(name, i);
                collect_slots_recursive(arg, slots, child_path, child_role);
            }
        }
        Expression::Match { .. } => {
            // TODO: Implement match expression slot collection
            // For now, don't collect slots from match expressions
        }
    }
}

fn determine_arg_role(op_name: &str, arg_index: usize) -> Option<String> {
    match op_name {
        "sup" | "power" => match arg_index {
            0 => Some("base".to_string()),
            1 => Some("superscript".to_string()),
            _ => None,
        },
        "sub" => match arg_index {
            0 => Some("base".to_string()),
            1 => Some("subscript".to_string()),
            _ => None,
        },
        "index" | "index_mixed" | "tensor_mixed" => match arg_index {
            0 => Some("base".to_string()),
            1 => Some("superscript".to_string()),
            2 => Some("subscript".to_string()),
            _ => None,
        },
        _ => None,
    }
}

// Health check endpoint
// Handler for type checking expressions
async fn type_check_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TypeCheckRequest>,
) -> impl IntoResponse {
    // Try to get type checker from state
    let type_checker_guard = state.type_checker.lock().unwrap();

    if type_checker_guard.is_none() {
        return Json(TypeCheckResponse {
            success: false,
            type_name: None,
            error: Some(
                "Type checker not initialized (stdlib/matrices.kleis not loaded)".to_string(),
            ),
            suggestion: None,
        });
    }

    // Parse AST from JSON
    let expr: kleis::ast::Expression = match serde_json::from_value(req.ast) {
        Ok(e) => e,
        Err(e) => {
            return Json(TypeCheckResponse {
                success: false,
                type_name: None,
                error: Some(format!("Failed to parse AST: {}", e)),
                suggestion: None,
            });
        }
    };

    // Need to clone the type checker since check() takes &mut self
    // For now, create a new one each time (TODO: make check() use &self)
    drop(type_checker_guard);

    // Re-create type checker with full stdlib (minimal_prelude + matrices)
    let result = match kleis::type_checker::TypeChecker::with_stdlib() {
        Ok(mut checker) => checker.check(&expr),
        Err(e) => {
            return Json(TypeCheckResponse {
                success: false,
                type_name: None,
                error: Some(format!("Failed to initialize TypeChecker: {}", e)),
                suggestion: None,
            });
        }
    };

    // Convert TypeCheckResult to response
    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => Json(TypeCheckResponse {
            success: true,
            type_name: Some(format!("{:?}", ty)),
            error: None,
            suggestion: None,
        }),
        kleis::type_checker::TypeCheckResult::Error {
            message,
            suggestion,
        } => Json(TypeCheckResponse {
            success: false,
            type_name: None,
            error: Some(message),
            suggestion,
        }),
        kleis::type_checker::TypeCheckResult::Polymorphic { type_var, .. } => {
            Json(TypeCheckResponse {
                success: true,
                type_name: Some(format!("Polymorphic({:?})", type_var)),
                error: None,
                suggestion: Some("Type is polymorphic - needs more context".to_string()),
            })
        }
    }
}

async fn health_handler() -> &'static str {
    "OK"
}
