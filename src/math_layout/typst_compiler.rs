// Typst compiler integration
//
// This module compiles Typst markup to SVG and extracts placeholder positions.
// Uses Typst library API (not CLI) to get professional math layout with layout tree access.

use std::path::PathBuf;
use typst::diag::{FileResult, FileError};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{Source, FileId, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst::layout::{Frame, FrameItem, Point, Transform};

/// Minimal World implementation for Typst compilation
/// 
/// This provides file access, fonts, and library to the Typst compiler.
/// Based on TestWorld from Typst's test suite.
#[derive(Clone)]
struct MinimalWorld {
    library: LazyHash<Library>,
    font_book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main_source: Source,
}

impl MinimalWorld {
    fn new(source_text: &str) -> Self {
        // Load embedded fonts
        let fonts: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::from_static(data)))
            .collect();
        
        let font_book = FontBook::from_fonts(&fonts);
        
        // Create main source file ID
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        
        // Create main source
        let main_source = Source::new(main_id, source_text.to_string());
        
        Self {
            library: LazyHash::new(Library::default()),
            font_book: LazyHash::new(font_book),
            fonts,
            main_source,
        }
    }
}

impl World for MinimalWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    
    fn book(&self) -> &LazyHash<FontBook> {
        &self.font_book
    }
    
    fn main(&self) -> FileId {
        self.main_source.id()
    }
    
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_source.id() {
            Ok(self.main_source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
        }
    }
    
    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(PathBuf::from("file access not supported")))
    }
    
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        // Return a fixed date for reproducibility
        Some(Datetime::from_ymd(2024, 1, 1).unwrap())
    }
}

/// Compile with semantic bounding boxes using two-pass rendering
///
/// This function uses AST structure to create accurate bounding boxes for each argument.
/// It renders each argument separately, then matches boxes in the full rendering.
pub fn compile_with_semantic_boxes(
    ast: &crate::ast::Expression,
    placeholder_ids: &[usize]
) -> Result<CompiledOutput, String> {
    use crate::render::{build_default_context, render_expression, RenderTarget};
    use crate::ast::Expression;
    
    eprintln!("=== compile_with_semantic_boxes (Two-Pass Rendering) ===");
    
    let ctx = build_default_context();
    
    // Pass 1: Count boxes for each argument by rendering separately
    let box_counts = match ast {
        Expression::Operation { args, .. } => {
            let mut counts = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                let arg_markup = render_expression(arg, &ctx, &RenderTarget::Typst);
                eprintln!("  Arg {} markup: {}", i, arg_markup);
                
                // Compile arg separately and extract its layout boxes
                let arg_doc = format!(
                    r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ {} $)
"#,
                    arg_markup
                );
                
                let world = MinimalWorld::new(&arg_doc);
                match typst::compile(&world).output {
                    Ok(document) => {
                        if let Some(page) = document.pages.first() {
                            let mut arg_boxes = Vec::new();
                            extract_bounding_boxes_from_frame(&page.frame, Transform::identity(), &mut arg_boxes);
                            
                            // Count text boxes only
                            let text_count = arg_boxes.iter().filter(|b| b.content_type == "text").count();
                            eprintln!("  Arg {} produces {} text boxes", i, text_count);
                            counts.push(text_count);
                        } else {
                            counts.push(0);
                        }
                    }
                    Err(e) => {
                        eprintln!("  Warning: Could not compile arg {}: {:?}", i, e);
                        counts.push(0);
                    }
                }
            }
            counts
        }
        _ => vec![],  // Leaf nodes have no arguments
    };
    
    // Pass 2: Render full expression and assign boxes to arguments
    let full_markup = render_expression(ast, &ctx, &RenderTarget::Typst);
    eprintln!("Full markup: {}", full_markup);
    
    let mut output = compile_math_to_svg_with_ids(&full_markup, placeholder_ids)?;
    
    // Replace spatial grouping with semantic grouping based on box counts
    if !box_counts.is_empty() {
        output.argument_bounding_boxes = extract_semantic_argument_boxes(
            &output.svg,
            box_counts,
            ast
        )?;
    }
    
    Ok(output)
}

/// Count text boxes in SVG (helper for two-pass rendering)
fn count_text_boxes_in_svg(svg: &str) -> usize {
    // Count <text> elements in SVG
    svg.matches("<text").count()
}

