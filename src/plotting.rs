//! Plotting module for Kleis
//!
//! Generates Lilaq/Typst code from plot expressions and compiles to SVG.
//!
//! ## Architecture (Compositional - matches Lilaq 1:1)
//!
//! ```text
//! diagram(options,
//!     plot(xs, ys, { color: "blue" }),
//!     bar(xs, heights, { label: "Data" }),
//!     scatter(xs, ys, { mark: "o" })
//! )
//! ```
//!
//! - Individual functions (`plot`, `bar`, `scatter`, etc.) return PlotElements
//! - `diagram()` composes elements and renders to SVG
//! - Matches Lilaq's API 1:1 for easy documentation reference
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
const LILAQ_VERSION: &str = "0.5.0";

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

/// A composable plot element (matches Lilaq's plot functions)
///
/// Individual functions like `plot()`, `bar()`, `scatter()` return PlotElements.
/// These are then composed together by `diagram()` which renders them to SVG.
#[derive(Debug, Clone)]
pub struct PlotElement {
    /// Type of plot element
    pub element_type: PlotType,
    /// X data (for most plot types)
    pub x_data: Option<Vec<f64>>,
    /// Y data (for most plot types)
    pub y_data: Option<Vec<f64>>,
    /// Y2 data (for fill-between stacking)
    pub y2_data: Option<Vec<f64>>,
    /// Matrix data (for heatmap, contour)
    pub matrix_data: Option<Vec<Vec<f64>>>,
    /// Direction data (for quiver)
    pub direction_data: Option<Vec<Vec<(f64, f64)>>>,
    /// Multiple datasets (for boxplot)
    pub datasets: Option<Vec<Vec<f64>>>,
    /// Element-specific options (matches Lilaq parameters)
    pub options: PlotElementOptions,
}

/// Options for a plot element (matches Lilaq's function parameters)
#[derive(Debug, Clone, Default)]
pub struct PlotElementOptions {
    // === Common options ===
    /// Legend label
    pub label: Option<String>,
    /// Color for the element
    pub color: Option<String>,
    /// Stroke style
    pub stroke: Option<String>,
    /// Z-index (rendering order)
    pub z_index: Option<i32>,
    /// Whether to clip to data area
    pub clip: Option<bool>,

    // === plot() specific ===
    /// Marker type
    pub mark: Option<String>,
    /// Marker size
    pub mark_size: Option<f64>,
    /// Marker color
    pub mark_color: Option<String>,
    /// X error bars
    pub xerr: Option<Vec<f64>>,
    /// Y error bars
    pub yerr: Option<Vec<f64>>,
    /// Step mode: "none", "start", "end", "center"
    pub step: Option<String>,
    /// Bézier spline interpolation
    pub smooth: Option<bool>,
    /// Show every nth mark
    pub every: Option<usize>,

    // === bar() specific ===
    /// Bar offset (for grouped bars)
    pub offset: Option<f64>,
    /// Bar width
    pub width: Option<f64>,
    /// Fill color
    pub fill: Option<String>,
    /// Base value for bars
    pub base: Option<f64>,

    // === scatter() specific ===
    /// Per-point sizes
    pub sizes: Option<Vec<f64>>,
    /// Per-point colors (floats for colormap)
    pub colors: Option<Vec<f64>>,
    /// Color map name
    pub colormap: Option<String>,
    /// Color range min
    pub color_min: Option<f64>,
    /// Color range max
    pub color_max: Option<f64>,
    /// Normalization: "linear", "log"
    pub norm: Option<String>,

    // === Opacity ===
    /// Opacity (0.0 to 1.0)
    pub opacity: Option<f64>,

    // === stem() specific ===
    /// Base stroke style
    pub base_stroke: Option<String>,

    // === quiver() specific ===
    /// Arrow scale
    pub scale: Option<f64>,
    /// Arrow pivot: "start", "center", "end"
    pub pivot: Option<String>,

    // === place() specific ===
    /// Text content for annotations
    pub text: Option<String>,
    /// Alignment: "top", "bottom", "left", "right", "center"
    pub align: Option<String>,
    /// Padding around text
    pub padding: Option<String>,

    // === yaxis() secondary axis specific ===
    /// Position: "left" or "right" for yaxis, "top" or "bottom" for xaxis
    pub position: Option<String>,
    /// Axis label for secondary axis
    pub axis_label: Option<String>,
    /// Child elements for secondary axis
    pub children: Option<Vec<Box<crate::plotting::PlotElement>>>,

    // === xaxis() secondary axis specific ===
    /// Tick distance for secondary axis
    pub tick_distance: Option<f64>,
    /// Exponent for axis labels (0 = no scientific notation)
    pub exponent: Option<i32>,
    /// Axis offset
    pub axis_offset: Option<f64>,
    /// Forward transformation function (as Kleis lambda string, e.g., "x => k / x")
    pub transform_forward: Option<String>,
    /// Inverse transformation function (as Kleis lambda string, e.g., "x => k / x")
    pub transform_inverse: Option<String>,

    // === path() specific ===
    /// Whether to close the path (connect last point to first)
    pub closed: Option<bool>,
}

