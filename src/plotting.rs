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
//!
//! ## Supported Plot Types (matching Lilaq)
//!
//! - `plot` - Line plot with connected points
//! - `scatter` - Scatter plot with markers
//! - `fill_between` - Shaded area between curves
//! - `bar` - Vertical bar chart
//! - `hbar` - Horizontal bar chart
//! - `stem` - Stem plot (vertical lines to points)
//! - `hstem` - Horizontal stem plot
//! - `boxplot` - Box and whisker plot
//! - `hboxplot` - Horizontal boxplot
//! - `colormesh` / `heatmap` - 2D color grid
//! - `contour` - Contour lines
//! - `quiver` - Vector/arrow field

use std::io::Write;
use std::process::Command;

/// Lilaq package version
const LILAQ_VERSION: &str = "0.3.0";

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

/// Plot type - matches Lilaq plot functions
#[derive(Debug, Clone)]
pub enum PlotType {
    /// Line plot with connected points
    Line,
    /// Scatter plot with markers only
    Scatter,
    /// Shaded area between two curves
    FillBetween,
    /// Vertical bar chart
    Bar,
    /// Horizontal bar chart
    HBar,
    /// Stem plot (vertical lines from x-axis to points)
    Stem,
    /// Horizontal stem plot
    HStem,
    /// Box and whisker plot
    Boxplot,
    /// Horizontal boxplot
    HBoxplot,
    /// 2D color mesh / heatmap
    Colormesh,
    /// Contour lines
    Contour,
    /// Vector/arrow field
    Quiver,
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
    /// Color for the plot (e.g., "blue", "#ff0000", "rgb(255, 0, 0)")
    pub color: Option<String>,
    /// Fill color for area plots
    pub fill_color: Option<String>,
    /// Line style: "solid", "dashed", "dotted"
    pub line_style: Option<String>,
    /// Line width in points
    pub line_width: Option<f64>,
    /// Opacity (0.0 to 1.0)
    pub opacity: Option<f64>,
    /// Legend label
    pub label: Option<String>,
    /// Show grid
    pub grid: bool,
    /// Error bar data (for scatter/plot)
    pub error_y: Option<Vec<f64>>,
    pub error_x: Option<Vec<f64>>,
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
            color: None,
            fill_color: None,
            line_style: None,
            line_width: None,
            opacity: None,
            label: None,
            grid: true,
            error_y: None,
            error_x: None,
        }
    }
}

/// Generate Lilaq Typst code preamble
fn generate_preamble() -> String {
    format!(
        "#import \"@preview/lilaq:{}\" as lq\n\n\
         #set page(width: auto, height: auto, margin: 0.5cm)\n\n",
        LILAQ_VERSION
    )
}

/// Format a vector of f64 as Typst array
fn format_array(data: &[f64]) -> String {
    let items: Vec<String> = data.iter().map(|x| format!("{:.6}", x)).collect();
    format!("({})", items.join(", "))
}

/// Generate Lilaq Typst code for a line/scatter plot
pub fn generate_lilaq_code(x_data: &[f64], y_data: &[f64], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

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
        PlotType::Line | PlotType::Scatter => "lq.plot",
        PlotType::Bar => "lq.bar",
        PlotType::HBar => "lq.hbar",
        PlotType::Stem => "lq.stem",
        PlotType::HStem => "lq.hstem",
        PlotType::FillBetween => "lq.fill-between",
        PlotType::Boxplot => "lq.boxplot",
        PlotType::HBoxplot => "lq.hboxplot",
        PlotType::Colormesh => "lq.colormesh",
        PlotType::Contour => "lq.contour",
        PlotType::Quiver => "lq.quiver",
    };

    code.push_str(&format!("  {}(\n", plot_cmd));
    code.push_str(&format!("    {},\n", format_array(x_data)));
    code.push_str(&format!("    {}", format_array(y_data)));

    // Add styling options
    add_styling_options(&mut code, config);

    code.push_str("\n  )\n");
    code.push_str(")\n");

    code
}

/// Add styling options to the plot command
fn add_styling_options(code: &mut String, config: &PlotConfig) {
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

    // Add color
    if let Some(color) = &config.color {
        code.push_str(&format!(",\n    stroke: {}", color));
    }

    // Add fill color
    if let Some(fill) = &config.fill_color {
        code.push_str(&format!(",\n    fill: {}", fill));
    }

    // Add opacity
    if let Some(opacity) = config.opacity {
        code.push_str(&format!(",\n    fill-opacity: {}", opacity));
    }

    // Add label for legend
    if let Some(label) = &config.label {
        code.push_str(&format!(",\n    label: \"{}\"", label));
    }
}

/// Generate Lilaq code for fill-between (shaded area under curve to y=0)
/// Lilaq fill-between takes x, y and fills between y and 0
pub fn generate_fill_between_code(x_data: &[f64], y_data: &[f64], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }
    if let Some(xlabel) = &config.xlabel {
        code.push_str(&format!("  x-label: \"{}\",\n", xlabel));
    }
    if let Some(ylabel) = &config.ylabel {
        code.push_str(&format!("  y-label: \"{}\",\n", ylabel));
    }

    // Lilaq fill-between takes: x, y and fills to y=0
    code.push_str("  lq.fill-between(\n");
    code.push_str(&format!("    {},\n", format_array(x_data)));
    code.push_str(&format!("    {}", format_array(y_data)));

    code.push_str("\n  )\n");
    code.push_str(")\n");

    code
}