/// Extract argument bounding boxes using semantic information from AST
fn extract_semantic_argument_boxes(
    _svg: &str,
    box_counts: Vec<usize>,
    ast: &crate::ast::Expression
) -> Result<Vec<ArgumentBoundingBox>, String> {
    eprintln!("Extracting semantic boxes with counts: {:?}", box_counts);
    
    // We need to extract layout boxes from the full rendering
    // and assign them to arguments based on box_counts
    
    // For now, compile the full expression again to get layout boxes
    use crate::render::{build_default_context, render_expression, RenderTarget};
    
    let ctx = build_default_context();
    let full_markup = render_expression(ast, &ctx, &RenderTarget::Typst);
    
    let full_doc = format!(
        r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ {} $)
"#,
        full_markup
    );
    
    let world = MinimalWorld::new(&full_doc);
    let document = typst::compile(&world).output
        .map_err(|e| format!("Compilation failed: {:?}", e))?;
    
    let page = document.pages.first()
        .ok_or("No pages in document")?;
    
    // Extract all layout boxes
    let mut all_boxes = Vec::new();
    extract_bounding_boxes_from_frame(&page.frame, Transform::identity(), &mut all_boxes);
    
    // Normalize coordinates
    if !all_boxes.is_empty() {
        let min_x = all_boxes.iter().map(|b| b.x).fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = all_boxes.iter().map(|b| b.y).fold(f64::INFINITY, |a, b| a.min(b));
        for bbox in &mut all_boxes {
            bbox.x -= min_x;
            bbox.y -= min_y;
        }
    }
    
    // Filter to text boxes only
    let text_boxes: Vec<&LayoutBoundingBox> = all_boxes.iter()
        .filter(|b| b.content_type == "text")
        .collect();
    
    eprintln!("Full expression has {} text boxes", text_boxes.len());
    
    // Assign boxes to arguments based on counts
    let mut result = Vec::new();
    let mut box_index = 0;
    
    for (arg_idx, &count) in box_counts.iter().enumerate() {
        if count == 0 {
            continue;  // Skip arguments with no boxes
        }
        
        if box_index + count > text_boxes.len() {
            eprintln!("Warning: Not enough boxes for arg {}", arg_idx);
            break;
        }
        
        // Get boxes for this argument
        let arg_boxes = &text_boxes[box_index..box_index + count];
        
        // Create bounding box encompassing all boxes for this argument
        let min_x = arg_boxes.iter().map(|b| b.x).fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = arg_boxes.iter().map(|b| b.y).fold(f64::INFINITY, |a, b| a.min(b));
        let max_x = arg_boxes.iter().map(|b| b.x + b.width).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let max_y = arg_boxes.iter().map(|b| b.y + b.height).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        
        let padding = 4.0;
        
        result.push(ArgumentBoundingBox {
            arg_index: arg_idx,
            node_id: format!("0.{}", arg_idx),
            x: min_x - padding,
            y: min_y - padding,
            width: (max_x - min_x) + padding * 2.0,
            height: (max_y - min_y) + padding * 2.0,
        });
        
        box_index += count;
    }
    
    eprintln!("Created {} semantic argument boxes", result.len());
    
    Ok(result)
}