/// Options for the diagram container
#[derive(Debug, Clone, Default)]
pub struct DiagramOptions {
    /// Width in cm
    pub width: Option<f64>,
    /// Height in cm
    pub height: Option<f64>,
    /// Title
    pub title: Option<String>,
    /// X-axis label
    pub xlabel: Option<String>,
    /// Y-axis label
    pub ylabel: Option<String>,
    /// X-axis limits (min, max)
    pub xlim: Option<(f64, f64)>,
    /// Y-axis limits (min, max)
    pub ylim: Option<(f64, f64)>,
    /// X-axis scale: "linear", "log", "symlog"
    pub xscale: Option<String>,
    /// Y-axis scale: "linear", "log", "symlog"
    pub yscale: Option<String>,
    /// Legend position: "left + top", "right + bottom", etc.
    pub legend_position: Option<String>,
    /// Grid options
    pub grid: Option<bool>,
    /// Background fill color
    pub fill: Option<String>,
    /// Aspect ratio
    pub aspect_ratio: Option<f64>,
    /// X-axis subticks: "none", "auto", or number
    pub xaxis_subticks: Option<String>,
    /// Y-axis subticks: "none", "auto", or number
    pub yaxis_subticks: Option<String>,
    /// Y-axis mirror: show on both sides
    pub yaxis_mirror: Option<bool>,
    /// Margin (top, bottom, left, right percentages)
    pub margin_top: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub margin_right: Option<String>,
    /// Custom x-axis tick labels (or empty vec to hide ticks)
    pub xaxis_ticks: Option<Vec<String>>,
    /// X-axis tick label rotation in degrees (e.g., -90 for vertical)
    pub xaxis_tick_rotate: Option<f64>,
    /// Hide x-axis ticks entirely
    pub xaxis_ticks_none: Option<bool>,
    /// Hide y-axis ticks entirely
    pub yaxis_ticks_none: Option<bool>,
    /// X-axis tick unit (e.g., π for multiples of pi)
    pub xaxis_tick_unit: Option<f64>,
    /// X-axis tick suffix (e.g., "π" to append to tick labels)
    pub xaxis_tick_suffix: Option<String>,
    /// Y-axis tick unit
    pub yaxis_tick_unit: Option<f64>,
    /// Y-axis tick suffix
    pub yaxis_tick_suffix: Option<String>,
    /// Theme: "schoolbook", "dark", etc.
    pub theme: Option<String>,
}

/// Plot type - matches Lilaq plot functions
#[derive(Debug, Clone, PartialEq)]
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
    /// Grouped bar chart with multiple series
    GroupedBars,
    /// Text annotation at a specific position
    Place,
    /// Secondary Y-axis wrapper (for twin axis charts)
    SecondaryYAxis,
    /// Secondary X-axis wrapper (for dual axis charts like wavelength/energy)
    SecondaryXAxis,
    /// Arbitrary path (for fractals, polygons, custom shapes)
    Path,
}

impl PlotType {
    /// Parse plot type from string (for unified graph() API)
    pub fn parse(s: &str) -> Option<PlotType> {
        match s.to_lowercase().as_str() {
            "line" | "plot" => Some(PlotType::Line),
            "scatter" => Some(PlotType::Scatter),
            "fill_between" | "fillbetween" | "area" => Some(PlotType::FillBetween),
            "bar" => Some(PlotType::Bar),
            "hbar" | "barh" => Some(PlotType::HBar),
            "stem" => Some(PlotType::Stem),
            "hstem" | "stemh" => Some(PlotType::HStem),
            "boxplot" | "box" => Some(PlotType::Boxplot),
            "hboxplot" | "boxh" => Some(PlotType::HBoxplot),
            "heatmap" | "colormesh" => Some(PlotType::Colormesh),
            "contour" => Some(PlotType::Contour),
            "quiver" | "vector" | "arrows" => Some(PlotType::Quiver),
            "grouped_bars" | "groupedbars" | "grouped" => Some(PlotType::GroupedBars),
            _ => None,
        }
    }

    /// Get all valid type names (for error messages)
    pub fn valid_names() -> &'static [&'static str] {
        &[
            "line",
            "scatter",
            "fill_between",
            "bar",
            "hbar",
            "stem",
            "hstem",
            "boxplot",
            "hboxplot",
            "heatmap",
            "contour",
            "quiver",
            "grouped_bars",
        ]
    }
}

/// Plot configuration - comprehensive options matching Lilaq API
#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub plot_type: PlotType,
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub width: f64,  // cm
    pub height: f64, // cm

    // === Line/Mark Styling (Phase 2) ===
    /// Marker type: "o", "x", "star", "d" (diamond), "s" (square), etc.
    pub mark: Option<String>,
    /// Marker size in points
    pub mark_size: Option<f64>,
    /// Marker color (separate from line color)
    pub mark_color: Option<String>,
    /// Combined color for line and marks
    pub color: Option<String>,
    /// Line stroke style (width, dash pattern)
    pub stroke: Option<String>,
    /// Fill color for area plots
    pub fill_color: Option<String>,
    /// Opacity (0.0 to 1.0)
    pub opacity: Option<f64>,

    // === Error Bars ===
    /// Y error bars (symmetric or asymmetric)
    pub yerr: Option<Vec<f64>>,
    /// X error bars (symmetric or asymmetric)
    pub xerr: Option<Vec<f64>>,

    // === Line Interpolation ===
    /// Step mode: "none", "start", "end", "center"
    pub step: Option<String>,
    /// Bézier spline interpolation
    pub smooth: bool,

    // === Display Options ===
    /// Mark interval (show every nth mark)
    pub every: Option<usize>,
    /// Legend label
    pub label: Option<String>,
    /// Clip to data area
    pub clip: bool,
    /// Rendering order (z-index)
    pub z_index: Option<i32>,

    // === Scatter-specific (per-point styling) ===
    /// Per-point sizes (for scatter)
    pub sizes: Option<Vec<f64>>,
    /// Per-point colors (for scatter) - floats for colormap
    pub colors: Option<Vec<f64>>,
    /// Color map: "viridis", "magma", "plasma", "inferno", "cividis"
    pub colormap: Option<String>,
    /// Color range min
    pub color_min: Option<f64>,
    /// Color range max
    pub color_max: Option<f64>,
    /// Color normalization: "linear", "log"
    pub norm: Option<String>,

    // === Stem-specific ===
    /// Baseline y-coordinate for stem plots
    pub base: Option<f64>,
    /// Baseline stroke style
    pub base_stroke: Option<String>,

    // === Bar-specific (Phase 3) ===
    /// Horizontal offset for grouped bars (e.g., -0.2 for left, 0.2 for right)
    pub bar_offset: Option<f64>,
    /// Bar width (default 0.8 in Lilaq)
    pub bar_width: Option<f64>,

    // === Legacy compat ===
    pub grid: bool,
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
            // Line/Mark Styling
            mark: None,
            mark_size: None,
            mark_color: None,
            color: None,
            stroke: None,
            fill_color: None,
            opacity: None,
            // Error Bars
            yerr: None,
            xerr: None,
            // Line Interpolation
            step: None,
            smooth: false,
            // Display Options
            every: None,
            label: None,
            clip: true,
            z_index: None,
            // Scatter-specific
            sizes: None,
            colors: None,
            colormap: None,
            color_min: None,
            color_max: None,
            norm: None,
            // Stem-specific
            base: None,
            base_stroke: None,
            // Bar-specific
            bar_offset: None,
            bar_width: None,
            // Legacy
            grid: true,
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

