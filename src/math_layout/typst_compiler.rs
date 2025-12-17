// Typst compiler integration
//
// This module compiles Typst markup to SVG and extracts placeholder positions.
// Uses Typst library API (not CLI) to get professional math layout with layout tree access.

use std::collections::HashMap;
use std::path::PathBuf;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::layout::{Frame, FrameItem, Point, Transform};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

use crate::ast::Expression;
use crate::editor_ast::EditorNode;
use crate::render::{
    build_default_context, render_editor_node_with_uuids, render_expression,
    render_expression_with_ids, GlyphContext, RenderTarget,
};

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
            Err(FileError::NotFound(
                id.vpath().as_rootless_path().to_path_buf(),
            ))
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(PathBuf::from(
            "file access not supported",
        )))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        // Return a fixed date for reproducibility
        Some(Datetime::from_ymd(2024, 1, 1).unwrap())
    }
}

// FEATURE FLAG: Enable/disable calibration
// Set to false to skip calibration and use layout boxes as-is
// Set to true to use calibration (current behavior)
const USE_CALIBRATION: bool = false;

/// Compile with semantic bounding boxes using two-pass rendering
///
/// This function uses AST structure to create accurate bounding boxes for each argument.
/// It renders each argument separately, then matches boxes in the full rendering.
pub fn compile_with_semantic_boxes(
    ast: &Expression,
    placeholder_ids: &[usize],
    all_slot_ids: &[usize],
) -> Result<CompiledOutput, String> {
    compile_with_semantic_boxes_and_slots(
        ast,
        placeholder_ids,
        all_slot_ids,
        &std::collections::HashMap::new(),
    )
}

/// Compile with UUID-based slot tracking
pub fn compile_with_semantic_boxes_and_slots(
    ast: &Expression,
    placeholder_ids: &[usize],
    all_slot_ids: &[usize],
    node_id_to_uuid: &std::collections::HashMap<String, String>,
) -> Result<CompiledOutput, String> {
    eprintln!("=== compile_with_semantic_boxes (Two-Pass Rendering) ===");
    eprintln!("USE_CALIBRATION = {}", USE_CALIBRATION);
    eprintln!("placeholder_ids: {:?}", placeholder_ids);
    eprintln!("all_slot_ids: {:?}", all_slot_ids);
    eprintln!("UUID map entries: {}", node_id_to_uuid.len());

    let ctx = build_default_context();
    let full_markup = if node_id_to_uuid.is_empty() {
        render_expression(ast, &ctx, &RenderTarget::Typst)
    } else {
        render_expression_with_ids(ast, &ctx, &RenderTarget::Typst, node_id_to_uuid)
    };
    eprintln!("Full markup: {}", full_markup);

    let mut output = compile_math_to_svg_with_ids(&full_markup, placeholder_ids, all_slot_ids)?;

    // Extract ALL labeled positions from SVG (both placeholders and filled slots)
    let labeled_positions = extract_positions_from_labels(&output.svg)?;
    eprintln!(
        "Extracted {} labeled positions for semantic matching",
        labeled_positions.len()
    );

    // Also extract UUID-based labels (id{uuid}) for filled slots
    let uuid_positions = extract_uuid_positions(&output.svg)?;
    eprintln!("Extracted {} UUID-based positions", uuid_positions.len());

    output.argument_bounding_boxes = extract_semantic_argument_boxes(
        ast,
        &ctx,
        &full_markup,
        &labeled_positions,
        node_id_to_uuid,
        &uuid_positions,
    )?;

    Ok(output)
}

/// Compile EditorNode with UUID-based slot tracking (parallel to compile_with_semantic_boxes_and_slots)
pub fn compile_editor_node_with_semantic_boxes(
    node: &EditorNode,
    placeholder_ids: &[usize],
    all_slot_ids: &[usize],
    node_id_to_uuid: &std::collections::HashMap<String, String>,
) -> Result<CompiledOutput, String> {
    eprintln!("=== compile_editor_node_with_semantic_boxes ===");
    eprintln!("placeholder_ids: {:?}", placeholder_ids);
    eprintln!("all_slot_ids: {:?}", all_slot_ids);
    eprintln!("UUID map entries: {}", node_id_to_uuid.len());

    let ctx = build_default_context();
    let full_markup =
        render_editor_node_with_uuids(node, &ctx, &RenderTarget::Typst, node_id_to_uuid);
    eprintln!("Full markup: {}", full_markup);

    let mut output = compile_math_to_svg_with_ids(&full_markup, placeholder_ids, all_slot_ids)?;

    // Extract labeled positions from SVG
    let labeled_positions = extract_positions_from_labels(&output.svg)?;
    eprintln!("Extracted {} labeled positions", labeled_positions.len());

    // Extract UUID-based labels for filled slots
    let uuid_positions = extract_uuid_positions(&output.svg)?;
    eprintln!("Extracted {} UUID-based positions", uuid_positions.len());

    // For EditorNode, we use a simpler extraction approach
    output.argument_bounding_boxes = extract_semantic_argument_boxes_from_editor_node(
        node,
        &ctx,
        &labeled_positions,
        node_id_to_uuid,
        &uuid_positions,
    )?;

    Ok(output)
}

/// Extract argument boxes from EditorNode (simpler than Expression version)
fn extract_semantic_argument_boxes_from_editor_node(
    node: &EditorNode,
    _ctx: &GlyphContext,
    labeled_positions: &[PlaceholderPosition],
    node_id_to_uuid: &std::collections::HashMap<String, String>,
    uuid_positions: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
) -> Result<Vec<ArgumentBoundingBox>, String> {
    let mut boxes = Vec::new();
    let mut placeholder_idx = 0;

    // Build UUID->Position lookup
    let uuid_to_position: std::collections::HashMap<_, _> = uuid_positions.clone();

    // Tensor variance: ensure tensor op always uses tensor-aware Typst markup
    // by tagging op.name when kind is not set
    let mut node_for_render = node.clone();
    fn tag_tensor_kind(n: &mut EditorNode) {
        match n {
            EditorNode::Operation { operation } => {
                if operation.kind.is_none() && operation.name == "tensor" {
                    operation.kind = Some("tensor".to_string());
                }
                for arg in operation.args.iter_mut() {
                    tag_tensor_kind(arg);
                }
            }
            EditorNode::List { list } => {
                for elem in list.iter_mut() {
                    tag_tensor_kind(elem);
                }
            }
            _ => {}
        }
    }
    tag_tensor_kind(&mut node_for_render);

    extract_boxes_recursive_editor(
        &node_for_render,
        &mut boxes,
        &mut placeholder_idx,
        labeled_positions,
        &uuid_to_position,
        node_id_to_uuid,
        vec![],
        "0",
    );

    Ok(boxes)
}

#[allow(clippy::too_many_arguments)]
fn extract_boxes_recursive_editor(
    node: &EditorNode,
    boxes: &mut Vec<ArgumentBoundingBox>,
    placeholder_idx: &mut usize,
    labeled_positions: &[PlaceholderPosition],
    uuid_positions: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
    node_id_to_uuid: &std::collections::HashMap<String, String>,
    path: Vec<usize>,
    node_id: &str,
) {
    match node {
        EditorNode::Placeholder { placeholder: _ } => {
            // Get position from labeled positions by index
            if let Some(pos) = labeled_positions.get(*placeholder_idx) {
                boxes.push(ArgumentBoundingBox {
                    arg_index: pos.id,
                    node_id: node_id.to_string(),
                    x: pos.x,
                    y: pos.y,
                    width: pos.width,
                    height: pos.height,
                });
            }
            *placeholder_idx += 1;
        }
        EditorNode::Operation { operation } => {
            // Check if this node has a UUID position
            if let Some(uuid) = node_id_to_uuid.get(node_id) {
                if let Some((x, y, w, h)) = uuid_positions.get(uuid) {
                    boxes.push(ArgumentBoundingBox {
                        arg_index: 0, // Will be filled by caller
                        node_id: node_id.to_string(),
                        x: *x,
                        y: *y,
                        width: *w,
                        height: *h,
                    });
                }
            }

            // Recurse into args
            for (i, arg) in operation.args.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                let child_id = format!("{}.{}", node_id, i);
                extract_boxes_recursive_editor(
                    arg,
                    boxes,
                    placeholder_idx,
                    labeled_positions,
                    uuid_positions,
                    node_id_to_uuid,
                    child_path,
                    &child_id,
                );
            }
        }
        EditorNode::List { list } => {
            for (i, elem) in list.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                let child_id = format!("{}.{}", node_id, i);
                extract_boxes_recursive_editor(
                    elem,
                    boxes,
                    placeholder_idx,
                    labeled_positions,
                    uuid_positions,
                    node_id_to_uuid,
                    child_path,
                    &child_id,
                );
            }
        }
        EditorNode::Object { .. } | EditorNode::Const { .. } => {
            // Check if this node has a UUID position
            if let Some(uuid) = node_id_to_uuid.get(node_id) {
                if let Some((x, y, w, h)) = uuid_positions.get(uuid) {
                    boxes.push(ArgumentBoundingBox {
                        arg_index: 0,
                        node_id: node_id.to_string(),
                        x: *x,
                        y: *y,
                        width: *w,
                        height: *h,
                    });
                }
            }
        }
    }
}

