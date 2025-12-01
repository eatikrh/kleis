// Layout box data structures
//
// These types represent the output of the layout engine.
// They describe WHAT to render and WHERE, but not HOW.
// The actual rendering (SVG, Canvas, etc.) is done by separate renderers.

/// A positioned box in 2D space with dimensions
///
/// This is the fundamental unit of layout. All dimensions are in em units
/// (relative to base font size) and positions are relative to the baseline.
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Width of the box in em units
    pub width: f64,

    /// Height above baseline in em units
    pub height: f64,

    /// Depth below baseline in em units  
    pub depth: f64,

    /// Distance from top of box to baseline
    pub baseline: f64,

    /// Child elements with positions relative to this box's origin
    pub children: Vec<PositionedElement>,
}

impl LayoutBox {
    /// Total vertical size
    pub fn total_height(&self) -> f64 {
        self.height + self.depth
    }

    /// Bounding box for this layout
    pub fn bbox(&self) -> BoundingBox {
        BoundingBox {
            x: 0.0,
            y: -self.height,
            width: self.width,
            height: self.total_height(),
        }
    }

    /// Create a simple text box
    pub fn text(content: &str, font_size: f64, font_family: FontFamily, italic: bool) -> Self {
        // Rough estimate - will be replaced with real font metrics
        let width = content.len() as f64 * font_size * 0.5;
        let height = font_size * 0.8;
        let depth = font_size * 0.2;

        LayoutBox {
            width,
            height,
            depth,
            baseline: height,
            children: vec![PositionedElement {
                x: 0.0,
                y: 0.0,
                content: ElementContent::Text {
                    content: content.to_string(),
                    font_size,
                    font_family,
                    italic,
                },
            }],
        }
    }
}

/// An element positioned within its parent box
#[derive(Debug, Clone)]
pub struct PositionedElement {
    /// X offset from parent's origin (left edge)
    pub x: f64,

    /// Y offset from parent's baseline (positive = down)
    pub y: f64,

    /// The content to render at this position
    pub content: ElementContent,
}

/// The actual content to be rendered
#[derive(Debug, Clone)]
pub enum ElementContent {
    /// Plain text character or symbol
    Text {
        content: String,
        font_size: f64,
        font_family: FontFamily,
        italic: bool,
    },

    /// Interactive placeholder (CRITICAL for editing)
    Placeholder {
        id: usize,
        hint: String,
        width: f64,
        height: f64,
    },

    /// Horizontal line (fraction bars, etc.)
    HorizontalLine { width: f64, thickness: f64 },

    /// Vertical line (matrix delimiters, etc.)
    VerticalLine { height: f64, thickness: f64 },

    /// Extensible symbol (integrals, summations, brackets)
    ExtensibleSymbol {
        base_char: char,
        target_height: f64,
        pieces: Vec<GlyphPiece>,
    },

    /// Nested group of elements
    Group {
        children: Vec<PositionedElement>,
        transform: Option<Transform>,
    },

    /// SVG path for custom shapes (radical signs, etc.)
    Path {
        data: String,
        fill: Color,
        stroke: Option<Stroke>,
    },
}

/// Font family for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    /// Latin Modern Math Roman (main text)
    Main,
    /// Latin Modern Math (mathematical symbols)
    Math,
    /// Script/calligraphic style
    Script,
    /// Fraktur/gothic style
    Fraktur,
    /// Sans-serif style
    SansSerif,
    /// Monospace/typewriter style
    Monospace,
}

/// Color representation (RGBA)
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 1.0,
    };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
        Color { r, g, b, a }
    }
}

/// Stroke style for paths
#[derive(Debug, Clone)]
pub struct Stroke {
    pub color: Color,
    pub width: f64,
    pub dash_array: Option<Vec<f64>>,
}

/// Transform for rotations, scaling, etc.
#[derive(Debug, Clone)]
pub struct Transform {
    /// SVG transform matrix [a, b, c, d, e, f]
    pub matrix: [f64; 6],
}

impl Transform {
    /// Identity transform
    pub fn identity() -> Self {
        Transform {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    /// Translation transform
    pub fn translate(x: f64, y: f64) -> Self {
        Transform {
            matrix: [1.0, 0.0, 0.0, 1.0, x, y],
        }
    }

    /// Scale transform
    pub fn scale(sx: f64, sy: f64) -> Self {
        Transform {
            matrix: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }
}

/// Piece of an extensible character
#[derive(Debug, Clone)]
pub struct GlyphPiece {
    pub glyph_id: u16,
    pub y_offset: f64,
}

/// Bounding box for collision detection
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    /// Check if a point is inside this box
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Check if this box intersects another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_box_total_height() {
        let layout = LayoutBox {
            width: 10.0,
            height: 8.0,
            depth: 2.0,
            baseline: 8.0,
            children: vec![],
        };

        assert_eq!(layout.total_height(), 10.0);
    }

    #[test]
    fn test_bounding_box_contains() {
        let bbox = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };

        assert!(bbox.contains(5.0, 5.0));
        assert!(!bbox.contains(15.0, 15.0));
    }

    #[test]
    fn test_color_creation() {
        let red = Color::rgb(255, 0, 0);
        assert_eq!(red.r, 255);
        assert_eq!(red.a, 1.0);

        let transparent = Color::rgba(0, 0, 0, 0.5);
        assert_eq!(transparent.a, 0.5);
    }
}