/// Generate preamble with theme support
///
/// Lilaq has built-in themes in lq.theme module:
/// - lq.theme.misty
/// - lq.theme.ocean
/// - lq.theme.skyline  
/// - lq.theme.schoolbook
/// - lq.theme.moon (dark theme)
///
/// For "schoolbook", we use the full manual implementation with arrow tips.
/// See: https://lilaq.org/themes
fn generate_preamble_with_theme(theme: Option<&str>) -> String {
    let mut code = String::new();

    // Base imports
    code.push_str(&format!(
        "#import \"@preview/lilaq:{}\" as lq\n",
        LILAQ_VERSION
    ));

    // Theme-specific imports
    if let Some("schoolbook") = theme {
        // Full manual implementation requires tiptoe and elembic
        code.push_str("#import \"@preview/tiptoe:0.3.1\"\n");
        code.push_str("#import \"@preview/elembic:1.1.1\" as e\n");
    }

    code.push_str("\n#set page(width: auto, height: auto, margin: 0.5cm)\n\n");

    // Apply theme
    match theme {
        Some("schoolbook") => {
            // Full manual schoolbook implementation with arrow tips
            code.push_str(
                r#"#let schoolbook-style = it => {
  let filter(value, distance) = value != 0 and distance >= 5pt
  let axis-args = (position: 0, filter: filter)
  
  show: lq.set-tick(inset: 1.5pt, outset: 1.5pt, pad: 0.4em)
  show: lq.set-spine(tip: tiptoe.stealth)
  show: lq.set-grid(stroke: none)

  show: lq.set-diagram(xaxis: axis-args, yaxis: axis-args)

  show: lq.set-label(pad: none, angle: 0deg)
  show: e.show_(
    lq.label.with(kind: "y"),
    it => place(bottom + right, dy: -100% - .0em, dx: -.5em, it)
  )
  show: e.show_(
    lq.label.with(kind: "x"),
    it => place(left + top, dx: 100% + .0em, dy: .4em, it)
  )
  
  it
}

#show: schoolbook-style

"#,
            );
        }
        Some("moon") | Some("dark") => {
            // Moon theme requires dark background
            code.push_str("#set page(fill: rgb(\"#1a1a2e\"))\n");
            code.push_str("#set text(fill: white)\n");
            code.push_str("#show: lq.theme.moon\n\n");
        }
        Some("misty") => {
            code.push_str("#show: lq.theme.misty\n\n");
        }
        Some("ocean") => {
            code.push_str("#show: lq.theme.ocean\n\n");
        }
        Some("skyline") => {
            code.push_str("#show: lq.theme.skyline\n\n");
        }
        _ => {}
    }

    code
}

/// Format a vector of f64 as Typst array
fn format_array(data: &[f64]) -> String {
    let items: Vec<String> = data.iter().map(|x| format!("{:.6}", x)).collect();
    // Typst requires trailing comma for single-element arrays/tuples
    if items.len() == 1 {
        format!("({},)", items[0])
    } else {
        format!("({})", items.join(", "))
    }
}

fn format_matrix(data: &[Vec<f64>]) -> String {
    let rows: Vec<String> = data
        .iter()
        .map(|row| {
            let items: Vec<String> = row.iter().map(|x| format!("{:.6}", x)).collect();
            format!("({})", items.join(", "))
        })
        .collect();
    format!("({})", rows.join(", "))
}

/// Generate Lilaq Typst code for a line/scatter plot
pub fn generate_lilaq_code(x_data: &[f64], y_data: &[f64], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

    // Build diagram
    code.push_str("#lq.diagram(\n");

    // Add title if present
    if let Some(title) = &config.title {
        code.push_str(&format!("  title: [{}],\n", title));
    }

    // Add axis labels if present
    if let Some(xlabel) = &config.xlabel {
        code.push_str(&format!("  x-label: [{}],\n", xlabel));
    }
    if let Some(ylabel) = &config.ylabel {
        code.push_str(&format!("  y-label: [{}],\n", ylabel));
    }

    // Plot command based on type
    // Note: GroupedBars uses its own generate_grouped_bar_code() function
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
        PlotType::GroupedBars => "lq.bar", // Uses generate_grouped_bar_code instead
        PlotType::Place => "lq.place",
        PlotType::SecondaryYAxis => "lq.yaxis",
        PlotType::SecondaryXAxis => "lq.xaxis",
        PlotType::Path => "lq.path",
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
    // === Mark styling ===
    match config.plot_type {
        PlotType::Scatter => {
            let mark = config.mark.as_deref().unwrap_or("o");
            code.push_str(&format!(",\n    mark: \"{}\"", mark));
        }
        PlotType::Line | PlotType::Stem | PlotType::HStem => {
            if let Some(mark) = &config.mark {
                code.push_str(&format!(",\n    mark: \"{}\"", mark));
            }
        }
        _ => {}
    }

    if let Some(size) = config.mark_size {
        code.push_str(&format!(",\n    mark-size: {}pt", size));
    }

    if let Some(mark_color) = &config.mark_color {
        code.push_str(&format!(",\n    mark-color: {}", mark_color));
    }

    // === Color / Stroke ===
    if let Some(color) = &config.color {
        code.push_str(&format!(",\n    color: {}", color));
    }

    if let Some(stroke) = &config.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }

    if let Some(fill) = &config.fill_color {
        code.push_str(&format!(",\n    fill: {}", fill));
    }

    if let Some(opacity) = config.opacity {
        code.push_str(&format!(",\n    alpha: {}%", (opacity * 100.0) as i32));
    }

    // === Error bars ===
    if let Some(yerr) = &config.yerr {
        code.push_str(&format!(",\n    yerr: {}", format_array(yerr)));
    }

    if let Some(xerr) = &config.xerr {
        code.push_str(&format!(",\n    xerr: {}", format_array(xerr)));
    }

    // === Line interpolation ===
    if let Some(step) = &config.step {
        code.push_str(&format!(",\n    step: {}", step));
    }

    if config.smooth {
        code.push_str(",\n    smooth: true");
    }

    // === Display options ===
    if let Some(every) = config.every {
        code.push_str(&format!(",\n    every: {}", every));
    }

    if let Some(label) = &config.label {
        code.push_str(&format!(",\n    label: \"{}\"", label));
    }

    if !config.clip {
        code.push_str(",\n    clip: false");
    }

    if let Some(z) = config.z_index {
        code.push_str(&format!(",\n    z-index: {}", z));
    }

    // === Scatter-specific: per-point styling ===
    if let Some(sizes) = &config.sizes {
        code.push_str(&format!(",\n    size: {}", format_array(sizes)));
    }

    if let Some(colors) = &config.colors {
        code.push_str(&format!(",\n    color: {}", format_array(colors)));
    }

    if let Some(cmap) = &config.colormap {
        code.push_str(&format!(",\n    map: color.map.{}", cmap));
    }

    if let Some(cmin) = config.color_min {
        code.push_str(&format!(",\n    min: {}", cmin));
    }

    if let Some(cmax) = config.color_max {
        code.push_str(&format!(",\n    max: {}", cmax));
    }

    if let Some(norm) = &config.norm {
        code.push_str(&format!(",\n    norm: \"{}\"", norm));
    }

    // === Stem-specific ===
    if let Some(base) = config.base {
        code.push_str(&format!(",\n    base: {}", base));
    }

    if let Some(base_stroke) = &config.base_stroke {
        code.push_str(&format!(",\n    base-stroke: {}", base_stroke));
    }
}

