//! Plotting module for Kleis
//!
//! Generates Lilaq/Typst code from plot expressions and compiles to SVG.
//!
//! ## Architecture
//!
//! ```text
//! plot(sin, 0..2π)  →  Sample function  →  Lilaq code  →  Typst CLI  →  SVG
//! ```
//!
//! Uses external Typst CLI with Lilaq package for publication-quality plots.

use std::process::Command;
use std::io::Write;

/// Result of a plot compilation
#[derive(Debug, Clone)]
pub struct PlotOutput {
    /// SVG content
    pub svg: String,
    /// Width in points
    pub width: f64,
    /// Height in points
    pub height: f64,
}

/// Plot type
#[derive(Debug, Clone)]
pub enum PlotType {
    Line,
    Scatter,
    Bar,
}

/// Plot configuration
#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub plot_type: PlotType,
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub width: f64,  // cm
    pub height: f64, // cm
    pub mark: Option<String>,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            plot_type: PlotType::Line,
            title: None,
            xlabel: None,
            ylabel: None,
            width: 8.0,
            height: 6.0,
            mark: None,
        }
    }
}

/// Generate Lilaq Typst code for a line/scatter plot
pub fn generate_lilaq_code(
    x_data: &[f64],
    y_data: &[f64],
    config: &PlotConfig,
) -> String {
    let mut code = String::new();
    
    // Import Lilaq
    code.push_str("#import \"@preview/lilaq:0.3.0\" as lq\n\n");
    
    // Set page size to content
    code.push_str("#set page(width: auto, height: auto, margin: 0.5cm)\n\n");
    
    // Format data arrays
    let x_str: Vec<String> = x_data.iter().map(|x| format!("{:.6}", x)).collect();
    let y_str: Vec<String> = y_data.iter().map(|y| format!("{:.6}", y)).collect();
    
    // Build diagram
    code.push_str("#lq.diagram(\n");
    
    // Add title if present
    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }
    
    // Add axis labels if present
    if let Some(xlabel) = &config.xlabel {
        code.push_str(&format!("  x-label: \"{}\",\n", xlabel));
    }
    if let Some(ylabel) = &config.ylabel {
        code.push_str(&format!("  y-label: \"{}\",\n", ylabel));
    }
    
    // Plot command based on type
    let plot_cmd = match config.plot_type {
        PlotType::Line => "lq.plot",
        PlotType::Scatter => "lq.plot",
        PlotType::Bar => "lq.bar",
    };
    
    code.push_str(&format!("  {}(\n", plot_cmd));
    code.push_str(&format!("    ({}),\n", x_str.join(", ")));
    code.push_str(&format!("    ({})", y_str.join(", ")));
    
    // Add mark for scatter
    match config.plot_type {
        PlotType::Scatter => {
            let mark = config.mark.as_deref().unwrap_or("o");
            code.push_str(&format!(",\n    mark: \"{}\"", mark));
        }
        PlotType::Line => {
            if let Some(mark) = &config.mark {
                code.push_str(&format!(",\n    mark: \"{}\"", mark));
            }
        }
        _ => {}
    }
    
    code.push_str("\n  )\n");
    code.push_str(")\n");
    
    code
}

/// Generate Lilaq code for a function plot
pub fn generate_function_plot_code(
    func_name: &str,
    x_min: f64,
    x_max: f64,
    samples: usize,
    y_data: &[f64],
    config: &PlotConfig,
) -> String {
    // Sample x values
    let x_data: Vec<f64> = (0..samples)
        .map(|i| x_min + (x_max - x_min) * (i as f64) / ((samples - 1) as f64))
        .collect();
    
    let mut cfg = config.clone();
    if cfg.title.is_none() {
        cfg.title = Some(format!("y = {}(x)", func_name));
    }
    if cfg.xlabel.is_none() {
        cfg.xlabel = Some("x".to_string());
    }
    if cfg.ylabel.is_none() {
        cfg.ylabel = Some("y".to_string());
    }
    
    generate_lilaq_code(&x_data, y_data, &cfg)
}

