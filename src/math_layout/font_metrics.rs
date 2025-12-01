// Font metrics for mathematical typesetting
//
// This module will eventually contain TeX-compatible font metrics
// extracted from Computer Modern / Latin Modern Math fonts.
//
// TODO: Port metrics from KaTeX's fontMetricsData.js

use std::collections::HashMap;

/// Font metrics database
pub struct FontMetrics {
    /// Metrics for individual characters
    pub char_metrics: HashMap<char, CharMetrics>,

    /// Extensible character definitions (for large operators, brackets, etc.)
    pub extensible_chars: HashMap<char, ExtensibleChar>,
}

impl FontMetrics {
    /// Create empty metrics (placeholder)
    pub fn new() -> Self {
        FontMetrics {
            char_metrics: HashMap::new(),
            extensible_chars: HashMap::new(),
        }
    }

    /// Get metrics for a character
    pub fn get_char_metrics(&self, c: char) -> Option<&CharMetrics> {
        self.char_metrics.get(&c)
    }

    /// Get extensible character definition
    pub fn get_extensible(&self, c: char) -> Option<&ExtensibleChar> {
        self.extensible_chars.get(&c)
    }

    /// Check if character is extensible
    pub fn is_extensible(&self, c: char) -> bool {
        self.extensible_chars.contains_key(&c)
    }
}

/// Metrics for an individual character
#[derive(Debug, Clone, Copy)]
pub struct CharMetrics {
    /// Character width in em units
    pub width: f64,

    /// Height above baseline in em units
    pub height: f64,

    /// Depth below baseline in em units
    pub depth: f64,

    /// Italic correction (spacing adjustment)
    pub italic: f64,

    /// Skew (for accent positioning)
    pub skew: f64,
}

impl Default for CharMetrics {
    fn default() -> Self {
        CharMetrics {
            width: 0.5,
            height: 0.7,
            depth: 0.0,
            italic: 0.0,
            skew: 0.0,
        }
    }
}

/// Extensible character definition
/// Used for symbols that can grow vertically (integrals, brackets, etc.)
#[derive(Debug, Clone)]
pub struct ExtensibleChar {
    /// Top piece (optional)
    pub top: Option<char>,

    /// Middle piece (optional, e.g., for large braces)
    pub middle: Option<char>,

    /// Bottom piece (optional)
    pub bottom: Option<char>,

    /// Repeating piece to fill the height
    pub rep: char,
}

/// Load default font metrics (placeholder)
/// TODO: Replace with actual Computer Modern metrics from KaTeX
pub fn load_default_metrics() -> FontMetrics {
    let mut metrics = FontMetrics::new();

    // Add some basic characters (rough estimates)
    // These will be replaced with real TeX metrics

    // Latin letters
    for c in 'a'..='z' {
        metrics.char_metrics.insert(
            c,
            CharMetrics {
                width: 0.5,
                height: 0.43,
                depth: 0.0,
                italic: 0.0,
                skew: 0.0,
            },
        );
    }

    for c in 'A'..='Z' {
        metrics.char_metrics.insert(
            c,
            CharMetrics {
                width: 0.7,
                height: 0.68,
                depth: 0.0,
                italic: 0.0,
                skew: 0.0,
            },
        );
    }

    // Digits
    for c in '0'..='9' {
        metrics.char_metrics.insert(
            c,
            CharMetrics {
                width: 0.5,
                height: 0.64,
                depth: 0.0,
                italic: 0.0,
                skew: 0.0,
            },
        );
    }

    // Some Greek letters (examples)
    metrics.char_metrics.insert(
        'α',
        CharMetrics {
            width: 0.62,
            height: 0.43,
            depth: 0.0,
            italic: 0.0,
            skew: 0.0,
        },
    );

    metrics.char_metrics.insert(
        'β',
        CharMetrics {
            width: 0.56,
            height: 0.69,
            depth: 0.19,
            italic: 0.0,
            skew: 0.0,
        },
    );

    metrics.char_metrics.insert(
        'π',
        CharMetrics {
            width: 0.57,
            height: 0.43,
            depth: 0.0,
            italic: 0.0,
            skew: 0.0,
        },
    );

    // Extensible symbols
    metrics.extensible_chars.insert(
        '∫',
        ExtensibleChar {
            top: Some('⌠'),
            middle: None,
            bottom: Some('⌡'),
            rep: '⎮',
        },
    );

    metrics.extensible_chars.insert(
        '∑',
        ExtensibleChar {
            top: Some('⎲'),
            middle: None,
            bottom: Some('⎳'),
            rep: '│',
        },
    );

    metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_metrics() {
        let metrics = load_default_metrics();

        // Check some basic characters exist
        assert!(metrics.get_char_metrics('x').is_some());
        assert!(metrics.get_char_metrics('5').is_some());
        assert!(metrics.get_char_metrics('α').is_some());
    }

    #[test]
    fn test_extensible_chars() {
        let metrics = load_default_metrics();

        assert!(metrics.is_extensible('∫'));
        assert!(metrics.is_extensible('∑'));
        assert!(!metrics.is_extensible('x'));
    }
}