/// Generate Lilaq code for fill-between (shaded area under curve to y=0)
/// Lilaq fill-between takes x, y and fills between y and 0
pub fn generate_fill_between_code(x_data: &[f64], y_data: &[f64], config: &PlotConfig) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    if let Some(title) = &config.title {
        code.push_str(&format!("  title: [{}],\n", title));
    }
    if let Some(xlabel) = &config.xlabel {
        code.push_str(&format!("  x-label: [{}],\n", xlabel));
    }
    if let Some(ylabel) = &config.ylabel {
        code.push_str(&format!("  y-label: [{}],\n", ylabel));
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
        code.push_str(&format!("  title: [{}],\n", title));
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
        code.push_str(&format!("  title: [{}],\n", title));
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
        code.push_str(&format!("  title: [{}],\n", title));
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
        code.push_str(&format!("  title: [{}],\n", title));
    }
    if let Some(xlabel) = &config.xlabel {
        code.push_str(&format!("  x-label: [{}],\n", xlabel));
    }
    if let Some(ylabel) = &config.ylabel {
        code.push_str(&format!("  y-label: [{}],\n", ylabel));
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
        code.push_str(&format!("  title: [{}],\n", title));
    }

    let cmd = match config.plot_type {
        PlotType::HBar => "lq.hbar",
        _ => "lq.bar",
    };

    code.push_str(&format!("  {}(\n", cmd));
    code.push_str(&format!("    {},\n", format_array(x_data)));
    code.push_str(&format!("    {}", format_array(heights)));

    // Bar offset for grouped bars
    if let Some(offset) = config.bar_offset {
        code.push_str(&format!(",\n    offset: {}", offset));
    }

    // Bar width (default 0.8 in Lilaq)
    if let Some(width) = config.bar_width {
        code.push_str(&format!(",\n    width: {}", width));
    }

    // Fill color
    if let Some(fill) = &config.fill_color {
        code.push_str(&format!(",\n    fill: {}", fill));
    }

    // Label for legend
    if let Some(label) = &config.label {
        code.push_str(&format!(",\n    label: [{}]", label));
    }

    code.push_str("\n  )\n");
    code.push_str(")\n");

    code
}

/// A single bar series: (heights, label, optional error bars)
pub type BarSeries = (Vec<f64>, String, Option<Vec<f64>>);

