//! Test Option 2: Extract source spans from Typst layout tree
//!
//! Goal: Determine if we can map FrameItems back to source positions,
//! which would let us identify which placeholder each glyph came from.

use std::path::PathBuf;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::layout::{Frame, FrameItem, Point, Transform};
use typst::syntax::{FileId, Source, Span, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

/// Minimal World implementation for Typst compilation
#[derive(Clone)]
struct MinimalWorld {
    library: LazyHash<Library>,
    font_book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main_source: Source,
}

impl MinimalWorld {
    fn new(source_text: &str) -> Self {
        let fonts: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::from_static(data)))
            .collect();

        let font_book = FontBook::from_fonts(&fonts);
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        let main_source = Source::new(main_id, source_text.to_string());

        Self {
            library: LazyHash::new(Library::default()),
            font_book: LazyHash::new(font_book),
            fonts,
            main_source,
        }
    }

    fn source_text(&self) -> &str {
        self.main_source.text()
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
        Some(Datetime::from_ymd(2024, 1, 1).unwrap())
    }
}

/// Item with span info extracted from layout tree
#[derive(Debug)]
struct SpannedItem {
    item_type: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    text: Option<String>,
    span: Option<Span>,
    source_range: Option<std::ops::Range<usize>>,
}

/// Extract items with span info from frame
fn extract_spanned_items(
    frame: &Frame,
    ts: Transform,
    source: &Source,
    items: &mut Vec<SpannedItem>,
) {
    for (pos, item) in frame.items() {
        let item_ts = ts.pre_concat(Transform::translate(pos.x, pos.y));

        match item {
            FrameItem::Group(group) => {
                let group_ts = item_ts.pre_concat(group.transform);
                extract_spanned_items(&group.frame, group_ts, source, items);
            }
            FrameItem::Text(text) => {
                let mut width = 0.0;
                for glyph in &text.glyphs {
                    width += glyph.x_advance.at(text.size).to_pt();
                }
                let height = text.size.to_pt();
                let tl = Point::zero().transform(item_ts);

                // Span is on each Glyph, not on TextItem
                // Get span from first glyph if available
                let (span, source_range) = if let Some(first_glyph) = text.glyphs.first() {
                    let glyph_span = first_glyph.span.0;
                    let range = source.range(glyph_span);
                    (Some(glyph_span), range)
                } else {
                    (None, None)
                };

                items.push(SpannedItem {
                    item_type: "text".to_string(),
                    x: tl.x.to_pt(),
                    y: tl.y.to_pt(),
                    width,
                    height,
                    text: Some(text.text.to_string()),
                    span,
                    source_range,
                });
            }
            FrameItem::Shape(shape, span) => {
                let bbox_size = shape.geometry.bbox_size();
                let tl = Point::zero().transform(item_ts);

                let source_range = source.range(*span);

                items.push(SpannedItem {
                    item_type: "shape".to_string(),
                    x: tl.x.to_pt(),
                    y: tl.y.to_pt(),
                    width: bbox_size.x.to_pt(),
                    height: bbox_size.y.to_pt(),
                    text: None,
                    span: Some(*span),
                    source_range,
                });
            }
            _ => {}
        }
    }
}