/// Compile Typst math markup to SVG with placeholder tracking (with known IDs)
///
/// Uses Typst library API to compile math to professional SVG with layout tree access
pub fn compile_math_to_svg_with_ids(markup: &str, placeholder_ids: &[usize]) -> Result<CompiledOutput, String> {
    eprintln!("=== compile_math_to_svg_with_ids called (Library API) ===");
    eprintln!("Input markup: {}", markup);
    eprintln!("Expected placeholder IDs: {:?}", placeholder_ids);
    
    let expected_placeholders = placeholder_ids.len();
    eprintln!("Expected {} placeholders", expected_placeholders);
    
    // Create Typst document with math mode
    // Use 0pt margin and top-left alignment to ensure coordinates are deterministic
    // We use a box around the math to prevent block-level centering
    let typst_doc = format!(
        r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ {} $)
"#,
        markup
    );
    
    eprintln!("Creating Typst world...");
    let world = MinimalWorld::new(&typst_doc);
    
    // Compile to document - typst::compile returns Warned<Result<Document, ...>>
    eprintln!("Compiling with Typst library...");
    let result = typst::compile(&world);
    
    // Extract document from Warned result
    let document = match result.output {
        Ok(doc) => doc,
        Err(errors) => {
            let error_msgs: Vec<String> = errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect();
            eprintln!("Typst compilation errors: {:?}", error_msgs);
            return Err(format!("Typst compilation failed: {}", error_msgs.join("; ")));
        }
    };
    
    eprintln!("Typst compilation successful!");
    eprintln!("Document has {} page(s)", document.pages.len());
    
    // Get the first page
    let page = &document.pages[0];
    let frame = &page.frame;
    
    eprintln!("Page size: {:?}x{:?}", frame.width(), frame.height());
    
    // Extract bounding boxes from layout
    let mut all_boxes = Vec::new();
    extract_bounding_boxes_from_frame(frame, typst::layout::Transform::identity(), &mut all_boxes);
    eprintln!("Extracted {} bounding boxes from layout tree", all_boxes.len());
    
    // Normalize coordinates: typst-svg shifts content so min_x/min_y is at 0 (plus padding)
    // We need to replicate this shift to match SVG coordinates
    if !all_boxes.is_empty() {
        let min_x = all_boxes.iter().map(|b| b.x).fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = all_boxes.iter().map(|b| b.y).fold(f64::INFINITY, |a, b| a.min(b));
        
        eprintln!("Layout bounds: min_x={:.2}, min_y={:.2}", min_x, min_y);
        
        // Apply shift to align with SVG (which starts at 0,0)
        // SVG likely adds a small padding even with margin:0pt, or maybe 0
        // Let's shift so min becomes 0
        for bbox in &mut all_boxes {
            bbox.x -= min_x;
            bbox.y -= min_y;
        }
    }
    
    // Convert page to SVG (not document - typst_svg::svg takes a Page)
    let svg = typst_svg::svg(page);
    eprintln!("Generated SVG length: {}", svg.len());
    
    // Extract placeholder positions (find square symbols, match by order with correct IDs)
    let placeholder_positions = extract_placeholder_positions_by_symbol(&svg, placeholder_ids)?;
    eprintln!("Extracted {} placeholder positions", placeholder_positions.len());
    
    // CALIBRATE COORDINATES
    // Calculate offset between Layout coordinates and SVG coordinates using the first placeholder
    let mut offset_x = 0.0;
    let mut offset_y = 0.0;
    
    if let Some(first_ph) = placeholder_positions.first() {
        // Find corresponding box in layout tree (Text element with similar size/position relative to others)
        // The square symbol in Typst is a text glyph
        // We look for a text box with width ~18pt
        
        // Find text boxes with width between 10 and 25
        let candidates: Vec<&LayoutBoundingBox> = all_boxes.iter()
            .filter(|b| b.content_type == "text" && b.width > 10.0 && b.width < 25.0)
            .collect();
            
        if let Some(match_box) = candidates.first() {
            // Calculate offset
            // SVG = Layout + Offset
            // Offset = SVG - Layout
            offset_x = first_ph.x - match_box.x;
            offset_y = first_ph.y - match_box.y;
            
            // Y-coordinate might be inverted or shifted differently
            // But let's try simple translation first
            eprintln!("Calibrated offset: ({:.2}, {:.2}) using placeholder ID {}", 
                     offset_x, offset_y, first_ph.id);
        }
    }
    
    // Apply offset to all layout boxes
    let calibrated_boxes: Vec<LayoutBoundingBox> = all_boxes.iter().map(|b| LayoutBoundingBox {
        x: b.x + offset_x,
        y: b.y + offset_y,
        width: b.width,
        height: b.height,
        content_type: b.content_type.clone(),
    }).collect();
    
    // Extract argument bounding boxes by grouping content boxes from layout tree
    // Use the CALIBRATED boxes
    let argument_bounding_boxes = group_content_into_arguments(&svg, &calibrated_boxes, &placeholder_positions)?;
    eprintln!("Extracted {} argument bounding boxes", argument_bounding_boxes.len());
    
    Ok(CompiledOutput {
        svg,
        placeholder_positions,
        argument_bounding_boxes,
    })
}