/// Generate Lilaq code for grouped bar charts with optional error bars
///
/// This creates a single diagram with multiple bar series, automatically
/// calculating offsets for side-by-side grouping.
///
/// # Arguments
/// * `x_data` - X coordinates for all bars
/// * `series` - Vec of (heights, label, optional yerr) for each series
/// * `config` - Plot configuration (title, etc.)
pub fn generate_grouped_bar_code(
    x_data: &[f64],
    series: &[BarSeries],
    config: &PlotConfig,
) -> String {
    let mut code = generate_preamble();

    code.push_str("#lq.diagram(\n");

    // Diagram-level options
    if let Some(title) = &config.title {
        code.push_str(&format!("  title: [{}],\n", title));
    }

    // Calculate bar width and offsets based on number of series
    let n_series = series.len() as f64;
    let total_width = 0.8; // Total width for all bars at each x
    let bar_width = total_width / n_series;
    let start_offset = -total_width / 2.0 + bar_width / 2.0;

    // Legend configuration
    code.push_str("  legend: (position: left + top),\n");

    // Generate each bar series
    for (i, (heights, label, yerr)) in series.iter().enumerate() {
        let offset = start_offset + (i as f64) * bar_width;

        code.push_str("  lq.bar(\n");
        code.push_str(&format!("    {},\n", format_array(x_data)));
        code.push_str(&format!("    {}", format_array(heights)));
        code.push_str(&format!(",\n    offset: {:.2}", offset));
        code.push_str(&format!(",\n    width: {:.2}", bar_width));
        code.push_str(&format!(",\n    label: [{}]", label));
        code.push_str("\n  ),\n");

        // Add error bars as a plot with stroke: none
        if let Some(err) = yerr {
            // Offset x coordinates to match bar positions
            let offset_x: Vec<f64> = x_data.iter().map(|x| x + offset).collect();
            code.push_str("  lq.plot(\n");
            code.push_str(&format!("    {},\n", format_array(&offset_x)));
            code.push_str(&format!("    {}", format_array(heights)));
            code.push_str(&format!(",\n    yerr: {}", format_array(err)));
            code.push_str(",\n    color: black");
            code.push_str(",\n    stroke: none");
            code.push_str("\n  ),\n");
        }
    }

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

// =============================================================================
// COMPOSITIONAL API (Lilaq-style diagram with multiple elements)
// =============================================================================

/// Generate Lilaq code for a diagram with multiple plot elements
pub fn generate_diagram_code(elements: &[PlotElement], options: &DiagramOptions) -> String {
    // Use theme-aware preamble if theme is specified
    let mut code = generate_preamble_with_theme(options.theme.as_deref());

    // Start diagram
    code.push_str("#lq.diagram(\n");

    // Diagram options
    if let Some(w) = options.width {
        code.push_str(&format!("  width: {}cm,\n", w));
    }
    if let Some(h) = options.height {
        code.push_str(&format!("  height: {}cm,\n", h));
    }
    if let Some(ref title) = options.title {
        code.push_str(&format!("  title: [{}],\n", title));
    }
    if let Some(ref xlabel) = options.xlabel {
        code.push_str(&format!("  xlabel: [{}],\n", xlabel));
    }
    if let Some(ref ylabel) = options.ylabel {
        code.push_str(&format!("  ylabel: [{}],\n", ylabel));
    }
    if let Some((min, max)) = options.xlim {
        code.push_str(&format!("  xlim: ({}, {}),\n", min, max));
    }
    if let Some((min, max)) = options.ylim {
        code.push_str(&format!("  ylim: ({}, {}),\n", min, max));
    }
    // Note: xscale and yscale are handled in xaxis/yaxis options below
    if let Some(ref pos) = options.legend_position {
        code.push_str(&format!("  legend: (position: {}),\n", pos));
    }
    if options.grid == Some(true) {
        code.push_str("  grid: auto,\n");
    } else if options.grid == Some(false) {
        code.push_str("  grid: none,\n");
    }
    if let Some(ref fill) = options.fill {
        code.push_str(&format!("  fill: {},\n", fill));
    }
    // X-axis options
    let mut xaxis_opts = Vec::new();
    // Scale: "linear" (default), "log", "symlog"
    if let Some(ref xscale) = options.xscale {
        if xscale != "linear" {
            xaxis_opts.push(format!("scale: \"{}\"", xscale));
        }
    }
    if options.xaxis_ticks_none == Some(true) {
        xaxis_opts.push("ticks: none".to_string());
    }
    if let Some(ref subticks) = options.xaxis_subticks {
        xaxis_opts.push(format!("subticks: {}", subticks));
    }
    // Tick unit for locating ticks at multiples (e.g., π)
    if let Some(unit) = options.xaxis_tick_unit {
        xaxis_opts.push(format!(
            "locate-ticks: lq.tick-locate.linear.with(unit: {})",
            unit
        ));
    }
    // Tick suffix for formatting (e.g., "π")
    if let Some(ref suffix) = options.xaxis_tick_suffix {
        xaxis_opts.push(format!(
            "format-ticks: lq.tick-format.linear.with(suffix: ${}$)",
            suffix
        ));
    }
    if options.xaxis_ticks_none != Some(true) {
        if let Some(ref ticks) = options.xaxis_ticks {
            // Format ticks as enumerated pairs: ((0, "Jan"), (1, "Feb"), ...)
            // Apply rotation if specified
            let tick_strs: Vec<String> = if let Some(degrees) = options.xaxis_tick_rotate {
                ticks
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        format!("({}, rotate({}deg, reflow: true)[{}])", i, degrees, label)
                    })
                    .collect()
            } else {
                ticks
                    .iter()
                    .enumerate()
                    .map(|(i, label)| format!("({}, [{}])", i, label))
                    .collect()
            };
            xaxis_opts.push(format!("ticks: ({})", tick_strs.join(", ")));
        }
    }
    if !xaxis_opts.is_empty() {
        code.push_str(&format!("  xaxis: ({}),\n", xaxis_opts.join(", ")));
    }

    // Y-axis options
    let mut yaxis_opts = Vec::new();
    // Scale: "linear" (default), "log", "symlog"
    if let Some(ref yscale) = options.yscale {
        if yscale != "linear" {
            yaxis_opts.push(format!("scale: \"{}\"", yscale));
        }
    }
    if options.yaxis_ticks_none == Some(true) {
        yaxis_opts.push("ticks: none".to_string());
    }
    if let Some(ref subticks) = options.yaxis_subticks {
        yaxis_opts.push(format!("subticks: {}", subticks));
    }
    if let Some(mirror) = options.yaxis_mirror {
        yaxis_opts.push(format!("mirror: {}", mirror));
    }
    // Tick unit for locating ticks at multiples
    if let Some(unit) = options.yaxis_tick_unit {
        yaxis_opts.push(format!(
            "locate-ticks: lq.tick-locate.linear.with(unit: {})",
            unit
        ));
    }
    // Tick suffix for formatting
    if let Some(ref suffix) = options.yaxis_tick_suffix {
        yaxis_opts.push(format!(
            "format-ticks: lq.tick-format.linear.with(suffix: ${}$)",
            suffix
        ));
    }
    if !yaxis_opts.is_empty() {
        code.push_str(&format!("  yaxis: ({}),\n", yaxis_opts.join(", ")));
    }

    // Margin options
    let mut margin_opts = Vec::new();
    if let Some(ref top) = options.margin_top {
        margin_opts.push(format!("top: {}", top));
    }
    if let Some(ref bottom) = options.margin_bottom {
        margin_opts.push(format!("bottom: {}", bottom));
    }
    if let Some(ref left) = options.margin_left {
        margin_opts.push(format!("left: {}", left));
    }
    if let Some(ref right) = options.margin_right {
        margin_opts.push(format!("right: {}", right));
    }
    if !margin_opts.is_empty() {
        code.push_str(&format!("  margin: ({}),\n", margin_opts.join(", ")));
    }
    if let Some(ratio) = options.aspect_ratio {
        code.push_str(&format!("  aspect-ratio: {},\n", ratio));
    }

    // Add each element
    for element in elements {
        code.push_str(&generate_element_code(element));
    }

    code.push_str(")\n");
    code
}