fn main() {
    println!("=== Testing Source Span Extraction from Typst Layout Tree ===\n");

    // Test 1: Simple placeholder
    let test1_markup = r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ square.stroked $)"#;

    println!("--- Test 1: Single placeholder ---");
    println!("Source:\n{}\n", test1_markup);

    let world = MinimalWorld::new(test1_markup);
    let result = typst::compile(&world);

    match result.output {
        Ok(doc) => {
            let page = &doc.pages[0];
            let frame = &page.frame;

            let mut items = Vec::new();
            extract_spanned_items(frame, Transform::identity(), &world.main_source, &mut items);

            println!("Extracted {} items with spans:\n", items.len());
            for (i, item) in items.iter().enumerate() {
                println!(
                    "Item {}: {} at ({:.1}, {:.1})",
                    i, item.item_type, item.x, item.y
                );
                if let Some(text) = &item.text {
                    println!("  Text content: {:?}", text);
                }
                if let Some(range) = &item.source_range {
                    let source_snippet = &world.source_text()[range.clone()];
                    println!("  Source range: {:?}", range);
                    println!("  Source text: {:?}", source_snippet);
                } else {
                    println!("  Source range: None (detached span)");
                }
                println!();
            }
        }
        Err(errors) => {
            println!("Compilation failed:");
            for e in errors {
                println!("  {:?}", e);
            }
        }
    }

    // Test 2: Matrix with multiple placeholders
    println!("\n--- Test 2: Matrix with 4 placeholders ---");

    // We'll use unique markers to identify each placeholder in source
    let test2_markup = r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ mat(delim: "[", square.stroked, square.stroked; square.stroked, square.stroked) $)"#;

    println!("Source:\n{}\n", test2_markup);

    let world2 = MinimalWorld::new(test2_markup);
    let result2 = typst::compile(&world2);

    match result2.output {
        Ok(doc) => {
            let page = &doc.pages[0];
            let frame = &page.frame;

            let mut items = Vec::new();
            extract_spanned_items(
                frame,
                Transform::identity(),
                &world2.main_source,
                &mut items,
            );

            // Filter to just text items (which include the squares)
            let text_items: Vec<_> = items.iter().filter(|i| i.item_type == "text").collect();

            println!("Extracted {} text items:\n", text_items.len());
            for (i, item) in text_items.iter().enumerate() {
                println!(
                    "Text {}: at ({:.1}, {:.1}) size {:.1}x{:.1}",
                    i, item.x, item.y, item.width, item.height
                );
                if let Some(text) = &item.text {
                    println!("  Glyph text: {:?}", text);
                }
                if let Some(range) = &item.source_range {
                    let source_snippet = &world2.source_text()[range.clone()];
                    println!("  Source range: {:?} → {:?}", range, source_snippet);
                } else {
                    println!("  Source range: None");
                }
            }

            // Key question: Do different square.stroked instances have different source ranges?
            println!("\n=== Key Finding ===");
            let ranges: Vec<_> = text_items
                .iter()
                .filter_map(|i| i.source_range.clone())
                .collect();

            if ranges.is_empty() {
                println!("❌ No source ranges found - spans are detached");
            } else {
                let unique_ranges: std::collections::HashSet<_> = ranges.iter().collect();
                if unique_ranges.len() == ranges.len() {
                    println!("✅ All {} items have UNIQUE source ranges!", ranges.len());
                    println!("   We can use source position to identify placeholders!");
                } else {
                    println!(
                        "⚠️ {} items but only {} unique ranges",
                        ranges.len(),
                        unique_ranges.len()
                    );
                    println!("   Some items share the same source range");
                }
            }
        }
        Err(errors) => {
            println!("Compilation failed:");
            for e in errors {
                println!("  {:?}", e);
            }
        }
    }

    // Test 3: Matrix with DIFFERENT content to see if spans differ
    println!("\n--- Test 3: Matrix with labeled placeholders ---");

    let test3_markup = r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ mat(delim: "[", a, b; c, d) $)"#;

    println!("Source:\n{}\n", test3_markup);

    let world3 = MinimalWorld::new(test3_markup);
    let result3 = typst::compile(&world3);

    match result3.output {
        Ok(doc) => {
            let page = &doc.pages[0];
            let frame = &page.frame;

            let mut items = Vec::new();
            extract_spanned_items(
                frame,
                Transform::identity(),
                &world3.main_source,
                &mut items,
            );

            let text_items: Vec<_> = items.iter().filter(|i| i.item_type == "text").collect();

            println!("Extracted {} text items:\n", text_items.len());
            for (i, item) in text_items.iter().enumerate() {
                if let Some(range) = &item.source_range {
                    let source_snippet = &world3.source_text()[range.clone()];
                    println!(
                        "Text {}: {:?} at ({:.1}, {:.1}) → source {:?}",
                        i, item.text, item.x, item.y, source_snippet
                    );
                }
            }
        }
        Err(_) => println!("Compilation failed"),
    }

    // Test 4: Use UNIQUE placeholder representations
    println!("\n--- Test 4: Unique placeholder symbols ---");

    // Instead of using the same square.stroked for all, use subscripted versions
    // or attach invisible unique identifiers
    let test4_markup = r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ mat(delim: "[", square.stroked_0, square.stroked_1; square.stroked_2, square.stroked_3) $)"#;

    println!("Source (with subscript IDs):\n{}\n", test4_markup);

    let world4 = MinimalWorld::new(test4_markup);
    let result4 = typst::compile(&world4);

    match result4.output {
        Ok(doc) => {
            let page = &doc.pages[0];
            let frame = &page.frame;

            let mut items = Vec::new();
            extract_spanned_items(
                frame,
                Transform::identity(),
                &world4.main_source,
                &mut items,
            );

            let text_items: Vec<_> = items.iter().filter(|i| i.item_type == "text").collect();

            println!("Extracted {} text items:", text_items.len());
            for (i, item) in text_items.iter().enumerate() {
                if let Some(range) = &item.source_range {
                    let source_snippet = &world4.source_text()[range.clone()];
                    println!(
                        "  {}: {:?} at ({:.1}, {:.1}) → {:?}",
                        i, item.text, item.x, item.y, source_snippet
                    );
                } else {
                    println!(
                        "  {}: {:?} at ({:.1}, {:.1}) → NO SPAN",
                        i, item.text, item.x, item.y
                    );
                }
            }

            // Check uniqueness
            let ranges: Vec<_> = text_items
                .iter()
                .filter(|i| {
                    i.text
                        .as_ref()
                        .map(|t| {
                            t.contains('□')
                                || t.contains('0')
                                || t.contains('1')
                                || t.contains('2')
                                || t.contains('3')
                        })
                        .unwrap_or(false)
                })
                .filter_map(|i| i.source_range.clone())
                .collect();

            let unique: std::collections::HashSet<_> = ranges.iter().collect();
            if unique.len() == ranges.len() && !ranges.is_empty() {
                println!("\n✅ All placeholder-related items have UNIQUE source ranges!");
            } else {
                println!("\n⚠️ {} ranges, {} unique", ranges.len(), unique.len());
            }
        }
        Err(_) => println!("Compilation failed"),
    }

    // Test 5: Use attach() with invisible subscripts
    println!("\n--- Test 5: Invisible subscript markers ---");

    let test5_markup = r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ mat(delim: "[", attach(square.stroked, b: #text(size: 0.1pt)[0]), attach(square.stroked, b: #text(size: 0.1pt)[1]); attach(square.stroked, b: #text(size: 0.1pt)[2]), attach(square.stroked, b: #text(size: 0.1pt)[3])) $)"#;

    println!("Source (with invisible attach markers):\n");

    let world5 = MinimalWorld::new(test5_markup);
    let result5 = typst::compile(&world5);

    match result5.output {
        Ok(doc) => {
            let page = &doc.pages[0];
            let frame = &page.frame;

            let mut items = Vec::new();
            extract_spanned_items(
                frame,
                Transform::identity(),
                &world5.main_source,
                &mut items,
            );

            let text_items: Vec<_> = items.iter().filter(|i| i.item_type == "text").collect();

            println!("Extracted {} text items:", text_items.len());

            // Look for items that might be our markers
            let mut marker_count = 0;
            for item in &text_items {
                if let Some(text) = &item.text {
                    if text == "0" || text == "1" || text == "2" || text == "3" {
                        marker_count += 1;
                        if let Some(range) = &item.source_range {
                            let source_snippet = &world5.source_text()[range.clone()];
                            println!(
                                "  Marker {:?} at ({:.1}, {:.1}) → {:?}",
                                text, item.x, item.y, source_snippet
                            );
                        }
                    }
                }
            }

            if marker_count == 4 {
                println!("\n✅ Found all 4 unique markers with positions!");
                println!("   We can map marker position → placeholder ID");
            } else {
                println!("\n⚠️ Only found {} markers", marker_count);
            }
        }
        Err(errors) => {
            println!("Compilation failed:");
            for e in errors {
                println!("  {:?}", e);
            }
        }
    }

    println!("\n=== Conclusion ===");
    println!("Option 2 works IF each placeholder has unique source text.");
    println!("Solutions:");
    println!("  A) Use subscripts: square.stroked_0, square.stroked_1, ...");
    println!("  B) Use attach() with tiny invisible markers");
    println!("  C) Use completely different symbols per placeholder");
}