/// Generate Lilaq code for boxplot
pub fn generate_boxplot_code(datasets: &[Vec<f64>], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }

    let cmd = match config.plot_type {
        PlotType::HBoxplot => "lq.hboxplot",
        _ => "lq.boxplot",
    };

    // Lilaq boxplot takes just the data array (can be nested for multiple)
    // For multiple datasets, we need multiple boxplot calls
    for data in datasets.iter() {
        code.push_str(&format!("  {}({}),\n", cmd, format_array(data)));
    }

    code.push_str(")\n");

    code
}

/// Generate Lilaq code for heatmap/colormesh
pub fn generate_heatmap_code(matrix: &[Vec<f64>], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }

    // Lilaq colormesh needs x, y coordinates matching matrix dimensions
    let nrows = matrix.len();
    let ncols = if nrows > 0 { matrix[0].len() } else { 0 };

    // Generate x coordinates (0 to ncols-1, as nrows values)
    // y coordinates must match number of rows, x must match ncols
    let x_coords: Vec<f64> = (0..ncols).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..nrows).map(|i| i as f64).collect();

    code.push_str("  lq.colormesh(\n");
    code.push_str(&format!("    {},\n", format_array(&x_coords)));
    code.push_str(&format!("    {},\n", format_array(&y_coords)));

    // Format matrix as nested array
    code.push_str("    (\n");
    for row in matrix {
        code.push_str(&format!("      {},\n", format_array(row)));
    }
    code.push_str("    )");

    code.push_str("\n  )\n");
    code.push_str(")\n");

    code
}

/// Generate Lilaq code for contour plot
pub fn generate_contour_code(
    matrix: &[Vec<f64>],
    levels: Option<&[f64]>,
    config: &PlotConfig,
) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }

    // Lilaq contour needs x, y, z coordinates (like colormesh)
    let nrows = matrix.len();
    let ncols = if nrows > 0 { matrix[0].len() } else { 0 };

    let x_coords: Vec<f64> = (0..ncols).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..nrows).map(|i| i as f64).collect();

    code.push_str("  lq.contour(\n");
    code.push_str(&format!("    {},\n", format_array(&x_coords)));
    code.push_str(&format!("    {},\n", format_array(&y_coords)));

    // Format matrix as nested array
    code.push_str("    (\n");
    for row in matrix {
        code.push_str(&format!("      {},\n", format_array(row)));
    }
    code.push_str("    )");

    // Add levels if specified
    if let Some(lvls) = levels {
        code.push_str(&format!(",\n    levels: {}", format_array(lvls)));
    }

    code.push_str("\n  )\n");
    code.push_str(")\n");

    code
}

/// Generate Lilaq code for quiver (vector field) plot
/// Lilaq quiver expects: x coords, y coords, and a 2D grid of (u, v) direction tuples
pub fn generate_quiver_code(
    x_coords: &[f64],
    y_coords: &[f64],
    directions: &[Vec<(f64, f64)>],
    config: &PlotConfig,
) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }
    if let Some(xlabel) = &config.xlabel {
        code.push_str(&format!("  x-label: \"{}\",\n", xlabel));
    }
    if let Some(ylabel) = &config.ylabel {
        code.push_str(&format!("  y-label: \"{}\",\n", ylabel));
    }

    code.push_str("  lq.quiver(\n");
    code.push_str(&format!("    {},\n", format_array(x_coords)));
    code.push_str(&format!("    {},\n", format_array(y_coords)));

    // Format directions as 2D array of (u, v) tuples
    code.push_str("    (\n");
    for row in directions {
        let tuples: Vec<String> = row
            .iter()
            .map(|(u, v)| format!("({:.6}, {:.6})", u, v))
            .collect();
        code.push_str(&format!("      ({}),\n", tuples.join(", ")));
    }
    code.push_str("    )");

    if let Some(color) = &config.color {
        code.push_str(&format!(",\n    stroke: {}", color));
    }

    code.push_str("\n  )\n");
    code.push_str(")\n");

    code
}

/// Generate Lilaq code for bar chart with numeric x positions
pub fn generate_bar_chart_code(x_data: &[f64], heights: &[f64], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: \"{}\",\n", title));
    }

    let cmd = match config.plot_type {
        PlotType::HBar => "lq.hbar",
        _ => "lq.bar",
    };

    code.push_str(&format!("  {}(\n", cmd));
    code.push_str(&format!("    {},\n", format_array(x_data)));
    code.push_str(&format!("    {}", format_array(heights)));

    if let Some(fill) = &config.fill_color {
        code.push_str(&format!(",\n    fill: {}", fill));
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
        .args([
            "compile",
            typst_path.to_str().unwrap(),
            svg_path.to_str().unwrap(),
            "--format",
            "svg",
        ])
        .output()
        .map_err(|e| format!("Failed to run typst: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Typst compilation failed: {}", stderr));
    }

    // Read SVG
    let svg =
        std::fs::read_to_string(&svg_path).map_err(|e| format!("Failed to read SVG: {}", e))?;

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