/// Generate Lilaq code for a single plot element
fn generate_element_code(element: &PlotElement) -> String {
    match element.element_type {
        PlotType::Line => generate_plot_element(element),
        PlotType::Scatter => generate_scatter_element(element),
        PlotType::Bar => generate_bar_element(element, false),
        PlotType::HBar => generate_bar_element(element, true),
        PlotType::Stem => generate_stem_element(element, false),
        PlotType::HStem => generate_stem_element(element, true),
        PlotType::FillBetween => generate_fill_between_element(element),
        PlotType::Boxplot => generate_boxplot_element(element, false),
        PlotType::HBoxplot => generate_boxplot_element(element, true),
        PlotType::Colormesh => generate_colormesh_element(element),
        PlotType::Contour => generate_contour_element(element),
        PlotType::Quiver => generate_quiver_element(element),
        PlotType::GroupedBars => String::new(), // Handled by multiple bar elements
        PlotType::Place => generate_place_element(element),
        PlotType::SecondaryYAxis => generate_yaxis_element(element),
        PlotType::SecondaryXAxis => generate_xaxis_element(element),
        PlotType::Path => generate_path_element(element),
    }
}

fn generate_plot_element(element: &PlotElement) -> String {
    let mut code = String::new();
    let x = element.x_data.as_ref().unwrap();
    let y = element.y_data.as_ref().unwrap();

    code.push_str("  lq.plot(\n");
    code.push_str(&format!("    {},\n", format_array(x)));
    code.push_str(&format!("    {}", format_array(y)));

    // Options
    let opts = &element.options;
    if let Some(ref color) = opts.color {
        code.push_str(&format!(",\n    color: {}", color));
    }
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }
    if let Some(ref mark) = opts.mark {
        code.push_str(&format!(",\n    mark: \"{}\"", mark));
    }
    if let Some(size) = opts.mark_size {
        code.push_str(&format!(",\n    mark-size: {}pt", size));
    }
    if let Some(ref xerr) = opts.xerr {
        code.push_str(&format!(",\n    xerr: {}", format_array(xerr)));
    }
    if let Some(ref yerr) = opts.yerr {
        code.push_str(&format!(",\n    yerr: {}", format_array(yerr)));
    }
    if let Some(ref step) = opts.step {
        code.push_str(&format!(",\n    step: {}", step));
    }
    if opts.smooth == Some(true) {
        code.push_str(",\n    smooth: true");
    }
    if let Some(every) = opts.every {
        code.push_str(&format!(",\n    every: {}", every));
    }
    if let Some(ref label) = opts.label {
        code.push_str(&format!(",\n    label: [{}]", label));
    }
    if opts.clip == Some(false) {
        code.push_str(",\n    clip: false");
    }
    if let Some(z) = opts.z_index {
        code.push_str(&format!(",\n    z-index: {}", z));
    }
    if let Some(opacity) = opts.opacity {
        code.push_str(&format!(",\n    alpha: {}%", (opacity * 100.0) as i32));
    }

    code.push_str("\n  ),\n");
    code
}

