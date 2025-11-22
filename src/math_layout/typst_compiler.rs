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
use typst::layout::{Frame, FrameItem, Point};

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
    let typst_doc = format!(
        r#"#set page(width: auto, height: auto, margin: 10pt)
#set text(size: 24pt)

$ {} $
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
    extract_bounding_boxes_from_frame(frame, Point::zero(), &mut all_boxes);
    eprintln!("Extracted {} bounding boxes from layout tree", all_boxes.len());
    
    // Convert page to SVG (not document - typst_svg::svg takes a Page)
    let svg = typst_svg::svg(page);
    eprintln!("Generated SVG length: {}", svg.len());
    
    // Extract placeholder positions (find square symbols, match by order with correct IDs)
    let placeholder_positions = extract_placeholder_positions_by_symbol(&svg, placeholder_ids)?;
    eprintln!("Extracted {} placeholder positions", placeholder_positions.len());
    
    // Map bounding boxes to argument slots
    // For now, return placeholder positions (will enhance with full bounding boxes)
    
    Ok(CompiledOutput {
        svg,
        placeholder_positions,
        argument_bounding_boxes: all_boxes,
    })
}

/// Extract bounding boxes from Typst layout frame (recursive)
///
/// Traverses the layout tree and collects bounding boxes for all items.
fn extract_bounding_boxes_from_frame(
    frame: &Frame,
    offset: Point,
    boxes: &mut Vec<LayoutBoundingBox>
) {
    for (pos, item) in frame.items() {
        let item_pos = offset + *pos;
        
        match item {
            FrameItem::Group(group) => {
                // Recursively process nested frames
                extract_bounding_boxes_from_frame(&group.frame, item_pos, boxes);
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
                
                boxes.push(LayoutBoundingBox {
                    x: item_pos.x.to_pt(),
                    y: item_pos.y.to_pt(),
                    width,
                    height,
                    content_type: "text".to_string(),
                });
            }
            FrameItem::Shape(shape, _) => {
                // Shape element - get geometry size
                let bbox_size = shape.geometry.bbox_size();
                boxes.push(LayoutBoundingBox {
                    x: item_pos.x.to_pt(),
                    y: item_pos.y.to_pt(),
                    width: bbox_size.x.to_pt(),
                    height: bbox_size.y.to_pt(),
                    content_type: "shape".to_string(),
                });
            }
            FrameItem::Image(_, size, _) => {
                // Image element
                boxes.push(LayoutBoundingBox {
                    x: item_pos.x.to_pt(),
                    y: item_pos.y.to_pt(),
                    width: size.x.to_pt(),
                    height: size.y.to_pt(),
                    content_type: "image".to_string(),
                });
            }
            FrameItem::Link(_, size) => {
                // Link element (hyperlinks, etc.) - record bounding box
                boxes.push(LayoutBoundingBox {
                    x: item_pos.x.to_pt(),
                    y: item_pos.y.to_pt(),
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
    
    /// All bounding boxes extracted from layout tree
    pub argument_bounding_boxes: Vec<LayoutBoundingBox>,
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