/// Compile Lilaq/Typst code to SVG using Typst CLI
pub fn compile_to_svg(typst_code: &str) -> Result<PlotOutput, String> {
    // Create temp file
    let temp_dir = std::env::temp_dir();
    let typst_path = temp_dir.join("kleis_plot.typ");
    let svg_path = temp_dir.join("kleis_plot.svg");
    
    // Write Typst code
    let mut file = std::fs::File::create(&typst_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(typst_code.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    // Run Typst CLI
    let output = Command::new("typst")
        .args(["compile", typst_path.to_str().unwrap(), svg_path.to_str().unwrap(), "--format", "svg"])
        .output()
        .map_err(|e| format!("Failed to run typst: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Typst compilation failed: {}", stderr));
    }
    
    // Read SVG
    let svg = std::fs::read_to_string(&svg_path)
        .map_err(|e| format!("Failed to read SVG: {}", e))?;
    
    // Extract dimensions from SVG viewBox
    let (width, height) = extract_svg_dimensions(&svg).unwrap_or((400.0, 300.0));
    
    // Clean up temp files
    let _ = std::fs::remove_file(&typst_path);
    let _ = std::fs::remove_file(&svg_path);
    
    Ok(PlotOutput { svg, width, height })
}

/// Extract width and height from SVG viewBox
fn extract_svg_dimensions(svg: &str) -> Option<(f64, f64)> {
    // Look for viewBox="x y width height"
    if let Some(start) = svg.find("viewBox=\"") {
        let rest = &svg[start + 9..];
        if let Some(end) = rest.find('"') {
            let viewbox = &rest[..end];
            let parts: Vec<&str> = viewbox.split_whitespace().collect();
            if parts.len() >= 4 {
                let width: f64 = parts[2].parse().ok()?;
                let height: f64 = parts[3].parse().ok()?;
                return Some((width, height));
            }
        }
    }
    None
}

/// Convenience function: sample a mathematical function and generate plot
pub fn plot_function<F>(
    func: F,
    x_min: f64,
    x_max: f64,
    samples: usize,
    config: &PlotConfig,
) -> Result<PlotOutput, String>
where
    F: Fn(f64) -> f64,
{
    // Sample x values
    let x_data: Vec<f64> = (0..samples)
        .map(|i| x_min + (x_max - x_min) * (i as f64) / ((samples - 1) as f64))
        .collect();
    
    // Compute y values
    let y_data: Vec<f64> = x_data.iter().map(|&x| func(x)).collect();
    
    // Generate Lilaq code
    let code = generate_lilaq_code(&x_data, &y_data, config);
    
    // Compile to SVG
    compile_to_svg(&code)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_lilaq_code() {
        let x = vec![0.0, 1.0, 2.0, 3.0];
        let y = vec![0.0, 1.0, 4.0, 9.0];
        let config = PlotConfig::default();
        
        let code = generate_lilaq_code(&x, &y, &config);
        
        assert!(code.contains("@preview/lilaq"));
        assert!(code.contains("lq.diagram"));
        assert!(code.contains("lq.plot"));
    }
    
    #[test]
    fn test_generate_scatter_code() {
        let x = vec![0.0, 1.0, 2.0];
        let y = vec![0.0, 1.0, 4.0];
        let config = PlotConfig {
            plot_type: PlotType::Scatter,
            mark: Some("o".to_string()),
            ..Default::default()
        };
        
        let code = generate_lilaq_code(&x, &y, &config);
        
        assert!(code.contains("mark: \"o\""));
    }
    
    #[test]
    #[ignore] // Requires Typst CLI
    fn test_compile_to_svg() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 1.0, 4.0, 9.0, 16.0];
        let config = PlotConfig {
            title: Some("x²".to_string()),
            ..Default::default()
        };
        
        let code = generate_lilaq_code(&x, &y, &config);
        let result = compile_to_svg(&code);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.svg.contains("<svg"));
    }
}