/// Generate Lilaq code for a scatter plot element using lq.scatter
fn generate_scatter_element(element: &PlotElement) -> String {
    let mut code = String::new();
    let x = element.x_data.as_ref().unwrap();
    let y = element.y_data.as_ref().unwrap();

    code.push_str("  lq.scatter(\n");
    code.push_str(&format!("    {},\n", format_array(x)));
    code.push_str(&format!("    {}", format_array(y)));

    let opts = &element.options;

    // Mark style (default to circle)
    let mark = opts.mark.as_deref().unwrap_or("o");
    code.push_str(&format!(",\n    mark: \"{}\"", mark));

    // Per-point colors (array of floats 0-1 for colormap)
    if let Some(ref colors) = opts.colors {
        code.push_str(&format!(",\n    color: {}", format_array(colors)));
    } else if let Some(ref color) = opts.color {
        // Single color for all points
        code.push_str(&format!(",\n    color: {}", color));
    }

    // Colormap for per-point colors
    if let Some(ref cmap) = opts.colormap {
        code.push_str(&format!(",\n    map: color.map.{}", cmap));
    }

    // Stroke (outline)
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }

    // Mark size
    if let Some(size) = opts.mark_size {
        code.push_str(&format!(",\n    size: {}pt", size));
    }

    // Label for legend
    if let Some(ref label) = opts.label {
        code.push_str(&format!(",\n    label: [{}]", label));
    }

    // Z-index
    if let Some(z) = opts.z_index {
        code.push_str(&format!(",\n    z-index: {}", z));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_bar_element(element: &PlotElement, horizontal: bool) -> String {
    let mut code = String::new();
    let x = element.x_data.as_ref().unwrap();
    let y = element.y_data.as_ref().unwrap();

    let cmd = if horizontal { "lq.hbar" } else { "lq.bar" };
    code.push_str(&format!("  {}(\n", cmd));
    code.push_str(&format!("    {},\n", format_array(x)));
    code.push_str(&format!("    {}", format_array(y)));

    let opts = &element.options;
    if let Some(offset) = opts.offset {
        code.push_str(&format!(",\n    offset: {}", offset));
    }
    if let Some(width) = opts.width {
        code.push_str(&format!(",\n    width: {}", width));
    }
    if let Some(ref fill) = opts.fill {
        code.push_str(&format!(",\n    fill: {}", fill));
    }
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }
    if let Some(base) = opts.base {
        code.push_str(&format!(",\n    base: {}", base));
    }
    if let Some(ref label) = opts.label {
        code.push_str(&format!(",\n    label: [{}]", label));
    }
    if let Some(z) = opts.z_index {
        code.push_str(&format!(",\n    z-index: {}", z));
    }
    // For bar charts, apply opacity through fill color with alpha
    if let Some(opacity) = opts.opacity {
        // If we have a fill color, make it transparent; otherwise use green with alpha
        let base_color = opts.fill.as_deref().unwrap_or("green");
        code.push_str(&format!(
            ",\n    fill: {}.transparentize({}%)",
            base_color,
            ((1.0 - opacity) * 100.0) as i32
        ));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_stem_element(element: &PlotElement, horizontal: bool) -> String {
    let mut code = String::new();
    let x = element.x_data.as_ref().unwrap();
    let y = element.y_data.as_ref().unwrap();

    let cmd = if horizontal { "lq.hstem" } else { "lq.stem" };
    code.push_str(&format!("  {}(\n", cmd));
    code.push_str(&format!("    {},\n", format_array(x)));
    code.push_str(&format!("    {}", format_array(y)));

    let opts = &element.options;
    if let Some(ref color) = opts.color {
        code.push_str(&format!(",\n    color: {}", color));
    }
    if let Some(ref mark) = opts.mark {
        code.push_str(&format!(",\n    mark: \"{}\"", mark));
    }
    if let Some(size) = opts.mark_size {
        code.push_str(&format!(",\n    mark-size: {}pt", size));
    }
    if let Some(base) = opts.base {
        code.push_str(&format!(",\n    base: {}", base));
    }
    if let Some(ref base_stroke) = opts.base_stroke {
        code.push_str(&format!(",\n    base-stroke: {}", base_stroke));
    }
    if let Some(ref label) = opts.label {
        code.push_str(&format!(",\n    label: [{}]", label));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_fill_between_element(element: &PlotElement) -> String {
    let mut code = String::new();
    let x = element.x_data.as_ref().unwrap();
    let y = element.y_data.as_ref().unwrap();

    code.push_str("  lq.fill-between(\n");
    code.push_str(&format!("    {},\n", format_array(x)));
    code.push_str(&format!("    {}", format_array(y)));

    // y2 for stacked area charts
    if let Some(ref y2) = element.y2_data {
        code.push_str(&format!(",\n    y2: {}", format_array(y2)));
    }

    let opts = &element.options;
    if let Some(ref fill) = opts.fill {
        code.push_str(&format!(",\n    fill: {}", fill));
    }
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }
    if let Some(ref label) = opts.label {
        code.push_str(&format!(",\n    label: [{}]", label));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_boxplot_element(element: &PlotElement, horizontal: bool) -> String {
    let mut code = String::new();
    let datasets = element.datasets.as_ref().unwrap();

    let cmd = if horizontal {
        "lq.hboxplot"
    } else {
        "lq.boxplot"
    };

    for dataset in datasets {
        code.push_str(&format!("  {}(\n", cmd));
        code.push_str(&format!("    {}", format_array(dataset)));

        let opts = &element.options;
        if let Some(ref fill) = opts.fill {
            code.push_str(&format!(",\n    fill: {}", fill));
        }
        if let Some(ref stroke) = opts.stroke {
            code.push_str(&format!(",\n    stroke: {}", stroke));
        }

        code.push_str("\n  ),\n");
    }
    code
}

fn generate_colormesh_element(element: &PlotElement) -> String {
    let mut code = String::new();
    let matrix = element.matrix_data.as_ref().unwrap();

    let rows = matrix.len();
    let cols = if rows > 0 { matrix[0].len() } else { 0 };

    // Generate coordinate arrays (must match matrix dimensions)
    let x_coords: Vec<f64> = (0..cols).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..rows).map(|i| i as f64).collect();

    code.push_str("  lq.colormesh(\n");
    code.push_str(&format!("    {},\n", format_array(&x_coords)));
    code.push_str(&format!("    {},\n", format_array(&y_coords)));
    code.push_str(&format!("    {}", format_matrix(matrix)));

    let opts = &element.options;
    if let Some(ref colormap) = opts.colormap {
        code.push_str(&format!(",\n    map: color.map.{}", colormap));
    }
    if let Some(min) = opts.color_min {
        code.push_str(&format!(",\n    min: {}", min));
    }
    if let Some(max) = opts.color_max {
        code.push_str(&format!(",\n    max: {}", max));
    }
    if let Some(ref norm) = opts.norm {
        code.push_str(&format!(",\n    norm: \"{}\"", norm));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_contour_element(element: &PlotElement) -> String {
    let mut code = String::new();
    let matrix = element.matrix_data.as_ref().unwrap();

    let rows = matrix.len();
    let cols = if rows > 0 { matrix[0].len() } else { 0 };

    let x_coords: Vec<f64> = (0..cols).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..rows).map(|i| i as f64).collect();

    code.push_str("  lq.contour(\n");
    code.push_str(&format!("    {},\n", format_array(&x_coords)));
    code.push_str(&format!("    {},\n", format_array(&y_coords)));
    code.push_str(&format!("    {}", format_matrix(matrix)));

    let opts = &element.options;
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_quiver_element(element: &PlotElement) -> String {
    let mut code = String::new();
    let x = element.x_data.as_ref().unwrap();
    let y = element.y_data.as_ref().unwrap();
    let directions = element.direction_data.as_ref().unwrap();

    code.push_str("  lq.quiver(\n");
    code.push_str(&format!("    {},\n", format_array(x)));
    code.push_str(&format!("    {},\n", format_array(y)));

    // Format directions as 2D array of (u, v) tuples
    let mut dir_str = String::from("(\n");
    for row in directions {
        dir_str.push_str("      (");
        for (i, (u, v)) in row.iter().enumerate() {
            if i > 0 {
                dir_str.push_str(", ");
            }
            dir_str.push_str(&format!("({}, {})", u, v));
        }
        dir_str.push_str("),\n");
    }
    dir_str.push_str("    )");
    code.push_str(&format!("    {}", dir_str));

    let opts = &element.options;
    if let Some(ref color) = opts.color {
        code.push_str(&format!(",\n    color: {}", color));
    }
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!(",\n    stroke: {}", stroke));
    }
    if let Some(scale) = opts.scale {
        code.push_str(&format!(",\n    scale: {}", scale));
    }
    if let Some(ref pivot) = opts.pivot {
        code.push_str(&format!(",\n    pivot: {}", pivot));
    }

    code.push_str("\n  ),\n");
    code
}

fn generate_place_element(element: &PlotElement) -> String {
    let mut code = String::new();

    // place() requires x, y coordinates and text
    let x = element
        .x_data
        .as_ref()
        .and_then(|v| v.first())
        .unwrap_or(&0.0);
    let y = element
        .y_data
        .as_ref()
        .and_then(|v| v.first())
        .unwrap_or(&0.0);
    let text = element.options.text.as_deref().unwrap_or("");

    let opts = &element.options;

    // Wrap content with pad() if padding specified (like Lilaq: pad(0.2em)[#y])
    let content = if let Some(ref padding) = opts.padding {
        format!("pad({})[{}]", padding, text)
    } else {
        format!("[{}]", text)
    };

    code.push_str(&format!("  lq.place({}, {}, {}", x, y, content));

    if let Some(ref align) = opts.align {
        code.push_str(&format!(", align: {}", align));
    }

    code.push_str("),\n");
    code
}

fn generate_yaxis_element(element: &PlotElement) -> String {
    let mut code = String::new();

    let opts = &element.options;

    code.push_str("  lq.yaxis(\n");

    // Position (left or right)
    if let Some(ref pos) = opts.position {
        code.push_str(&format!("    position: {},\n", pos));
    }

    // Axis label
    if let Some(ref label) = opts.axis_label {
        code.push_str(&format!("    label: [{}],\n", label));
    }

    // Child elements (plots on this axis)
    if let Some(ref children) = opts.children {
        for child in children {
            code.push_str(&generate_element_code(child));
        }
    }

    code.push_str("  ),\n");
    code
}

fn generate_xaxis_element(element: &PlotElement) -> String {
    let mut code = String::new();

    let opts = &element.options;

    code.push_str("  lq.xaxis(\n");

    // Position (top or bottom)
    if let Some(ref pos) = opts.position {
        code.push_str(&format!("    position: {},\n", pos));
    }

    // Axis label
    if let Some(ref label) = opts.axis_label {
        code.push_str(&format!("    label: [{}],\n", label));
    }

    // Axis offset
    if let Some(offset) = opts.axis_offset {
        code.push_str(&format!("    offset: {},\n", offset));
    }

    // Exponent (0 = no scientific notation)
    if let Some(exp) = opts.exponent {
        code.push_str(&format!("    exponent: {},\n", exp));
    }

    // Tick distance
    if let Some(dist) = opts.tick_distance {
        code.push_str(&format!("    tick-distance: {},\n", dist));
    }

    // Transformation functions (forward and inverse)
    if let (Some(ref forward), Some(ref inverse)) =
        (&opts.transform_forward, &opts.transform_inverse)
    {
        code.push_str(&format!("    functions: ({}, {}),\n", forward, inverse));
    }

    // Child elements (plots on this axis)
    if let Some(ref children) = opts.children {
        for child in children {
            code.push_str(&generate_element_code(child));
        }
    }

    code.push_str("  ),\n");
    code
}

fn generate_path_element(element: &PlotElement) -> String {
    let mut code = String::new();

    // Path takes individual (x, y) coordinate pairs as variadic arguments
    // In Lilaq: lq.path(..points, fill: color, closed: true)
    // We output each point as a separate (x, y) argument
    code.push_str("  lq.path(\n");

    // Output each point as (x, y) - matching Lilaq's spread syntax
    if let (Some(ref x_data), Some(ref y_data)) = (&element.x_data, &element.y_data) {
        for (x, y) in x_data.iter().zip(y_data.iter()) {
            code.push_str(&format!("    ({}, {}),\n", x, y));
        }
    }

    let opts = &element.options;

    // Fill color
    if let Some(ref fill) = opts.fill {
        code.push_str(&format!("    fill: {},\n", fill));
    }

    // Stroke color
    if let Some(ref stroke) = opts.stroke {
        code.push_str(&format!("    stroke: {},\n", stroke));
    }

    // Closed path
    if let Some(closed) = opts.closed {
        code.push_str(&format!("    closed: {},\n", closed));
    }

    // Label for legend
    if let Some(ref label) = opts.label {
        code.push_str(&format!("    label: [{}],\n", label));
    }

    // Clip to data area
    if opts.clip == Some(false) {
        code.push_str("    clip: false,\n");
    }

    // Z-index
    if let Some(z) = opts.z_index {
        code.push_str(&format!("    z-index: {},\n", z));
    }

    code.push_str("  ),\n");
    code
}

/// Compile a diagram to SVG
pub fn compile_diagram(
    elements: &[PlotElement],
    options: &DiagramOptions,
) -> Result<PlotOutput, String> {
    let code = generate_diagram_code(elements, options);
    // Debug: print generated Typst code (set KLEIS_DEBUG_TYPST=1 to enable)
    if std::env::var("KLEIS_DEBUG_TYPST").is_ok() {
        eprintln!("=== Generated Typst Code ===\n{}\n===", code);
    }
    compile_to_svg(&code)
}

/// Export a diagram as Typst code (without compiling to SVG)
///
/// Returns the raw Typst/Lilaq code that can be embedded in a .typ document
/// or used with `typst compile` directly.
///
/// # Example output
/// ```typst
/// #import "@preview/lilaq:0.5.0" as lq
/// #set page(width: auto, height: auto, margin: 0.5em)
/// #lq.diagram(
///   lq.plot((0, 1, 2, 3), (1, 4, 9, 16)),
///   title: [Quadratic Growth],
/// )
/// ```
pub fn export_diagram_typst(elements: &[PlotElement], options: &DiagramOptions) -> String {
    generate_diagram_code(elements, options)
}

/// Export just the lq.diagram(...) call without preamble
///
/// Useful for embedding in an existing Typst document that already
/// has the Lilaq import.
pub fn export_diagram_typst_fragment(elements: &[PlotElement], options: &DiagramOptions) -> String {
    let full_code = generate_diagram_code(elements, options);

    // Find the start of lq.diagram and return from there
    if let Some(pos) = full_code.find("#lq.diagram(") {
        // Return from #lq.diagram onwards, removing the leading #
        full_code[pos + 1..].to_string()
    } else if let Some(pos) = full_code.find("lq.diagram(") {
        full_code[pos..].to_string()
    } else {
        // Fallback: return full code
        full_code
    }
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