/// Extract bounding boxes from Typst layout frame (recursive)
///
/// Traverses the layout tree and collects bounding boxes for all items.
/// Tracks the accumulated transform matrix to give absolute page coordinates.
fn extract_bounding_boxes_from_frame(
    frame: &Frame,
    ts: Transform,
    boxes: &mut Vec<LayoutBoundingBox>
) {
    for (pos, item) in frame.items() {
        // Apply item position to current transform
        // Transform::pre_concat applies the transformation *before* the current one
        // But here we want to translate the coordinate system origin
        let item_ts = ts.pre_concat(Transform::translate(pos.x, pos.y));
        
        match item {
            FrameItem::Group(group) => {
                // Apply group's transform
                let group_ts = item_ts.pre_concat(group.transform);
                extract_bounding_boxes_from_frame(&group.frame, group_ts, boxes);
            }
            FrameItem::Text(text) => {
                // Text element - calculate bounding box from glyphs
                let glyphs = &text.glyphs;
                let mut width = 0.0;
                for glyph in glyphs {
                    // x_advance is Em, need to convert to points
                    // Em is relative to font size, so multiply by size
                    width += glyph.x_advance.at(text.size).to_pt();
                }
                
                // Height is the font size
                let height = text.size.to_pt();
                
                // Transform the top-left (0,0) point to get absolute bounding box
                let tl = Point::zero().transform(item_ts);
                
                boxes.push(LayoutBoundingBox {
                    x: tl.x.to_pt(),
                    y: tl.y.to_pt(),
                    width,
                    height,
                    content_type: "text".to_string(),
                });
            }
            FrameItem::Shape(shape, _) => {
                // Shape element - get geometry size
                let bbox_size = shape.geometry.bbox_size();
                let tl = Point::zero().transform(item_ts);
                
                boxes.push(LayoutBoundingBox {
                    x: tl.x.to_pt(),
                    y: tl.y.to_pt(),
                    width: bbox_size.x.to_pt(),
                    height: bbox_size.y.to_pt(),
                    content_type: "shape".to_string(),
                });
            }
            FrameItem::Image(_, size, _) => {
                // Image element
                let tl = Point::zero().transform(item_ts);
                
                boxes.push(LayoutBoundingBox {
                    x: tl.x.to_pt(),
                    y: tl.y.to_pt(),
                    width: size.x.to_pt(),
                    height: size.y.to_pt(),
                    content_type: "image".to_string(),
                });
            }
            FrameItem::Link(_, size) => {
                // Link element (hyperlinks, etc.) - record bounding box
                let tl = Point::zero().transform(item_ts);
                
                boxes.push(LayoutBoundingBox {
                    x: tl.x.to_pt(),
                    y: tl.y.to_pt(),
                    width: size.x.to_pt(),
                    height: size.y.to_pt(),
                    content_type: "link".to_string(),
                });
            }
            FrameItem::Tag(_) => {
                // Tag element (metadata) - skip, no visual representation
            }
        }
    }
}

/// Legacy function (for backward compatibility)
pub fn compile_math_to_svg(markup: &str) -> Result<CompiledOutput, String> {
    // Extract IDs by counting (fallback)
    let count = markup.matches("square.stroked").count();
    let ids: Vec<usize> = (0..count).collect();
    compile_math_to_svg_with_ids(markup, &ids)
}

/// Generate mock SVG for testing (temporary)
/// 
/// This creates a realistic-looking fraction layout with placeholder markers
/// positioned where Typst would actually place them.
fn generate_mock_svg(markup: &str) -> Result<String, String> {
    eprintln!("Generating mock SVG for: {}", markup);
    
    // Check if it's a fraction pattern
    if markup.contains("/(") {
        // Fraction: render numerator above denominator with fraction bar
        return generate_fraction_mock_svg(markup);
    }
    
    // Default: simple horizontal layout
    let mut svg_elements = Vec::new();
    
    svg_elements.push(format!(
        r#"<text x="100" y="100" font-family="Latin Modern Math, Times New Roman, serif" font-size="32" fill="black">{}</text>"#,
        escape_xml(markup)
    ));
    
    svg_elements.push(
        r#"<text x="20" y="180" font-size="14" fill="gray">Mock rendering - Typst library integration pending</text>"#.to_string()
    );
    
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"600\" height=\"200\" viewBox=\"0 0 600 200\">\n  <rect width=\"600\" height=\"200\" fill=\"#fafafa\"/>\n  {}\n</svg>",
        svg_elements.join("\n  ")
    ))
}

