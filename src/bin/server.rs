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
    // For now, just echo back the LaTeX
    // TODO: Parse LaTeX → Expression → Render
    
    let format = req.format.unwrap_or_else(|| "latex".to_string());
    
    // Placeholder: In real implementation, this would:
    // 1. Parse req.latex to Expression
    // 2. Call render_expression()
    // 3. Return result
    
    let response = RenderResponse {
        output: req.latex.clone(),
        format: format.clone(),
        success: true,
        error: None,
    };

    (StatusCode::OK, Json(response))
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

// Health check endpoint
async fn health_handler() -> &'static str {
    "OK"
}