/// Extract argument bounding boxes for every AST node using recursive semantic grouping
fn extract_semantic_argument_boxes(
    ast: &Expression,
    ctx: &GlyphContext,
    full_markup: &str,
    labeled_positions: &[PlaceholderPosition],
    node_id_to_uuid: &std::collections::HashMap<String, String>,
    uuid_positions: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
) -> Result<Vec<ArgumentBoundingBox>, String> {
    eprintln!("Extracting semantic boxes recursively...");

    let mut markup_cache: HashMap<String, Vec<LayoutBoundingBox>> = HashMap::new();
    let text_boxes = compile_markup_to_text_boxes(full_markup)?;
    markup_cache.insert(full_markup.to_string(), text_boxes.clone());
    if text_boxes.is_empty() {
        return Ok(Vec::new());
    }

    // Build UUID->Position lookup map for direct matching
    let mut uuid_to_position = std::collections::HashMap::new();
    for (uuid, (x, y, w, h)) in uuid_positions {
        uuid_to_position.insert(uuid.clone(), (*x, *y, *w, *h));
    }
    eprintln!("UUID position map: {} entries", uuid_to_position.len());

    // Sort labeled positions by reading order (y, then x) to handle matrices correctly
    // This is crucial because Typst may output labels in column-major or other orders
    let mut sorted_labeled_positions = labeled_positions.to_vec();
    sorted_labeled_positions.sort_by(|a, b| {
        let y_diff = a.y - b.y;
        if y_diff.abs() < 3.0 {
            // Same row (tolerance for slight vertical variations)
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_diff
                .partial_cmp(&0.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    if !sorted_labeled_positions.is_empty() {
        eprintln!(
            "Sorted {} labeled positions by reading order:",
            sorted_labeled_positions.len()
        );
        for (i, p) in sorted_labeled_positions.iter().take(15).enumerate() {
            eprintln!("  [{}] id={} at ({:.1}, {:.1})", i, p.id, p.x, p.y);
        }
    }

    let mut result = Vec::new();
    let mut label_pool = sorted_labeled_positions.clone();
    assign_boxes_recursive(
        ast,
        ctx,
        &text_boxes,
        "0",
        &mut markup_cache,
        &mut label_pool,
        node_id_to_uuid,
        &uuid_to_position,
        &mut result,
    )?;

    eprintln!("Created {} semantic argument boxes", result.len());

    // POST-PROCESSING: Fix matrix cell order if needed
    if let Expression::Operation { name, args } = ast {
        let is_matrix = name.starts_with("matrix")
            || name.starts_with("vmatrix")
            || name.starts_with("pmatrix");
        if is_matrix {
            fix_matrix_cell_order(&mut result, name, args.len())?;
        }
    }

    Ok(result)
}

/// Fix matrix cell positions by re-sorting them spatially if they're out of reading order
fn fix_matrix_cell_order(
    boxes: &mut [ArgumentBoundingBox],
    _op_name: &str,
    num_args: usize,
) -> Result<(), String> {
    // Get top-level matrix cells (node_id like "0.0", "0.1", etc.)
    let mut matrix_cells: Vec<_> = boxes
        .iter()
        .filter(|b| b.node_id.starts_with("0.") && b.node_id.matches('.').count() == 1)
        .cloned()
        .collect();

    if matrix_cells.len() != num_args {
        eprintln!(
            "  Matrix has {}/{} cells, skipping order fix",
            matrix_cells.len(),
            num_args
        );
        return Ok(());
    }

    // Sort by spatial position (reading order)
    matrix_cells.sort_by(|a, b| {
        let y_diff = a.y - b.y;
        if y_diff.abs() < 3.0 {
            // Same row
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_diff
                .partial_cmp(&0.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    // Check if already in correct order
    let current_order: Vec<_> = matrix_cells.iter().map(|b| b.node_id.as_str()).collect();
    let expected_order: Vec<String> = (0..num_args).map(|i| format!("0.{}", i)).collect();
    let expected_refs: Vec<&str> = expected_order.iter().map(|s| s.as_str()).collect();

    if current_order == expected_refs {
        eprintln!("  Matrix cells already in correct reading order");
        return Ok(());
    }

    eprintln!("  🔧 Fixing matrix cell order:");
    eprintln!("     Current: {:?}", current_order);
    eprintln!("     Fixed:   {:?}", expected_refs);

    // Create a mapping from old node_id to new node_id
    let mut id_mapping: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (new_idx, cell) in matrix_cells.iter().enumerate() {
        id_mapping.insert(cell.node_id.clone(), format!("0.{}", new_idx));
    }

    // Apply the mapping to all boxes
    for b in boxes.iter_mut() {
        // Check if this is a matrix cell that needs remapping
        if let Some(new_id) = id_mapping.get(&b.node_id) {
            let old_id = b.node_id.clone();
            b.node_id = new_id.clone();
            // Extract the arg_index from new node_id
            if let Some(idx_str) = new_id.split('.').next_back() {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    b.arg_index = idx;
                }
            }
            eprintln!("     Remapped: {} -> {}", old_id, new_id);
        } else {
            // Check if this is a descendant of a remapped cell
            for (old_id, new_id) in &id_mapping {
                if b.node_id.starts_with(&format!("{}.", old_id)) {
                    let suffix = &b.node_id[old_id.len()..];
                    b.node_id = format!("{}{}", new_id, suffix);
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Compile Typst markup and collect normalized text bounding boxes
fn compile_markup_to_text_boxes(markup: &str) -> Result<Vec<LayoutBoundingBox>, String> {
    let full_doc = format!(
        r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ {} $)
"#,
        markup
    );

    let world = MinimalWorld::new(&full_doc);
    let document = typst::compile(&world)
        .output
        .map_err(|e| format!("Compilation failed: {:?}", e))?;

    let page = document.pages.first().ok_or("No pages in document")?;

    let mut all_boxes = Vec::new();
    extract_bounding_boxes_from_frame(&page.frame, Transform::identity(), &mut all_boxes);

    if !all_boxes.is_empty() {
        let min_x = all_boxes
            .iter()
            .map(|b| b.x)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = all_boxes
            .iter()
            .map(|b| b.y)
            .fold(f64::INFINITY, |a, b| a.min(b));
        for bbox in &mut all_boxes {
            bbox.x -= min_x;
            bbox.y -= min_y;
        }
    }

    Ok(all_boxes
        .into_iter()
        .filter(|b| b.content_type == "text")
        .collect())
}

/// Recursively assign bounding boxes to every AST argument
#[allow(clippy::too_many_arguments)]
fn assign_boxes_recursive(
    expr: &Expression,
    ctx: &GlyphContext,
    text_boxes: &[LayoutBoundingBox],
    node_id: &str,
    cache: &mut HashMap<String, Vec<LayoutBoundingBox>>,
    labeled_positions: &mut Vec<PlaceholderPosition>,
    node_id_to_uuid: &std::collections::HashMap<String, String>,
    uuid_positions: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
    result: &mut Vec<ArgumentBoundingBox>,
) -> Result<(), String> {
    if let Expression::Operation { name, args } = expr {
        eprintln!(
            "assign_boxes_recursive: node={}, operation={}, args={}, available_boxes={}",
            node_id,
            name,
            args.len(),
            text_boxes.len()
        );

        // UUID-BASED DIRECT LOOKUP: Try to get position from UUID map first
        // This is deterministic - no heuristics, just direct UUID matching
        // Enable for ALL nodes (not just root) to support nested structures like List
        let use_uuid_lookup = !uuid_positions.is_empty();

        if use_uuid_lookup {
            eprintln!(
                "  🔑 UUID positions available (at depth {}), attempting direct lookup",
                node_id
            );
        }

        let arg_signatures = collect_text_boxes_for_args(args, ctx, cache);
        let mut cursor = 0usize;

        for (idx, arg) in args.iter().enumerate() {
            let child_node_id = if node_id.is_empty() {
                format!("0.{}", idx)
            } else {
                format!("{}.{}", node_id, idx)
            };

            // PRIORITY 1: UUID-based direct lookup (deterministic)
            if let Some(uuid) = node_id_to_uuid.get(&child_node_id) {
                let display_uuid = &uuid[..8.min(uuid.len())];
                eprintln!(
                    "  Arg {}: Looking for UUID {}... (len={}) in position map",
                    idx,
                    display_uuid,
                    uuid.len()
                );
                if let Some((x, y, w, h)) = uuid_positions.get(uuid) {
                    eprintln!(
                        "  Arg {}: 🔑 UUID match! {}... -> ({:.1}, {:.1})",
                        idx, display_uuid, x, y
                    );

                    result.push(ArgumentBoundingBox {
                        arg_index: idx,
                        node_id: child_node_id.clone(),
                        x: *x,
                        y: *y,
                        width: *w,
                        height: *h,
                    });

                    // Recursively process children
                    assign_boxes_recursive(
                        arg,
                        ctx,
                        &[],
                        &child_node_id,
                        cache,
                        labeled_positions,
                        node_id_to_uuid,
                        uuid_positions,
                        result,
                    )?;
                    continue;
                }
            }

            // PRIORITY 2: Pattern matching in text boxes
            let pattern = match arg_signatures.get(idx) {
                Some(p) if !p.is_empty() => p.as_slice(),
                _ => {
                    eprintln!("  Arg {}: No pattern available, skipping", idx);
                    continue;
                }
            };

            if pattern.is_empty() {
                continue;
            }

            // Check if we've run out of boxes - try to use labeled position or geometry
            if cursor >= text_boxes.len() {
                eprintln!("  Arg {}: Cursor at end of boxes ({})", idx, cursor);

                // Strategy 1: Pop next labeled position from spatially-sorted pool
                let (fallback_x, fallback_y, fallback_w, fallback_h) =
                    if !labeled_positions.is_empty() {
                        let pos = labeled_positions[0].clone();
                        eprintln!(
                            "    → Using next spatial label: id={} at ({:.1}, {:.1})",
                            pos.id, pos.x, pos.y
                        );
                        labeled_positions.remove(0);
                        (pos.x, pos.y, pos.width, pos.height)
                    } else if !result.is_empty() {
                        // Strategy 2: Estimate based on already-placed siblings (matrix geometry)
                        // For 2x2 matrix: if we have cells 0,1,2 and need cell 3:
                        //   cell 3.x ≈ cell 1.x (same column as cell 1)
                        //   cell 3.y ≈ cell 2.y (same row as cell 2)
                        eprintln!(
                            "    → Using geometric estimation based on {} existing boxes",
                            result.len()
                        );
                        let prev_boxes: Vec<_> = result
                            .iter()
                            .filter(|b| {
                                b.node_id.matches('.').count() == node_id.matches('.').count() + 1
                            })
                            .collect();

                        if prev_boxes.len() >= 2 {
                            // Use position from geometric pattern
                            let x = prev_boxes
                                .iter()
                                .map(|b| b.x)
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap_or(0.0);
                            let y = prev_boxes
                                .iter()
                                .map(|b| b.y)
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap_or(0.0);
                            let w = prev_boxes.iter().map(|b| b.width).sum::<f64>()
                                / prev_boxes.len() as f64;
                            let h = prev_boxes.iter().map(|b| b.height).sum::<f64>()
                                / prev_boxes.len() as f64;
                            (x, y, w, h)
                        } else if !text_boxes.is_empty() {
                            let last_box = &text_boxes[text_boxes.len() - 1];
                            (last_box.x + last_box.width + 5.0, last_box.y, 30.0, 30.0)
                        } else {
                            (0.0, 0.0, 30.0, 30.0)
                        }
                    } else {
                        (0.0, 0.0, 30.0, 30.0)
                    };

                eprintln!(
                    "    → Fallback BBox: ({:.1}, {:.1}) {:.1}×{:.1}",
                    fallback_x, fallback_y, fallback_w, fallback_h
                );

                result.push(ArgumentBoundingBox {
                    arg_index: idx,
                    node_id: child_node_id.clone(),
                    x: fallback_x,
                    y: fallback_y,
                    width: fallback_w,
                    height: fallback_h,
                });

                // Process children with empty slice
                assign_boxes_recursive(
                    arg,
                    ctx,
                    &[],
                    &child_node_id,
                    cache,
                    labeled_positions,
                    node_id_to_uuid,
                    uuid_positions,
                    result,
                )?;
                continue;
            }

            eprintln!(
                "  Arg {}: Searching for pattern (len={}) from cursor={}",
                idx,
                pattern.len(),
                cursor
            );
            eprintln!(
                "    Pattern boxes: {:?}",
                pattern
                    .iter()
                    .map(|b| format!(
                        "{}@({:.1},{:.1})",
                        b.text.as_ref().unwrap_or(&"?".to_string()),
                        b.x,
                        b.y
                    ))
                    .collect::<Vec<_>>()
            );

            let (start, end) = match find_matching_slice(text_boxes, pattern, cursor) {
                Some(range) => {
                    eprintln!("    ✓ Match found at [{}, {})", range.0, range.1);
                    range
                }
                None => {
                    eprintln!("    ✗ Pattern not found sequentially!");
                    eprintln!("    Attempting spatial fallback...");

                    // Try spatial matching as fallback
                    match find_matching_slice_spatial(text_boxes, pattern, cursor) {
                        Some(range) => {
                            eprintln!("    ✓ Spatial match found at [{}, {})", range.0, range.1);
                            range
                        }
                        None => {
                            eprintln!("    ✗ Spatial match also failed, using best-effort slice");
                            let fallback_end = (cursor + pattern.len()).min(text_boxes.len());
                            if fallback_end <= cursor {
                                eprintln!("    ✗ Cannot advance cursor, skipping arg");
                                continue;
                            }
                            (cursor, fallback_end)
                        }
                    }
                }
            };

            let slice = &text_boxes[start..end];
            if slice.is_empty() {
                eprintln!("    ✗ Empty slice, skipping");
                continue;
            }

            let bbox = merge_boxes(slice);
            let child_node_id = if node_id.is_empty() {
                format!("0.{}", idx)
            } else {
                format!("{}.{}", node_id, idx)
            };

            eprintln!(
                "    → BBox: ({:.1}, {:.1}) {}×{}",
                bbox.0, bbox.1, bbox.2, bbox.3
            );

            result.push(ArgumentBoundingBox {
                arg_index: idx,
                node_id: child_node_id.clone(),
                x: bbox.0,
                y: bbox.1,
                width: bbox.2,
                height: bbox.3,
            });

            assign_boxes_recursive(
                arg,
                ctx,
                slice,
                &child_node_id,
                cache,
                labeled_positions,
                node_id_to_uuid,
                uuid_positions,
                result,
            )?;

            // Advance cursor but ensure progress
            cursor = end;
            eprintln!("  Advanced cursor to {}", cursor);
        }
    } else if let Expression::List(elements) = expr {
        // Handle List nodes: recursively process each element
        eprintln!(
            "assign_boxes_recursive: node={}, List with {} elements",
            node_id,
            elements.len()
        );

        for (idx, elem) in elements.iter().enumerate() {
            let child_node_id = if node_id.is_empty() {
                format!("0.{}", idx)
            } else {
                format!("{}.{}", node_id, idx)
            };

            eprintln!(
                "  List element {}: processing at node_id={}",
                idx, child_node_id
            );

            // Check if this element has a UUID position (for filled values)
            if let Some(uuid) = node_id_to_uuid.get(&child_node_id) {
                if let Some((x, y, w, h)) = uuid_positions.get(uuid) {
                    let display_uuid = &uuid[..8.min(uuid.len())];
                    eprintln!(
                        "  List element {}: 🔑 UUID match! {}... -> ({:.1}, {:.1})",
                        idx, display_uuid, x, y
                    );

                    result.push(ArgumentBoundingBox {
                        arg_index: idx,
                        node_id: child_node_id.clone(),
                        x: *x,
                        y: *y,
                        width: *w,
                        height: *h,
                    });

                    // Recursively process children if any
                    assign_boxes_recursive(
                        elem,
                        ctx,
                        &[],
                        &child_node_id,
                        cache,
                        labeled_positions,
                        node_id_to_uuid,
                        uuid_positions,
                        result,
                    )?;
                    continue;
                }
            }

            // No UUID found, try recursive processing
            // This handles nested structures or placeholder positions
            assign_boxes_recursive(
                elem,
                ctx,
                text_boxes,
                &child_node_id,
                cache,
                labeled_positions,
                node_id_to_uuid,
                uuid_positions,
                result,
            )?;
        }
    } else {
        // Leaf nodes (Const, Object, Placeholder, Match)
        // If we reach here with a leaf node, try UUID lookup
        if let Some(uuid) = node_id_to_uuid.get(node_id) {
            if let Some((x, y, w, h)) = uuid_positions.get(uuid) {
                let display_uuid = &uuid[..8.min(uuid.len())];
                eprintln!(
                    "  Leaf node at {}: 🔑 UUID match! {}... -> ({:.1}, {:.1})",
                    node_id, display_uuid, x, y
                );

                result.push(ArgumentBoundingBox {
                    arg_index: 0,
                    node_id: node_id.to_string(),
                    x: *x,
                    y: *y,
                    width: *w,
                    height: *h,
                });
            }
        }
    }

    Ok(())
}

/// Collect text box signatures for each argument using cached renders
fn collect_text_boxes_for_args(
    args: &[Expression],
    ctx: &GlyphContext,
    cache: &mut HashMap<String, Vec<LayoutBoundingBox>>,
) -> Vec<Vec<LayoutBoundingBox>> {
    let mut signatures = Vec::with_capacity(args.len());

    for (i, arg) in args.iter().enumerate() {
        let arg_markup = render_expression(arg, ctx, &RenderTarget::Typst);
        eprintln!("  Arg {} markup: {}", i, arg_markup);
        match get_text_boxes_for_markup(&arg_markup, cache) {
            Ok(boxes) => {
                eprintln!("  Arg {} produces {} text boxes", i, boxes.len());
                signatures.push(boxes);
            }
            Err(err) => {
                eprintln!("  Warning: Could not compile arg {}: {}", i, err);
                signatures.push(Vec::new());
            }
        }
    }

    signatures
}

/// Retrieve text boxes for markup with memoization
fn get_text_boxes_for_markup(
    markup: &str,
    cache: &mut HashMap<String, Vec<LayoutBoundingBox>>,
) -> Result<Vec<LayoutBoundingBox>, String> {
    if let Some(cached) = cache.get(markup) {
        return Ok(cached.clone());
    }

    let boxes = compile_markup_to_text_boxes(markup)?;
    cache.insert(markup.to_string(), boxes.clone());
    Ok(boxes)
}

/// Find contiguous slice in full expression that matches argument signature
fn find_matching_slice(
    haystack: &[LayoutBoundingBox],
    needle: &[LayoutBoundingBox],
    start_index: usize,
) -> Option<(usize, usize)> {
    if needle.is_empty() || start_index >= haystack.len() {
        return None;
    }

    let max_start = haystack.len().saturating_sub(needle.len());
    for start in start_index..=max_start {
        let mut matched = true;
        for offset in 0..needle.len() {
            if !boxes_match(&haystack[start + offset], &needle[offset]) {
                matched = false;
                break;
            }
        }
        if matched {
            return Some((start, start + needle.len()));
        }
    }

    None
}

/// Spatial fallback matching: try to find pattern using 2D reading order
fn find_matching_slice_spatial(
    haystack: &[LayoutBoundingBox],
    needle: &[LayoutBoundingBox],
    start_index: usize,
) -> Option<(usize, usize)> {
    if needle.is_empty() || start_index >= haystack.len() {
        return None;
    }

    eprintln!(
        "      [Spatial] Reordering {} boxes from index {}",
        haystack.len(),
        start_index
    );

    // Create indexed copies of remaining boxes with spatial sorting
    let mut indexed_boxes: Vec<(usize, &LayoutBoundingBox)> = haystack[start_index..]
        .iter()
        .enumerate()
        .map(|(i, b)| (start_index + i, b))
        .collect();

    // Sort by reading order: top-to-bottom, then left-to-right
    indexed_boxes.sort_by(|a, b| {
        let y_diff = a.1.y - b.1.y;
        if y_diff.abs() < 2.0 {
            // Same row (within 2pt tolerance)
            a.1.x
                .partial_cmp(&b.1.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_diff
                .partial_cmp(&0.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    eprintln!("      [Spatial] Reordered boxes:");
    for (orig_idx, b) in indexed_boxes.iter().take(10) {
        eprintln!(
            "        [{}] {}@({:.1},{:.1})",
            orig_idx,
            b.text.as_ref().unwrap_or(&"?".to_string()),
            b.x,
            b.y
        );
    }

    // Try to match pattern in spatially-ordered sequence
    if indexed_boxes.len() < needle.len() {
        eprintln!("      [Spatial] Not enough boxes to match pattern");
        return None;
    }

    for window_start in 0..=(indexed_boxes.len() - needle.len()) {
        let mut all_match = true;
        for offset in 0..needle.len() {
            if !boxes_match(indexed_boxes[window_start + offset].1, &needle[offset]) {
                all_match = false;
                break;
            }
        }

        if all_match {
            // Found a match! Now determine the actual slice range in original haystack
            let matched_indices: Vec<usize> = indexed_boxes
                [window_start..window_start + needle.len()]
                .iter()
                .map(|(orig_idx, _)| *orig_idx)
                .collect();

            let min_idx = *matched_indices.iter().min().unwrap();
            let max_idx = *matched_indices.iter().max().unwrap();

            eprintln!(
                "      [Spatial] ✓ Match found! Original indices: {:?}",
                matched_indices
            );
            eprintln!(
                "      [Spatial] Returning range [{}, {})",
                min_idx,
                max_idx + 1
            );

            return Some((min_idx, max_idx + 1));
        }
    }

    eprintln!("      [Spatial] No match found in reordered sequence");
    None
}

/// Determine if two text boxes represent the same glyph run
fn boxes_match(full: &LayoutBoundingBox, pattern: &LayoutBoundingBox) -> bool {
    if let (Some(full_text), Some(pattern_text)) = (&full.text, &pattern.text) {
        if !full_text.is_empty() || !pattern_text.is_empty() {
            return full_text == pattern_text;
        }
    }

    if !pattern.glyph_ids.is_empty() {
        return full.glyph_ids == pattern.glyph_ids;
    }

    // Fallback to comparing widths/heights when text is unavailable
    (full.width - pattern.width).abs() < 0.01 && (full.height - pattern.height).abs() < 0.01
}

/// Merge a slice of layout boxes into a padded bounding box
fn merge_boxes(boxes: &[LayoutBoundingBox]) -> (f64, f64, f64, f64) {
    let min_x = boxes
        .iter()
        .map(|b| b.x)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let min_y = boxes
        .iter()
        .map(|b| b.y)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_x = boxes
        .iter()
        .map(|b| b.x + b.width)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    let max_y = boxes
        .iter()
        .map(|b| b.y + b.height)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let padding = 4.0;

    (
        min_x - padding,
        min_y - padding,
        (max_x - min_x) + padding * 2.0,
        (max_y - min_y) + padding * 2.0,
    )
}

/// Compile Typst math markup to SVG with placeholder tracking (with known IDs)
///
/// Uses Typst library API to compile math to professional SVG with layout tree access
pub fn compile_math_to_svg_with_ids(
    markup: &str,
    placeholder_ids: &[usize],
    all_slot_ids: &[usize],
) -> Result<CompiledOutput, String> {
    eprintln!("=== compile_math_to_svg_with_ids called (Library API) ===");
    eprintln!("Input markup: {}", markup);
    eprintln!("Expected placeholder IDs: {:?}", placeholder_ids);
    eprintln!("All slot IDs: {:?}", all_slot_ids);

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
            let error_msgs: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
            eprintln!("Typst compilation errors: {:?}", error_msgs);
            return Err(format!(
                "Typst compilation failed: {}",
                error_msgs.join("; ")
            ));
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
    eprintln!(
        "Extracted {} bounding boxes from layout tree",
        all_boxes.len()
    );

    // Normalize coordinates: typst-svg shifts content so min_x/min_y is at 0 (plus padding)
    // We need to replicate this shift to match SVG coordinates
    if !all_boxes.is_empty() {
        let min_x = all_boxes
            .iter()
            .map(|b| b.x)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = all_boxes
            .iter()
            .map(|b| b.y)
            .fold(f64::INFINITY, |a, b| a.min(b));

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

    // NEW APPROACH: Extract positions from data-typst-label attributes in SVG
    // This is much simpler and more reliable than layout tree traversal
    // The render.rs now outputs #[#box[$content$]<ph{id}>] which produces
    // <g data-typst-label="ph{id}"> in the SVG with proper transform attributes

    let mut placeholder_positions = Vec::new();

    // Only extract placeholder positions if there are actual placeholders
    if !placeholder_ids.is_empty() {
        placeholder_positions = extract_positions_from_labels(&svg)?;
        eprintln!(
            "Found {} positions from SVG labels",
            placeholder_positions.len()
        );

        // If no labels found (old markup format), fall back to span-based extraction
        if placeholder_positions.is_empty() {
            eprintln!("No labels found, falling back to span-based extraction");
            let source = world
                .source(world.main())
                .map_err(|e| format!("Failed to get source: {:?}", e))?;
            let mut span_markers: Vec<SpanBasedPlaceholder> = Vec::new();
            extract_placeholders_by_span(frame, &source, Transform::identity(), &mut span_markers);

            placeholder_positions = span_markers
                .iter()
                .map(|m| PlaceholderPosition {
                    id: m.id,
                    x: m.x,
                    y: m.y,
                    width: m.width,
                    height: m.height,
                })
                .collect();

            // If still empty, try SVG glyph detection
            if placeholder_positions.is_empty() {
                eprintln!("No span markers found, falling back to SVG glyph detection");
                placeholder_positions =
                    extract_placeholder_positions_by_symbol(&svg, placeholder_ids)?;
            }
        }
    } else {
        eprintln!("No placeholders in expression - skipping placeholder extraction");
    }

    // Sort by ID for consistent output
    placeholder_positions.sort_by_key(|p| p.id);

    eprintln!("Final placeholder count: {}", placeholder_positions.len());

    // CALIBRATE COORDINATES
    // Calculate offset between Layout coordinates and SVG coordinates using the first placeholder
    let mut offset_x = 0.0;
    let mut offset_y = 0.0;

    if USE_CALIBRATION {
        eprintln!("Calibration enabled - attempting to match layout boxes to SVG coordinates");
    } else {
        eprintln!("Calibration disabled - using layout boxes as-is (may not align with SVG)");
    }

    if USE_CALIBRATION {
        if let Some(first_ph) = placeholder_positions.first() {
            // Find corresponding box in layout tree (Text element with similar size/position relative to others)
            // The square symbol in Typst is a text glyph
            // We look for a text box with width ~18pt (square.stroked size)

            // Find text boxes with width between 10 and 25 (likely squares)
            let candidates: Vec<&LayoutBoundingBox> = all_boxes
                .iter()
                .filter(|b| b.content_type == "text" && b.width > 10.0 && b.width < 25.0)
                .collect();

            eprintln!(
                "Found {} candidate boxes for calibration (width 10-25pt)",
                candidates.len()
            );

            // IMPROVED: Find the box closest to the first placeholder's relative position
            // Instead of just taking first(), find the one with similar relative position
            let match_box = if candidates.len() > 1 {
                // If we have multiple candidates, find the best match by position
                // The first placeholder in SVG should correspond to first square in layout
                // They should have similar relative positions within their coordinate systems
                candidates
                    .iter()
                    .min_by(|a, b| {
                        // Prefer boxes closer to origin (likely the first one)
                        let dist_a = (a.x * a.x + a.y * a.y).sqrt();
                        let dist_b = (b.x * b.x + b.y * b.y).sqrt();
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
            } else {
                candidates.first().copied()
            };

            if let Some(match_box) = match_box {
                // Calculate offset
                // SVG = Layout + Offset
                // Offset = SVG - Layout
                offset_x = first_ph.x - match_box.x;
                offset_y = first_ph.y - match_box.y;

                eprintln!(
                    "Calibrated offset: ({:.2}, {:.2}) using placeholder ID {} matched to layout box at ({:.2}, {:.2})",
                    offset_x, offset_y, first_ph.id, match_box.x, match_box.y
                );

                // Sanity check: offset shouldn't be too large
                if offset_x.abs() > 50.0 || offset_y.abs() > 50.0 {
                    eprintln!(
                        "⚠️ WARNING: Large calibration offset detected! This may indicate matching error."
                    );
                    eprintln!(
                        "   First placeholder SVG: ({:.2}, {:.2})",
                        first_ph.x, first_ph.y
                    );
                    eprintln!(
                        "   Matched layout box: ({:.2}, {:.2})",
                        match_box.x, match_box.y
                    );
                }
            }
        } // End of if let Some(first_ph)
    } // End of if USE_CALIBRATION

    // Apply offset to all layout boxes (if calibration enabled)
    let calibrated_boxes: Vec<LayoutBoundingBox> = if USE_CALIBRATION {
        eprintln!(
            "Applying calibration offset ({:.2}, {:.2}) to {} layout boxes",
            offset_x,
            offset_y,
            all_boxes.len()
        );
        all_boxes
            .iter()
            .map(|b| LayoutBoundingBox {
                x: b.x + offset_x,
                y: b.y + offset_y,
                width: b.width,
                height: b.height,
                content_type: b.content_type.clone(),
                text: b.text.clone(),
                glyph_ids: b.glyph_ids.clone(),
            })
            .collect()
    } else {
        eprintln!("Skipping calibration - using layout boxes as-is");
        all_boxes.clone()
    };

    // Extract argument bounding boxes by grouping content boxes from layout tree
    // Use the CALIBRATED boxes
    let argument_bounding_boxes =
        group_content_into_arguments(&svg, &calibrated_boxes, &placeholder_positions)?;
    eprintln!(
        "Extracted {} argument bounding boxes",
        argument_bounding_boxes.len()
    );

    // Expand viewBox to encompass all content (fixes clipping of interactive markers)
    let expanded_svg = expand_viewbox_for_markers(&svg, &calibrated_boxes)?;

    Ok(CompiledOutput {
        svg: expanded_svg,
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
    boxes: &mut Vec<LayoutBoundingBox>,
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
                    text: Some(text.text.to_string()),
                    glyph_ids: glyphs.iter().map(|g| g.id).collect(),
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
                    text: None,
                    glyph_ids: Vec::new(),
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
                    text: None,
                    glyph_ids: Vec::new(),
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
                    text: None,
                    glyph_ids: Vec::new(),
                });
            }
            FrameItem::Tag(_) => {
                // Tag element (metadata) - skip, no visual representation
            }
        }
    }
}

/// Placeholder info extracted from layout tree using source spans
#[derive(Debug, Clone)]
struct SpanBasedPlaceholder {
    /// Placeholder ID extracted from subscript (e.g., "0" from square.stroked_0)
    id: usize,
    /// X position of the square glyph
    x: f64,
    /// Y position of the square glyph  
    y: f64,
    /// Width of the square
    width: f64,
    /// Height of the square
    height: f64,
}

/// Extract placeholder positions using source spans from layout tree
///
/// This uses the attach-based ID encoding where each placeholder has an invisible
/// bottom marker: attach(square.stroked, b: #text(size: 0.1pt, fill: white)[N])
///
/// We find the Group frames that contain these markers and use the Group's
/// bounding box as the placeholder position. The Group IS the attach structure.
fn extract_placeholders_by_span(
    frame: &Frame,
    source: &Source,
    ts: Transform,
    placeholders: &mut Vec<SpanBasedPlaceholder>,
) {
    // Find groups that contain markers and use their bounding boxes
    find_marker_groups(frame, source, ts, placeholders);

    eprintln!(
        "Found {} placeholders from marker groups",
        placeholders.len()
    );
}

/// Check if a frame contains a tiny marker and return its ID
fn find_marker_in_frame(frame: &Frame, source: &Source) -> Option<usize> {
    for (_pos, item) in frame.items() {
        match item {
            FrameItem::Text(text) => {
                let font_size = text.size.to_pt();
                let text_content = text.text.as_str();

                if font_size < 1.0 {
                    // Try placeholder ID (format: "N")
                    if let Ok(id) = text_content.parse::<usize>() {
                        if let Some(first_glyph) = text.glyphs.first() {
                            let span = first_glyph.span.0;
                            if let Some(range) = source.range(span) {
                                let source_text = &source.text()[range.clone()];
                                if source_text.parse::<usize>().ok() == Some(id) {
                                    return Some(id);
                                }
                            }
                        }
                    }
                    // Try slot marker (format: "SN")
                    else if let Some(stripped) = text_content.strip_prefix('S') {
                        if let Ok(id) = stripped.parse::<usize>() {
                            return Some(id);
                        }
                    }
                }
            }
            FrameItem::Group(group) => {
                // Check nested groups
                if let Some(id) = find_marker_in_frame(&group.frame, source) {
                    return Some(id);
                }
            }
            _ => {}
        }
    }
    None
}

/// Recursively find groups that contain markers
fn find_marker_groups(
    frame: &Frame,
    source: &Source,
    ts: Transform,
    placeholders: &mut Vec<SpanBasedPlaceholder>,
) {
    for (pos, item) in frame.items() {
        let item_ts = ts.pre_concat(Transform::translate(pos.x, pos.y));

        if let FrameItem::Group(group) = item {
            let group_ts = item_ts.pre_concat(group.transform);

            // Check if this group directly contains a marker (not in a nested group)
            let has_direct_marker = group.frame.items().any(|(_p, i)| {
                if let FrameItem::Text(text) = i {
                    let font_size = text.size.to_pt();
                    font_size < 1.0
                } else {
                    false
                }
            });

            if has_direct_marker {
                // This group contains a marker - use its bounding box
                if let Some(id) = find_marker_in_frame(&group.frame, source) {
                    let tl = Point::zero().transform(group_ts);
                    let size = group.frame.size();

                    placeholders.push(SpanBasedPlaceholder {
                        id,
                        x: tl.x.to_pt(),
                        y: tl.y.to_pt(),
                        width: size.x.to_pt(),
                        height: size.y.to_pt(),
                    });

                    eprintln!(
                        "  Found marker group for id {} at ({:.1}, {:.1}) size {:.1}x{:.1}",
                        id,
                        tl.x.to_pt(),
                        tl.y.to_pt(),
                        size.x.to_pt(),
                        size.y.to_pt()
                    );
                }
            } else {
                // No direct marker - recurse into this group
                find_marker_groups(&group.frame, source, group_ts, placeholders);
            }
        }
    }
}

/// Extract placeholder positions by correlating subscripts with squares
///
/// Finds square glyphs and their associated subscript IDs using layout tree spans.
/// Alternative implementation kept for reference
#[allow(dead_code)]
fn extract_placeholders_with_spans(
    frame: &Frame,
    source: &Source,
    placeholder_ids: &[usize],
) -> Vec<PlaceholderPosition> {
    eprintln!("Extracting placeholders using span-based ID detection");

    // First, find all subscript digits with their positions
    let mut subscripts: Vec<SpanBasedPlaceholder> = Vec::new();
    extract_placeholders_by_span(frame, source, Transform::identity(), &mut subscripts);

    // Also extract all layout boxes to find square positions
    let mut all_boxes: Vec<LayoutBoundingBox> = Vec::new();
    extract_bounding_boxes_from_frame(frame, Transform::identity(), &mut all_boxes);

    // Normalize coordinates to match SVG (shift so min becomes 0)
    if !all_boxes.is_empty() {
        let min_x = all_boxes
            .iter()
            .map(|b| b.x)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = all_boxes
            .iter()
            .map(|b| b.y)
            .fold(f64::INFINITY, |a, b| a.min(b));

        for bbox in &mut all_boxes {
            bbox.x -= min_x;
            bbox.y -= min_y;
        }

        // Also normalize subscript marker positions
        for sub in &mut subscripts {
            sub.x -= min_x;
            sub.y -= min_y;
        }

        eprintln!(
            "  Normalized coordinates: shifted by ({:.1}, {:.1})",
            min_x, min_y
        );
    }

    // Find square glyphs (text items containing "□")
    let squares: Vec<&LayoutBoundingBox> = all_boxes
        .iter()
        .filter(|b| {
            b.content_type == "text" && b.text.as_ref().map(|t| t.contains('□')).unwrap_or(false)
        })
        .collect();

    eprintln!(
        "  Found {} subscript markers and {} square glyphs",
        subscripts.len(),
        squares.len()
    );

    // Debug: print all marker and square positions
    for (i, marker) in subscripts.iter().enumerate() {
        eprintln!(
            "    Marker {}: id={} at ({:.1}, {:.1})",
            i, marker.id, marker.x, marker.y
        );
    }
    for (i, sq) in squares.iter().enumerate() {
        eprintln!("    Square {}: at ({:.1}, {:.1})", i, sq.x, sq.y);
    }

    // Match markers to squares using a greedy approach
    // The marker from attach(..., b: ...) is placed BELOW and CENTERED on the square
    // Track which squares have been used
    let mut used_squares: Vec<bool> = vec![false; squares.len()];
    let mut positions = Vec::new();

    // Sort markers by ID to process in order
    let mut sorted_markers = subscripts.clone();
    sorted_markers.sort_by_key(|m| m.id);

    for marker in &sorted_markers {
        // Find the closest UNUSED square
        // The marker from attach(..., b: ...) appears to be placed to the RIGHT of the square
        // (after the square's width) and slightly below
        let mut best_square_idx: Option<usize> = None;
        let mut best_dist = f64::MAX;

        for (sq_idx, sq) in squares.iter().enumerate() {
            if used_squares[sq_idx] {
                continue; // Skip already-matched squares
            }

            // The marker is placed after the square (to the right)
            // marker.x should be close to sq.x + sq.width
            let sq_right_edge = sq.x + sq.width;
            let dx = (marker.x - sq_right_edge).abs();
            let dy = (marker.y - sq.y).abs(); // Should be on same row

            // Marker is to the right of square and on same row
            if dx < 25.0 && dy < 35.0 {
                let dist = dx * dx + dy * dy;
                if dist < best_dist {
                    best_dist = dist;
                    best_square_idx = Some(sq_idx);
                }
            }
        }

        if let Some(sq_idx) = best_square_idx {
            used_squares[sq_idx] = true;
            let sq = squares[sq_idx];
            positions.push(PlaceholderPosition {
                id: marker.id,
                x: sq.x,
                y: sq.y,
                width: sq.width,
                height: sq.height,
            });
            eprintln!(
                "  Matched marker {} to square {} at ({:.1}, {:.1})",
                marker.id, sq_idx, sq.x, sq.y
            );
        } else {
            // Fallback: use marker position offset (marker is to the right of square)
            eprintln!(
                "  No square match for marker {} at ({:.1}, {:.1}), using offset",
                marker.id, marker.x, marker.y
            );
            positions.push(PlaceholderPosition {
                id: marker.id,
                x: marker.x - 18.0, // Square is to the left of marker
                y: marker.y - 5.0,
                width: 18.0,
                height: 18.0,
            });
        }
    }

    // Sort by ID to ensure consistent order
    positions.sort_by_key(|p| p.id);

    // Verify we found the expected placeholders
    if positions.len() != placeholder_ids.len() {
        eprintln!(
            "  Warning: Found {} placeholders but expected {}",
            positions.len(),
            placeholder_ids.len()
        );
    }

    positions
}

/// Legacy function (for backward compatibility)
pub fn compile_math_to_svg(markup: &str) -> Result<CompiledOutput, String> {
    // Extract IDs by counting attach-based placeholders
    let count = markup.matches("attach(square.stroked").count();
    let ids: Vec<usize> = (0..count).collect();
    compile_math_to_svg_with_ids(markup, &ids, &ids)
}

/// Generate mock SVG for testing (temporary)
///
/// This creates a realistic-looking fraction layout with placeholder markers
/// positioned where Typst would actually place them.
/// Kept for testing purposes
#[allow(dead_code)]
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
/// Kept for testing purposes
#[allow(dead_code)]
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
        center_x - bar_width / 2.0,
        bar_y,
        center_x + bar_width / 2.0,
        bar_y
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

/// Escape XML special characters
/// Kept for potential future use
#[allow(dead_code)]
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

    /// Plain text contained in this run, if any
    pub text: Option<String>,

    /// Glyph identifiers for matching text runs
    pub glyph_ids: Vec<u16>,
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

/// Parse a translate transform string like "translate(10.5 20.3)" and return (x, y)
fn parse_translate(transform: &str) -> Option<(f64, f64)> {
    // Match "translate(x y)" or "translate(x, y)"
    let re = regex::Regex::new(r"translate\(([\d.e+-]+)[,\s]+([\d.e+-]+)\)").ok()?;
    let caps = re.captures(transform)?;
    let x = caps.get(1)?.as_str().parse::<f64>().ok()?;
    let y = caps.get(2)?.as_str().parse::<f64>().ok()?;
    Some((x, y))
}

/// Walk up the ancestor chain of a node and accumulate all translate transforms
fn accumulate_ancestor_translates(node: &roxmltree::Node) -> (f64, f64) {
    let mut total_x = 0.0;
    let mut total_y = 0.0;
    let mut depth = 0;

    let mut current = node.parent();
    while let Some(parent) = current {
        if let Some(transform) = parent.attribute("transform") {
            if let Some((x, y)) = parse_translate(transform) {
                total_x += x;
                total_y += y;
                eprintln!(
                    "    depth {}: translate({}, {}) -> cumulative ({}, {})",
                    depth, x, y, total_x, total_y
                );
            }
        }
        depth += 1;
        current = parent.parent();
    }

    (total_x, total_y)
}

/// A glyph position extracted from SVG
#[derive(Debug, Clone)]
struct SvgGlyphPosition {
    glyph_id: String,
    x: f64,
    y: f64,
}

/// Extract ALL glyph positions from SVG using DOM parsing
fn extract_all_glyph_positions(svg: &str) -> Result<Vec<SvgGlyphPosition>, String> {
    let doc = roxmltree::Document::parse(svg).map_err(|e| format!("Failed to parse SVG: {}", e))?;

    let mut positions = Vec::new();
    let mut first_logged = false;

    for node in doc.descendants() {
        if node.tag_name().name() == "use" {
            if let Some(href) = node.attribute(("http://www.w3.org/1999/xlink", "href")) {
                if let Some(glyph_id) = href.strip_prefix("#g") {
                    if !first_logged {
                        eprintln!(
                            "DEBUG: Processing first <use> element with glyph #{}",
                            glyph_id
                        );
                        first_logged = true;
                    }
                    let (x, y) = accumulate_ancestor_translates(&node);
                    positions.push(SvgGlyphPosition {
                        glyph_id: glyph_id.to_string(),
                        x,
                        y,
                    });
                }
            }
        }
    }

    eprintln!(
        "DEBUG: extract_all_glyph_positions found {} positions",
        positions.len()
    );
    if let Some(first) = positions.first() {
        eprintln!(
            "DEBUG: First position: glyph={}, x={:.2}, y={:.2}",
            first.glyph_id, first.x, first.y
        );
    }

    Ok(positions)
}

/// Extract placeholder positions by finding square symbols in SVG
///
/// Typst renders square.stroked as SVG <use> elements with transform matrices.
/// We parse the SVG as XML and walk the DOM to find <use> elements, then
/// accumulate all ancestor translate transforms to get the absolute position.
fn extract_placeholder_positions_by_symbol(
    svg: &str,
    placeholder_ids: &[usize],
) -> Result<Vec<PlaceholderPosition>, String> {
    let expected_count = placeholder_ids.len();
    eprintln!(
        "Extracting {} placeholders by finding square symbols in Typst SVG",
        expected_count
    );
    eprintln!("Using placeholder IDs: {:?}", placeholder_ids);

    let mut positions = Vec::new();

    // Get all glyph positions
    let all_glyphs = extract_all_glyph_positions(svg)?;

    // Group by glyph ID
    let mut glyph_counts: std::collections::HashMap<String, Vec<(f64, f64)>> =
        std::collections::HashMap::new();

    for glyph in &all_glyphs {
        glyph_counts
            .entry(glyph.glyph_id.clone())
            .or_default()
            .push((glyph.x, glyph.y));
    }

    eprintln!(
        "Found {} unique glyphs, {} total",
        glyph_counts.len(),
        all_glyphs.len()
    );
    for (glyph, positions_vec) in &glyph_counts {
        if positions_vec.len() > 1 {
            eprintln!("  Glyph #{}: {} occurrences", glyph, positions_vec.len());
        }
    }

    // Strategy: Find the glyph(s) that best match the expected placeholder count
    // 1. Exact match: single glyph appears exactly expected_count times -> use it
    // 2. Multiple glyphs needed: if no exact match and expected > 6, collect from multiple
    // 3. Close match: use single glyph closest to expected_count
    // 4. Single placeholder: special handling for expected_count == 1
    // 5. Different-sized placeholders: multiple glyphs each appearing once (e.g., nth_root)

    let mut exact_match: Option<(&String, &Vec<(f64, f64)>)> = None;
    let mut best_glyph: Option<(&String, &Vec<(f64, f64)>)> = None;
    let mut best_diff = usize::MAX;

    // First pass: find exact match or closest glyph with multiple occurrences
    for (glyph_id, positions_vec) in &glyph_counts {
        let count = positions_vec.len();
        // Look for glyphs with multiple occurrences first
        if count >= 2 {
            if count == expected_count {
                exact_match = Some((glyph_id, positions_vec));
                break;
            }
            if count <= expected_count + 2 {
                let diff = (count as i32 - expected_count as i32).unsigned_abs() as usize;
                if diff < best_diff {
                    best_diff = diff;
                    best_glyph = Some((glyph_id, positions_vec));
                }
            }
        }
    }

    // Special case: single placeholder (expected_count == 1)
    // Find any glyph that appears exactly once
    if exact_match.is_none() && best_glyph.is_none() && expected_count == 1 {
        for (glyph_id, positions_vec) in &glyph_counts {
            if positions_vec.len() == 1 {
                exact_match = Some((glyph_id, positions_vec));
                break;
            }
        }
    }

    let mut all_square_positions: Vec<(f64, f64, f64)> = Vec::new(); // (x, y, estimated_height)

    if let Some((glyph_id, positions_vec)) = exact_match {
        // Exact match - use this single glyph
        let glyph_height = 18.0;
        eprintln!(
            "  Using glyph #{} with {} positions (exact match)",
            glyph_id,
            positions_vec.len()
        );
        for (x, y) in positions_vec {
            all_square_positions.push((*x, *y, glyph_height));
        }
    } else if expected_count > 2 && best_diff > 0 {
        // No exact match - try combining multiple glyphs
        // This handles cases like integrals where bounds use smaller squares than the main content
        eprintln!(
            "  No exact match for {} placeholders, collecting from multiple glyphs",
            expected_count
        );

        // Collect all glyphs with 2+ occurrences
        for (glyph_id, positions_vec) in &glyph_counts {
            if positions_vec.len() >= 2 {
                let glyph_height = if positions_vec.len() <= 4 { 12.0 } else { 18.0 };
                for (x, y) in positions_vec {
                    all_square_positions.push((*x, *y, glyph_height));
                }
                eprintln!(
                    "  Collecting {} positions from glyph #{}",
                    positions_vec.len(),
                    glyph_id
                );
            }
        }

        // If we got enough or close enough, keep them; otherwise fall through to best_glyph
        if all_square_positions.len() >= expected_count
            || (!all_square_positions.is_empty()
                && all_square_positions.len() >= expected_count - 1)
        {
            eprintln!(
                "  Combined glyphs gave {} positions",
                all_square_positions.len()
            );
        } else {
            // Not enough from combining - clear and try single best glyph
            all_square_positions.clear();
            if let Some((glyph_id, positions_vec)) = best_glyph.as_ref() {
                let glyph_height = 18.0;
                eprintln!(
                    "  Using glyph #{} with {} positions (closest to expected {})",
                    glyph_id,
                    positions_vec.len(),
                    expected_count
                );
                for (x, y) in *positions_vec {
                    all_square_positions.push((*x, *y, glyph_height));
                }
            }
        }
    } else if let Some((glyph_id, positions_vec)) = best_glyph {
        // Use closest match
        let glyph_height = 18.0;
        eprintln!(
            "  Using glyph #{} with {} positions (closest to expected {})",
            glyph_id,
            positions_vec.len(),
            expected_count
        );
        for (x, y) in positions_vec {
            all_square_positions.push((*x, *y, glyph_height));
        }
    } else if expected_count > 0 {
        // Fallback: collect from glyphs
        // First try glyphs with 2+ occurrences
        eprintln!("  No good match, collecting from all glyphs with 2+ occurrences");
        for (glyph_id, positions_vec) in &glyph_counts {
            if positions_vec.len() >= 2 {
                let glyph_height = 18.0;
                for (x, y) in positions_vec {
                    all_square_positions.push((*x, *y, glyph_height));
                }
                eprintln!(
                    "  Collecting {} positions from glyph #{}",
                    positions_vec.len(),
                    glyph_id
                );
            }
        }

        // If still not enough, collect from single-occurrence glyphs
        // This handles cases like nth_root where different-sized squares each appear once
        if all_square_positions.len() < expected_count {
            eprintln!(
                "  Still need {} more, checking single-occurrence glyphs",
                expected_count - all_square_positions.len()
            );
            for (glyph_id, positions_vec) in &glyph_counts {
                if positions_vec.len() == 1 && all_square_positions.len() < expected_count {
                    let glyph_height = 18.0;
                    for (x, y) in positions_vec {
                        all_square_positions.push((*x, *y, glyph_height));
                    }
                    eprintln!("  Collecting 1 position from glyph #{}", glyph_id);
                }
            }
        }
    }

    // Deduplicate positions that are very close together (same visual position)
    all_square_positions.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap()
            .then(a.0.partial_cmp(&b.0).unwrap())
    });

    // Remove duplicates (positions within 5pt of each other)
    let mut unique_positions: Vec<(f64, f64, f64)> = Vec::new();
    for pos in &all_square_positions {
        let is_duplicate = unique_positions
            .iter()
            .any(|p| (p.0 - pos.0).abs() < 5.0 && (p.1 - pos.1).abs() < 5.0);
        if !is_duplicate {
            unique_positions.push(*pos);
        }
    }

    eprintln!(
        "Found {} unique square-like positions (expected {})",
        unique_positions.len(),
        expected_count
    );

    // Sort by visual order (row then column)
    unique_positions.sort_by(|a, b| {
        // Group by rows (within 15pt tolerance)
        let row_a = (a.1 / 15.0).floor() as i32;
        let row_b = (b.1 / 15.0).floor() as i32;
        row_a.cmp(&row_b).then(a.0.partial_cmp(&b.0).unwrap())
    });

    // Match with placeholder IDs
    for (i, (x, y, glyph_height)) in unique_positions.iter().enumerate() {
        if i >= placeholder_ids.len() {
            eprintln!("  Skipping extra square at ({:.1}, {:.1})", x, y);
            continue;
        }

        let placeholder_id = placeholder_ids[i];

        // The y-coordinate from accumulated transforms is the visual position
        // We need to adjust for the glyph's visual rendering:
        // - Typst uses scale(1, -1) which flips text vertically
        // - The transform y-coordinate is where the glyph baseline sits
        // - For overlay positioning, we want the top-left corner
        // After testing: the accumulated y IS the correct visual position
        // The glyph renders from y downward (in SVG coordinates)
        let visual_y = *y - glyph_height;

        eprintln!(
            "  Square {} (ID {}): position ({:.1}, {:.1}) -> visual y={:.1}",
            i, placeholder_id, x, y, visual_y
        );
        positions.push(PlaceholderPosition {
            id: placeholder_id,
            x: *x,
            y: *y, // Use the raw y-coordinate, not adjusted
            width: *glyph_height,
            height: *glyph_height,
        });
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
    eprintln!(
        "Grouping {} layout boxes into arguments...",
        layout_boxes.len()
    );

    let mut argument_boxes = Vec::new();

    // Filter for text content only
    let content_boxes: Vec<&LayoutBoundingBox> = layout_boxes
        .iter()
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
        for line in lines.iter_mut() {
            if let Some(first) = line.first() {
                // If Y centers are close, it's the same line
                // Tolerance increased to 20.0 to handle nested fractions (e.g. 3/x in denominator)
                // which might have components shifted vertically
                let center_y = bbox.y + bbox.height / 2.0;
                let line_y = first.y + first.height / 2.0;
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
        if line.is_empty() {
            continue;
        }

        let min_x = line
            .iter()
            .map(|b| b.x)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let min_y = line
            .iter()
            .map(|b| b.y)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let max_x = line
            .iter()
            .map(|b| b.x + b.width)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let max_y = line
            .iter()
            .map(|b| b.y + b.height)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        let width = (max_x - min_x).max(20.0);
        let height = (max_y - min_y).max(20.0);

        // Add padding
        let padding = 4.0;

        eprintln!(
            "  Line {}: bbox ({:.1}, {:.1}) size {:.1}x{:.1}",
            index, min_x, min_y, width, height
        );

        argument_boxes.push(ArgumentBoundingBox {
            arg_index: index,
            node_id: format!("0.{}", index), // Generate node ID from index
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
#[allow(dead_code)]
fn extract_argument_bounding_boxes_markers(svg: &str) -> Result<Vec<ArgumentBoundingBox>, String> {
    eprintln!("Extracting argument bounding boxes from invisible markers...");

    let mut argument_boxes = Vec::new();

    // Pattern to find translate transforms with white-filled text (our markers)
    // The markers are rendered with fill="#ffffff"
    let pattern_str =
        r###"<g[^>]*transform="translate\(([\d.]+) ([\d.]+)\)"[^>]*>[\s\S]*?fill="#ffffff""###;
    let transform_pattern =
        regex::Regex::new(pattern_str).map_err(|e| format!("Regex error: {}", e))?;

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
        let width = (max_x - x).max(20.0); // Minimum width
        let height = (max_y - y).max(20.0); // Minimum height

        eprintln!(
            "  Arg {}: bbox ({:.1}, {:.1}) size {:.1}x{:.1}",
            arg_index, x, y, width, height
        );

        argument_boxes.push(ArgumentBoundingBox {
            arg_index,
            node_id: format!("0.{}", arg_index), // Generate node ID from index
            x,
            y,
            width,
            height,
        });

        arg_index += 1;
        i += 2; // Move to next pair
    }

    eprintln!("  Created {} argument bounding boxes", argument_boxes.len());

    Ok(argument_boxes)
}

/// Extract bounding boxes of colored argument boxes from Typst SVG
///
/// Each argument is wrapped in #box(fill: rgb(...)) which Typst renders
/// as SVG rectangles. We find these and extract their positions.
#[allow(dead_code)]
fn extract_colored_boxes(svg: &str) -> Result<Vec<BoundingBox>, String> {
    let mut boxes = Vec::new();

    // Pattern: <path fill="#rrggbb" d="M x y h width v height ..."/>
    // Or: <rect fill="#rrggbb" x="..." y="..." width="..." height="..."/>

    let rect_pattern = regex::Regex::new(r#"<rect[^>]*fill="[^"]*"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"#)
        .map_err(|e| format!("Regex error: {}", e))?;

    for cap in rect_pattern.captures_iter(svg) {
        if let (Some(x_str), Some(y_str), Some(w_str), Some(h_str)) =
            (cap.get(1), cap.get(2), cap.get(3), cap.get(4))
        {
            if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                x_str.as_str().parse::<f64>(),
                y_str.as_str().parse::<f64>(),
                w_str.as_str().parse::<f64>(),
                h_str.as_str().parse::<f64>(),
            ) {
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

/// Bounding box for layout elements
/// Kept for potential future use
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct BoundingBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Expand SVG viewBox to encompass all bounding boxes (prevents clipping)
fn expand_viewbox_for_markers(svg: &str, boxes: &[LayoutBoundingBox]) -> Result<String, String> {
    if boxes.is_empty() {
        return Ok(svg.to_string());
    }

    // Calculate actual content bounds from all bounding boxes
    let min_x = boxes.iter().map(|b| b.x).fold(f64::INFINITY, f64::min);
    let min_y = boxes.iter().map(|b| b.y).fold(f64::INFINITY, f64::min);
    let max_x = boxes
        .iter()
        .map(|b| b.x + b.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = boxes
        .iter()
        .map(|b| b.y + b.height)
        .fold(f64::NEG_INFINITY, f64::max);

    // Calculate content dimensions
    let content_width = max_x - min_x;
    let content_height = max_y - min_y;

    // Compute proportional padding: 10% of each dimension, with minimum 20pt
    let padding_x = (content_width * 0.1).max(20.0);
    let padding_y = (content_height * 0.1).max(20.0);

    // Apply padding (allow negative x/y if content requires it)
    let new_x = min_x - padding_x;
    let new_y = min_y - padding_y;
    let new_width = content_width + (2.0 * padding_x);
    let new_height = content_height + (2.0 * padding_y);

    // Parse current viewBox and always replace with content-aware bounds
    let viewbox_regex = regex::Regex::new(r#"viewBox="([^"]+)""#).unwrap();
    if let Some(captures) = viewbox_regex.captures(svg) {
        let old_vb = captures.get(1).unwrap().as_str();
        let old_parts: Vec<f64> = old_vb
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if old_parts.len() == 4 {
            let new_viewbox = format!("{} {} {} {}", new_x, new_y, new_width, new_height);
            let expanded = viewbox_regex.replace(svg, format!(r#"viewBox="{}""#, new_viewbox));

            eprintln!(
                "📐 Content-aware viewBox: [{:.1}, {:.1}] to [{:.1}, {:.1}] → viewBox({:.1}, {:.1}, {:.1}, {:.1})",
                min_x, min_y, max_x, max_y, new_x, new_y, new_width, new_height
            );

            return Ok(expanded.to_string());
        }
    }

    Ok(svg.to_string())
}

/// Extract UUID-based positions from SVG (labels like "id12345678")
#[allow(clippy::type_complexity)]
fn extract_uuid_positions(
    svg: &str,
) -> Result<std::collections::HashMap<String, (f64, f64, f64, f64)>, String> {
    use roxmltree::Document;

    let doc = Document::parse(svg).map_err(|e| format!("Failed to parse SVG: {}", e))?;

    let mut positions = std::collections::HashMap::new();

    fn find_uuid_labels(
        node: roxmltree::Node,
        parent_transforms: &[(f64, f64)],
        positions: &mut std::collections::HashMap<String, (f64, f64, f64, f64)>,
    ) {
        let mut current_transforms = parent_transforms.to_vec();
        if let Some(transform) = node.attribute("transform") {
            if let Some((tx, ty)) = parse_translate(transform) {
                current_transforms.push((tx, ty));
            }
        }

        if let Some(label) = node.attribute("data-typst-label") {
            if let Some(uuid_part) = label.strip_prefix("id") {
                // Remove "id" prefix (rest is the UUID)
                let (abs_x, abs_y) = current_transforms
                    .iter()
                    .fold((0.0, 0.0), |(ax, ay), (tx, ty)| (ax + tx, ay + ty));

                let (width, height) = estimate_group_size(&node);

                positions.insert(uuid_part.to_string(), (abs_x, abs_y, width, height));
                let display_uuid = &uuid_part[..8.min(uuid_part.len())];
                eprintln!(
                    "Found UUID label: id={}... (len={}) at ({:.1}, {:.1})",
                    display_uuid,
                    uuid_part.len(),
                    abs_x,
                    abs_y
                );
            }
        }

        for child in node.children() {
            find_uuid_labels(child, &current_transforms, positions);
        }
    }

    find_uuid_labels(doc.root(), &[], &mut positions);
    Ok(positions)
}

/// Extract placeholder positions from SVG using data-typst-label attributes
///
/// This is the NEW APPROACH: Typst's SVG output includes data-typst-label attributes
/// on <g> elements that correspond to labeled boxes in the source.
/// We use syntax like #[#box[$content$]<label>] to create these labels.
///
/// The label format is:
/// - "ph{id}" for placeholders (empty slots)
/// - "sl{index}" for filled slots
fn extract_positions_from_labels(svg: &str) -> Result<Vec<PlaceholderPosition>, String> {
    use roxmltree::Document;

    let doc = Document::parse(svg).map_err(|e| format!("Failed to parse SVG: {}", e))?;

    let mut positions = Vec::new();

    // Find all elements with data-typst-label attribute
    fn find_labeled_elements(
        node: roxmltree::Node,
        parent_transforms: &[(f64, f64)],
        positions: &mut Vec<PlaceholderPosition>,
    ) {
        // Check for transform on this node
        let mut current_transforms = parent_transforms.to_vec();
        if let Some(transform) = node.attribute("transform") {
            if let Some((tx, ty)) = parse_translate(transform) {
                current_transforms.push((tx, ty));
            }
        }

        // Check for data-typst-label attribute
        if let Some(label) = node.attribute("data-typst-label") {
            // Calculate absolute position by summing all transforms
            let (abs_x, abs_y) = current_transforms
                .iter()
                .fold((0.0, 0.0), |(ax, ay), (tx, ty)| (ax + tx, ay + ty));

            // Extract ID from label
            // Format: "ph{id}" for placeholders, "sl{index}" for filled slots
            let id = if let Some(stripped) = label.strip_prefix("ph") {
                stripped.parse::<usize>().ok()
            } else if let Some(stripped) = label.strip_prefix("sl") {
                // For filled slots, use index + 1000 to distinguish from placeholders
                stripped.parse::<usize>().ok().map(|i| i + 1000)
            } else {
                None
            };

            if let Some(id) = id {
                // Get the bounding box of the labeled group
                // We need to find the content size inside this group
                let (width, height) = estimate_group_size(&node);

                positions.push(PlaceholderPosition {
                    id,
                    x: abs_x,
                    y: abs_y,
                    width,
                    height,
                });
                eprintln!(
                    "Found labeled element: label='{}', id={}, pos=({:.1}, {:.1}), size=({:.1}x{:.1})",
                    label, id, abs_x, abs_y, width, height
                );
            }
        }

        // Recurse into children
        for child in node.children() {
            find_labeled_elements(child, &current_transforms, positions);
        }
    }

    find_labeled_elements(doc.root(), &[], &mut positions);

    eprintln!(
        "Extracted {} positions from data-typst-label attributes",
        positions.len()
    );
    Ok(positions)
}

/// Estimate the size of a group by looking at its content
fn estimate_group_size(node: &roxmltree::Node) -> (f64, f64) {
    // Default size for a placeholder square
    let default_width = 18.0;
    let default_height = 18.0;

    // Try to find a use element (glyph reference) inside the group
    fn find_glyph_size(node: &roxmltree::Node) -> Option<(f64, f64)> {
        if node.tag_name().name() == "use" {
            // Glyphs typically have a standard size
            // The square.stroked glyph is about 18pt x 18pt
            return Some((18.0, 18.0));
        }
        for child in node.children() {
            if let Some(size) = find_glyph_size(&child) {
                return Some(size);
            }
        }
        None
    }

    find_glyph_size(node).unwrap_or((default_width, default_height))
}

/// Extract placeholder positions from SVG text (legacy method)
///
/// Searches for marker patterns like ⟨⟨PH0⟩⟩ in the SVG and extracts
/// their positions from the parent <text> element attributes.
#[allow(dead_code)]
fn extract_placeholder_positions(svg: &str) -> Result<Vec<PlaceholderPosition>, String> {
    let mut positions = Vec::new();

    // Debug: Print SVG snippet
    eprintln!("Extracting placeholders from SVG (length: {})", svg.len());
    eprintln!("SVG snippet: {}", &svg[..svg.len().min(300)]);

    // Parse SVG to find text elements containing placeholder markers
    // Pattern: <text x="..." y="...">...⟨⟨PH{id}⟩⟩...</text>

    // Try to find the markers
    let marker_pattern =
        regex::Regex::new(r"⟨⟨PH(\d+)⟩⟩").map_err(|e| format!("Regex error: {}", e))?;

    eprintln!("Searching for pattern: ⟨⟨PH(\\d+)⟩⟩");

    let matches: Vec<_> = marker_pattern.captures_iter(svg).collect();
    eprintln!("Found {} marker matches", matches.len());

    for cap in matches {
        if let Some(id_match) = cap.get(1) {
            let id: usize = id_match
                .as_str()
                .parse()
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
                    width: 30.0,  // Default width - will be refined
                    height: 20.0, // Default height - will be refined
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