/// Generate mock SVG specifically for fractions with realistic positioning
fn generate_fraction_mock_svg(markup: &str) -> Result<String, String> {
    // Parse the fraction: (numerator)/(denominator)
    let parts: Vec<&str> = markup.split(")/").collect();
    if parts.len() != 2 {
        return generate_mock_svg(markup);
    }
    
    let numerator = parts[0].trim_start_matches('(');
    let denominator = parts[1].trim_end_matches(')');
    
    eprintln!("Fraction: num='{}', den='{}'", numerator, denominator);
    
    // Realistic fraction positions
    let center_x = 150.0;
    let numerator_y = 60.0;
    let bar_y = 90.0;
    let denominator_y = 120.0;
    let bar_width = 100.0;
    
    let mut svg_elements = Vec::new();
    
    // Numerator (top)
    svg_elements.push(format!(
        r#"<text x="{}" y="{}" font-family="Latin Modern Math, Times New Roman, serif" font-size="28" fill="black" text-anchor="middle">{}</text>"#,
        center_x, numerator_y, escape_xml(numerator)
    ));
    
    // Fraction bar
    svg_elements.push(format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"/>"#,
        center_x - bar_width/2.0, bar_y, center_x + bar_width/2.0, bar_y
    ));
    
    // Denominator (bottom)
    svg_elements.push(format!(
        r#"<text x="{}" y="{}" font-family="Latin Modern Math, Times New Roman, serif" font-size="28" fill="black" text-anchor="middle">{}</text>"#,
        center_x, denominator_y, escape_xml(denominator)
    ));
    
    // Annotation
    svg_elements.push(
        r#"<text x="20" y="180" font-size="12" fill="gray">Mock fraction layout - realistic positioning</text>"#.to_string()
    );
    
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"200\" viewBox=\"0 0 400 200\">\n  <rect width=\"400\" height=\"200\" fill=\"#fafafa\"/>\n  {}\n</svg>",
        svg_elements.join("\n  ")
    ))
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Output from Typst compilation
#[derive(Debug, Clone)]
pub struct CompiledOutput {
    /// SVG string with beautiful math rendering
    pub svg: String,
    
    /// Positions of placeholders in the SVG
    pub placeholder_positions: Vec<PlaceholderPosition>,
    
    /// Bounding boxes for each argument (extracted from invisible markers)
    pub argument_bounding_boxes: Vec<ArgumentBoundingBox>,
}

/// Bounding box extracted from Typst layout tree
#[derive(Debug, Clone)]
pub struct LayoutBoundingBox {
    /// X position in SVG coordinates (pt)
    pub x: f64,
    
    /// Y position in SVG coordinates (pt)
    pub y: f64,
    
    /// Width (pt)
    pub width: f64,
    
    /// Height (pt)
    pub height: f64,
    
    /// Type of content (text, shape, etc.)
    pub content_type: String,
}

/// Bounding box for a specific argument (extracted from invisible markers)
#[derive(Debug, Clone)]
pub struct ArgumentBoundingBox {
    /// Argument index (0, 1, 2, etc.)
    pub arg_index: usize,
    
    /// Unique node ID in the AST (e.g., "0.1.2" for path through tree)
    pub node_id: String,
    
    /// X position in SVG coordinates (pt)
    pub x: f64,
    
    /// Y position in SVG coordinates (pt)
    pub y: f64,
    
    /// Width (pt)
    pub width: f64,
    
