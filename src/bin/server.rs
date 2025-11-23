use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
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

// Shared application state
#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {});

    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/render", post(render_handler))
        .route("/api/render_ast", post(render_ast_handler))
        .route("/api/render_typst", post(render_typst_handler))
        .route("/api/parse", post(parse_handler))
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
            Html("<h1>index.html not found. Make sure static/ directory exists.</h1>".to_string())
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
                let id = placeholder_obj.get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing or invalid placeholder id")? as usize;
                let hint = placeholder_obj.get("hint")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid placeholder hint")?
                    .to_string();
                return Ok(Expression::Placeholder { id, hint });
            }
        } else if let Some(op_val) = obj.get("Operation") {
            if let Some(op_obj) = op_val.as_object() {
                let name = op_obj.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid operation name")?
                    .to_string();
                let args_json = op_obj.get("args")
                    .and_then(|v| v.as_array())
                    .ok_or("Missing or invalid operation args")?;
                let args: Result<Vec<Expression>, String> = args_json
                    .iter()
                    .map(json_to_expression)
                    .collect();
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
    match json_to_expression(&req.ast) {
        Ok(expr) => {
            // Render to Typst markup
            let ctx = kleis::render::build_default_context();
            let typst_markup = kleis::render::render_expression(
                &expr,
                &ctx,
                &kleis::render::RenderTarget::Typst
            );
            
            // Collect ALL argument slots with their info (empty or filled)
            let arg_slots = collect_argument_slots(&expr);
            eprintln!("Argument slots: {} total", arg_slots.len());
            
            // Get unfilled placeholder IDs for Typst square rendering
            let unfilled_ids: Vec<usize> = arg_slots.iter()
                .filter(|s| s.is_placeholder)
                .map(|s| s.id)
                .collect();
            
            // Compile with Typst to get SVG
            match kleis::math_layout::compile_math_to_svg_with_ids(&typst_markup, &unfilled_ids) {
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
    id: usize,           // Placeholder ID if unfilled, or auto-assigned ID
    path: Vec<usize>,    // Path in AST (e.g., [0] = first arg of root operation)
    hint: String,        // Description of this slot
    is_placeholder: bool, // True if empty placeholder, false if filled
}

// Collect ALL argument slots from expression (both empty and filled)
fn collect_argument_slots(expr: &kleis::ast::Expression) -> Vec<ArgumentSlot> {
    let mut slots = Vec::new();
    let mut next_auto_id = 1000; // Auto IDs for filled args start at 1000
    collect_slots_recursive(expr, &mut slots, &mut next_auto_id, vec![]);
    slots
}

fn collect_slots_recursive(
    expr: &kleis::ast::Expression,
    slots: &mut Vec<ArgumentSlot>,
    next_auto_id: &mut usize,
    path: Vec<usize>
) {
    use kleis::ast::Expression;
    
    match expr {
        Expression::Placeholder { id, hint } => {
            // Empty placeholder - use its ID
            slots.push(ArgumentSlot {
                id: *id,
                path: path.clone(),
                hint: hint.clone(),
                is_placeholder: true,
            });
        }
        Expression::Const(value) | Expression::Object(value) => {
            // Filled value - assign auto ID
            slots.push(ArgumentSlot {
                id: *next_auto_id,
                path: path.clone(),
                hint: format!("value: {}", value),
                is_placeholder: false,
            });
            *next_auto_id += 1;
        }
        Expression::Operation { args, .. } => {
            // Each arg is a slot
            for (i, arg) in args.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                collect_slots_recursive(arg, slots, next_auto_id, child_path);
            }
        }
    }
}

// Health check endpoint
async fn health_handler() -> &'static str {
    "OK"
}