    /// Height (pt)
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct PlaceholderPosition {
    /// Placeholder ID (from marker ⟨⟨PH{id}⟩⟩)
    pub id: usize,
    
    /// X position in SVG coordinates
    pub x: f64,
    
    /// Y position in SVG coordinates
    pub y: f64,
    
    /// Width of the text element containing the marker
    pub width: f64,
    
    /// Height of the text element
    pub height: f64,
}

/// Extract placeholder positions by finding square symbols in SVG
///
/// Typst renders square.stroked as SVG <use> elements with transform matrices.
/// We find these and extract their positions, using the provided IDs.
fn extract_placeholder_positions_by_symbol(svg: &str, placeholder_ids: &[usize]) -> Result<Vec<PlaceholderPosition>, String> {
    let expected_count = placeholder_ids.len();
    eprintln!("Extracting {} placeholders by finding square symbols in Typst SVG", expected_count);
    eprintln!("Using placeholder IDs: {:?}", placeholder_ids);
    
    let mut positions = Vec::new();
    
    // Typst library uses: <g transform="translate(X Y)">
    //                        <g class="typst-text" transform="scale(1, -1)">
    //                          <use xlink:href="#gXXX" x="0"/>
    //                        </g>
    //                      </g>
    
    // Pattern to find translate transforms with nested use elements
    // Pattern: <g transform="translate(X Y)"> ... <use xlink:href="#gID"/>
    let pattern_str = r###"<g[^>]*transform="translate\(([\d.]+) ([\d.]+)\)"[^>]*>[\s\S]*?<use[^>]*xlink:href="#g([A-F0-9]+)""###;
    let transform_pattern = regex::Regex::new(pattern_str)
        .map_err(|e| format!("Regex error: {}", e))?;
    
    // First, identify which glyph ID is the square
    // It should appear exactly as many times as expected_count
    let mut glyph_counts: std::collections::HashMap<String, Vec<(f64, f64)>> = std::collections::HashMap::new();
    
    for cap in transform_pattern.captures_iter(svg) {
        if let (Some(x_str), Some(y_str), Some(glyph_id)) = (cap.get(1), cap.get(2), cap.get(3)) {
            if let (Ok(x), Ok(y)) = (x_str.as_str().parse::<f64>(), y_str.as_str().parse::<f64>()) {
                glyph_counts.entry(glyph_id.as_str().to_string())
                    .or_insert_with(Vec::new)
                    .push((x, y));
            }
        }
    }
    
    eprintln!("Found {} unique glyphs", glyph_counts.len());
    for (glyph, positions_vec) in &glyph_counts {
        eprintln!("  Glyph #{}: {} occurrences", glyph, positions_vec.len());
    }
    
    // Find the glyph that appears exactly expected_count times (likely the square)
    let square_positions = glyph_counts.iter()
        .find(|(_, positions_vec)| positions_vec.len() == expected_count)
        .map(|(_, positions_vec)| positions_vec);
    
    if let Some(square_pos) = square_positions {
        eprintln!("Identified square glyph with {} instances", square_pos.len());
        for (i, (x, y)) in square_pos.iter().enumerate() {
            // Use the actual placeholder ID from the AST, not just the index
            let placeholder_id = placeholder_ids.get(i).copied().unwrap_or(i);
            eprintln!("  Square {} (ID {}): position ({}, {})", i, placeholder_id, x, y);
            positions.push(PlaceholderPosition {
                id: placeholder_id,  // Use actual ID from AST!
                x: *x,
                y: *y,
                width: 18.0,  // Approximate from Typst square
                height: 18.0,
            });
        }
    } else {
        eprintln!("Warning: Could not identify square glyph");
    }
    
    eprintln!("Total placeholders extracted: {}", positions.len());
    
    Ok(positions)
}

/// Group content bounding boxes into argument-level boxes
///
/// Uses the layout boxes extracted from the Typst frame (absolute coordinates).
/// Groups by y-position to separate lines (e.g. numerator vs denominator).
fn group_content_into_arguments(
    _svg: &str,
    layout_boxes: &[LayoutBoundingBox],
    _placeholder_positions: &[PlaceholderPosition],
) -> Result<Vec<ArgumentBoundingBox>, String> {
    eprintln!("Grouping {} layout boxes into arguments...", layout_boxes.len());
    
    let mut argument_boxes = Vec::new();
    
    // Filter for text content only
    let content_boxes: Vec<&LayoutBoundingBox> = layout_boxes.iter()
        .filter(|b| b.content_type == "text")
        .collect();
    
    // Sort by Y position first
    let mut sorted_boxes = content_boxes.clone();
    sorted_boxes.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
    
    // Group by Y-position (lines)
    let mut lines: Vec<Vec<&LayoutBoundingBox>> = Vec::new();
    
    for bbox in sorted_boxes {
        // Find if this box belongs to an existing line
        let mut placed = false;
        for (line_idx, line) in lines.iter_mut().enumerate() {
            if let Some(first) = line.first() {
                // If Y centers are close, it's the same line
                // Tolerance increased to 20.0 to handle nested fractions (e.g. 3/x in denominator)
                // which might have components shifted vertically
                let center_y = bbox.y + bbox.height/2.0;
                let line_y = first.y + first.height/2.0;
                let diff = (center_y - line_y).abs();
                
                if diff < 20.0 {
                    line.push(bbox);
                    placed = true;
                    break;
                }
            }
        }
        if !placed {
            lines.push(vec![bbox]);
        }
    }
    
    // Create a bounding box for each line (argument)
    for (index, line) in lines.iter().enumerate() {
        if line.is_empty() { continue; }
        
        let min_x = line.iter().map(|b| b.x).fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = line.iter().map(|b| b.y).fold(f64::INFINITY, |a, b| a.min(b));
        let max_x = line.iter().map(|b| b.x + b.width).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let max_y = line.iter().map(|b| b.y + b.height).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        
        let width = (max_x - min_x).max(20.0);
        let height = (max_y - min_y).max(20.0);
        
        // Add padding
        let padding = 4.0;
        
        eprintln!("  Line {}: bbox ({:.1}, {:.1}) size {:.1}x{:.1}", 
                 index, min_x, min_y, width, height);
        
        argument_boxes.push(ArgumentBoundingBox {
            arg_index: index,
            node_id: format!("0.{}", index),  // Generate node ID from index
            x: min_x - padding,
            y: min_y - padding,
            width: width + padding * 2.0,
            height: height + padding * 2.0,
        });
    }
    
    Ok(argument_boxes)
}

/// Extract argument bounding boxes from invisible markers (OLD APPROACH - not used)
///
/// Looks for white-filled text groups (our invisible markers) and pairs them up
/// to create bounding boxes for each argument.
fn extract_argument_bounding_boxes_markers(svg: &str) -> Result<Vec<ArgumentBoundingBox>, String> {
    eprintln!("Extracting argument bounding boxes from invisible markers...");
    
    let mut argument_boxes = Vec::new();
    
    // Pattern to find translate transforms with white-filled text (our markers)
    // The markers are rendered with fill="#ffffff"
    let pattern_str = r###"<g[^>]*transform="translate\(([\d.]+) ([\d.]+)\)"[^>]*>[\s\S]*?fill="#ffffff""###;
    let transform_pattern = regex::Regex::new(pattern_str)
        .map_err(|e| format!("Regex error: {}", e))?;
    
    // Collect all white-filled text positions (these are our markers)
    let mut marker_positions: Vec<(f64, f64)> = Vec::new();
    
    for cap in transform_pattern.captures_iter(svg) {
        if let (Some(x_str), Some(y_str)) = (cap.get(1), cap.get(2)) {
            if let (Ok(x), Ok(y)) = (x_str.as_str().parse::<f64>(), y_str.as_str().parse::<f64>()) {
                marker_positions.push((x, y));
                eprintln!("  Found white marker at ({:.1}, {:.1})", x, y);
            }
        }
    }
    
    eprintln!("  Total markers found: {}", marker_positions.len());
    
    // Markers come in pairs: start and end for each argument
    // For N arguments, we expect 2*N markers
    // Pair them up: (0,1), (2,3), (4,5), etc.
    let mut arg_index = 0;
    let mut i = 0;
    
    while i + 1 < marker_positions.len() {
        let (start_x, start_y) = marker_positions[i];
        let (end_x, end_y) = marker_positions[i + 1];
        
        // Create bounding box from start to end marker
        let x = start_x.min(end_x);
        let y = start_y.min(end_y);
        let max_x = start_x.max(end_x);
        let max_y = start_y.max(end_y);
        let width = (max_x - x).max(20.0);  // Minimum width
        let height = (max_y - y).max(20.0);  // Minimum height
        
        eprintln!("  Arg {}: bbox ({:.1}, {:.1}) size {:.1}x{:.1}", arg_index, x, y, width, height);
        
        argument_boxes.push(ArgumentBoundingBox {
            arg_index,
            node_id: format!("0.{}", arg_index),  // Generate node ID from index
            x,
            y,
            width,
            height,
        });
        
        arg_index += 1;
        i += 2;  // Move to next pair
    }
    
    eprintln!("  Created {} argument bounding boxes", argument_boxes.len());
    
    Ok(argument_boxes)
}

/// Extract bounding boxes of colored argument boxes from Typst SVG
/// 
/// Each argument is wrapped in #box(fill: rgb(...)) which Typst renders
/// as SVG rectangles. We find these and extract their positions.
fn extract_colored_boxes(svg: &str) -> Result<Vec<BoundingBox>, String> {
    let mut boxes = Vec::new();
    
    // Pattern: <path fill="#rrggbb" d="M x y h width v height ..."/>
    // Or: <rect fill="#rrggbb" x="..." y="..." width="..." height="..."/>
    
    let rect_pattern = regex::Regex::new(r#"<rect[^>]*fill="[^"]*"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"#)
        .map_err(|e| format!("Regex error: {}", e))?;
    
    for cap in rect_pattern.captures_iter(svg) {
        if let (Some(x_str), Some(y_str), Some(w_str), Some(h_str)) = 
            (cap.get(1), cap.get(2), cap.get(3), cap.get(4)) {
            if let (Ok(x), Ok(y), Ok(w), Ok(h)) = 
                (x_str.as_str().parse::<f64>(), y_str.as_str().parse::<f64>(),
                 w_str.as_str().parse::<f64>(), h_str.as_str().parse::<f64>()) {
                
                eprintln!("Found colored box at ({}, {}) size {}x{}", x, y, w, h);
                
                boxes.push(BoundingBox {
                    x,
                    y,
                    width: w,
                    height: h,
                });
            }
        }
    }
    
    Ok(boxes)
}

#[derive(Debug, Clone)]
struct BoundingBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Extract placeholder positions from SVG text (legacy method)
///
/// Searches for marker patterns like ⟨⟨PH0⟩⟩ in the SVG and extracts
/// their positions from the parent <text> element attributes.
fn extract_placeholder_positions(svg: &str) -> Result<Vec<PlaceholderPosition>, String> {
    let mut positions = Vec::new();
    
    // Debug: Print SVG snippet
    eprintln!("Extracting placeholders from SVG (length: {})", svg.len());
    eprintln!("SVG snippet: {}", &svg[..svg.len().min(300)]);
    
    // Parse SVG to find text elements containing placeholder markers
    // Pattern: <text x="..." y="...">...⟨⟨PH{id}⟩⟩...</text>
    
    // Try to find the markers
    let marker_pattern = regex::Regex::new(r"⟨⟨PH(\d+)⟩⟩")
        .map_err(|e| format!("Regex error: {}", e))?;
    
    eprintln!("Searching for pattern: ⟨⟨PH(\\d+)⟩⟩");
    
    let matches: Vec<_> = marker_pattern.captures_iter(svg).collect();
    eprintln!("Found {} marker matches", matches.len());
    
    for cap in matches {
        if let Some(id_match) = cap.get(1) {
            let id: usize = id_match.as_str().parse()
                .map_err(|e| format!("Failed to parse placeholder ID: {}", e))?;
            
            eprintln!("Found placeholder marker: ID={}", id);
            
            // Find the containing <text> element
            let marker_pos = cap.get(0).unwrap().start();
            if let Some(position) = find_text_element_position(svg, marker_pos) {
                eprintln!("  Position: ({}, {})", position.0, position.1);
                positions.push(PlaceholderPosition {
                    id,
                    x: position.0,
                    y: position.1,
                    width: 30.0,   // Default width - will be refined
                    height: 20.0,  // Default height - will be refined
                });
            } else {
                eprintln!("  Warning: Could not find <text> element position");
            }
        }
    }
    
    eprintln!("Total placeholders extracted: {}", positions.len());
    
    Ok(positions)
}

/// Find the position attributes of the <text> element containing a marker
fn find_text_element_position(svg: &str, marker_pos: usize) -> Option<(f64, f64)> {
    // Find the <text> tag before this position
    eprintln!("  Looking for <text> tag before position {}", marker_pos);
    let text_start = svg[..marker_pos].rfind("<text")?;
    eprintln!("  Found <text> at position {}", text_start);
    
    let text_end = svg[text_start..].find('>')?;
    let text_tag = &svg[text_start..text_start + text_end];
    eprintln!("  Text tag: {}", text_tag);
    
    // Extract x and y attributes
    let x_str = extract_attribute(text_tag, "x")?;
    eprintln!("  x attribute: {}", x_str);
    let y_str = extract_attribute(text_tag, "y")?;
    eprintln!("  y attribute: {}", y_str);
    
    let x = x_str.parse().ok()?;
    let y = y_str.parse().ok()?;
    
    eprintln!("  Parsed position: ({}, {})", x, y);
    
    Some((x, y))
}

/// Extract attribute value from XML tag
fn extract_attribute<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    // Pattern: attr="value"
    let pattern = format!("{}=\"", attr);
    let start_pos = tag.find(&pattern)? + pattern.len();
    let remaining = &tag[start_pos..];
    let end_pos = remaining.find('"')?;
    Some(&remaining[..end_pos])
}

// TODO: Implement MinimalWorld and Typst library integration
// For now using mock SVG to test the pipeline end-to-end

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_attribute() {
        let tag = r#"<text x="42.5" y="100" font-size="16">"#;
        assert_eq!(extract_attribute(tag, "x"), Some("42.5"));
        assert_eq!(extract_attribute(tag, "y"), Some("100"));
        assert_eq!(extract_attribute(tag, "font-size"), Some("16"));
        assert_eq!(extract_attribute(tag, "missing"), None);
    }
    
    #[test]
    #[ignore] // Requires full Typst setup
    fn test_compile_simple_fraction() {
        let markup = "(x)/(2)";
        let result = compile_math_to_svg(markup);
        
        // Should compile successfully
        assert!(result.is_ok());
        
        let output = result.unwrap();
        // Should contain SVG
        assert!(output.svg.contains("<svg"));
    }
    
    #[test]
    #[ignore] // Requires full Typst setup
    fn test_compile_with_placeholder() {
        let markup = "(⟨⟨PH0⟩⟩)/(2)";
        let result = compile_math_to_svg(markup);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        
        // Should find placeholder marker
        assert_eq!(output.placeholder_positions.len(), 1);
        assert_eq!(output.placeholder_positions[0].id, 0);
    }
}

