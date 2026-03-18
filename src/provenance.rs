//! Provenance Tracking for Kleis Definitions
//!
//! This module tracks which file each definition originated from,
//! enabling file unloading, reloading, and IDE integration.
//!
//! ## Design
//!
//! When a file is loaded, we record:
//! - Which functions it defined
//! - Which data types it defined
//! - Which structures it defined
//! - Which implements blocks it defined
//!
//! This enables:
//! - `:unload file.kleis` — remove all definitions from that file
//! - `:reload file.kleis` — unload + load
//! - `:reset` — clear all definitions
//! - IDE integration — update on save
//!
//! ## Usage
//!
//! ```ignore
//! let mut tracker = ProvenanceTracker::new();
//!
//! // Record definitions from a file
//! tracker.record_function("my_file.kleis", "factorial");
//! tracker.record_structure("my_file.kleis", "MyGroup");
//!
//! // Get all definitions from a file
//! let defs = tracker.get_definitions("my_file.kleis");
//!
//! // Get list of loaded files
//! let files = tracker.loaded_files();
//! ```

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Set of definitions originating from a single file
#[derive(Debug, Clone, Default)]
pub struct DefinitionSet {
    /// Function names defined in this file
    pub functions: HashSet<String>,

    /// Data type names defined in this file (e.g., "Bool", "Option")
    pub data_types: HashSet<String>,

    /// Structure names defined in this file (e.g., "Ring", "VectorSpace")
    pub structures: HashSet<String>,

    /// Implements blocks: (structure_name, type_string)
    /// e.g., ("Ring", "ℤ") for `implements Ring(ℤ)`
    pub implements: HashSet<(String, String)>,

    /// Type alias names defined in this file
    pub type_aliases: HashSet<String>,
}

impl DefinitionSet {
    /// Create an empty definition set
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this set is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.data_types.is_empty()
            && self.structures.is_empty()
            && self.implements.is_empty()
            && self.type_aliases.is_empty()
    }

    /// Get total count of all definitions
    pub fn total_count(&self) -> usize {
        self.functions.len()
            + self.data_types.len()
            + self.structures.len()
            + self.implements.len()
            + self.type_aliases.len()
    }
}

/// Tracks which definitions came from which files
#[derive(Debug, Clone, Default)]
pub struct ProvenanceTracker {
    /// Map from canonical file path to definitions from that file
    definitions_by_file: HashMap<PathBuf, DefinitionSet>,

    /// Reverse lookup: function name → source file
    function_sources: HashMap<String, PathBuf>,

    /// Reverse lookup: data type name → source file
    data_type_sources: HashMap<String, PathBuf>,

    /// Reverse lookup: structure name → source file
    structure_sources: HashMap<String, PathBuf>,

    /// Reverse lookup: type alias name → source file
    type_alias_sources: HashMap<String, PathBuf>,
}

impl ProvenanceTracker {
    /// Create a new provenance tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Canonicalize a path for consistent tracking
    fn canonical_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    /// Ensure a file entry exists
    fn ensure_file_entry(&mut self, path: &Path) -> PathBuf {
        let canonical = Self::canonical_path(path);
        self.definitions_by_file
            .entry(canonical.clone())
            .or_default();
        canonical
    }

    // === Recording Definitions ===

    /// Record a function as coming from a file
    pub fn record_function(&mut self, path: &Path, name: &str) {
        let canonical = self.ensure_file_entry(path);
        if let Some(defs) = self.definitions_by_file.get_mut(&canonical) {
            defs.functions.insert(name.to_string());
        }
        self.function_sources.insert(name.to_string(), canonical);
    }

    /// Record a data type as coming from a file
    pub fn record_data_type(&mut self, path: &Path, name: &str) {
        let canonical = self.ensure_file_entry(path);
        if let Some(defs) = self.definitions_by_file.get_mut(&canonical) {
            defs.data_types.insert(name.to_string());
        }
        self.data_type_sources.insert(name.to_string(), canonical);
    }

    /// Record a structure as coming from a file
    pub fn record_structure(&mut self, path: &Path, name: &str) {
        let canonical = self.ensure_file_entry(path);
        if let Some(defs) = self.definitions_by_file.get_mut(&canonical) {
            defs.structures.insert(name.to_string());
        }
        self.structure_sources.insert(name.to_string(), canonical);
    }

    /// Record an implements block as coming from a file
    pub fn record_implements(&mut self, path: &Path, structure_name: &str, type_str: &str) {
        let canonical = self.ensure_file_entry(path);
        if let Some(defs) = self.definitions_by_file.get_mut(&canonical) {
            defs.implements
                .insert((structure_name.to_string(), type_str.to_string()));
        }
    }

    /// Record a type alias as coming from a file
    pub fn record_type_alias(&mut self, path: &Path, name: &str) {
        let canonical = self.ensure_file_entry(path);
        if let Some(defs) = self.definitions_by_file.get_mut(&canonical) {
            defs.type_aliases.insert(name.to_string());
        }
        self.type_alias_sources.insert(name.to_string(), canonical);
    }

    // === Querying ===

    /// Get all definitions from a file
    pub fn get_definitions(&self, path: &Path) -> Option<&DefinitionSet> {
        let canonical = Self::canonical_path(path);
        self.definitions_by_file.get(&canonical)
    }

    /// Get list of all loaded files
    pub fn loaded_files(&self) -> Vec<&PathBuf> {
        self.definitions_by_file.keys().collect()
    }

    /// Check if a file is loaded
    pub fn is_file_loaded(&self, path: &Path) -> bool {
        let canonical = Self::canonical_path(path);
        self.definitions_by_file.contains_key(&canonical)
    }

    /// Get source file for a function
    pub fn function_source(&self, name: &str) -> Option<&PathBuf> {
        self.function_sources.get(name)
    }

    /// Get source file for a structure
    pub fn structure_source(&self, name: &str) -> Option<&PathBuf> {
        self.structure_sources.get(name)
    }

    /// Get source file for a data type
    pub fn data_type_source(&self, name: &str) -> Option<&PathBuf> {
        self.data_type_sources.get(name)
    }

    // === Unloading ===

    /// Get definitions to remove for a file (for unloading)
    /// Returns the DefinitionSet if the file was loaded, None otherwise
    pub fn prepare_unload(&self, path: &Path) -> Option<DefinitionSet> {
        let canonical = Self::canonical_path(path);
        self.definitions_by_file.get(&canonical).cloned()
    }

    /// Remove tracking for a file (call after actually unloading definitions)
    pub fn remove_file(&mut self, path: &Path) {
        let canonical = Self::canonical_path(path);

        if let Some(defs) = self.definitions_by_file.remove(&canonical) {
            // Remove reverse lookups
            for func in &defs.functions {
                self.function_sources.remove(func);
            }
            for dt in &defs.data_types {
                self.data_type_sources.remove(dt);
            }
            for s in &defs.structures {
                self.structure_sources.remove(s);
            }
            for ta in &defs.type_aliases {
                self.type_alias_sources.remove(ta);
            }
        }
    }

    /// Clear all tracking (for :reset)
    pub fn reset(&mut self) {
        self.definitions_by_file.clear();
        self.function_sources.clear();
        self.data_type_sources.clear();
        self.structure_sources.clear();
        self.type_alias_sources.clear();
    }

    // === Statistics ===

    /// Get total number of loaded files
    pub fn file_count(&self) -> usize {
        self.definitions_by_file.len()
    }

    /// Get total number of definitions across all files
    pub fn total_definition_count(&self) -> usize {
        self.definitions_by_file
            .values()
            .map(|d| d.total_count())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query() {
        let mut tracker = ProvenanceTracker::new();
        let path = Path::new("test.kleis");

        tracker.record_function(path, "foo");
        tracker.record_function(path, "bar");
        tracker.record_structure(path, "MyGroup");

        let defs = tracker.get_definitions(path).unwrap();
        assert!(defs.functions.contains("foo"));
        assert!(defs.functions.contains("bar"));
        assert!(defs.structures.contains("MyGroup"));
        assert_eq!(defs.total_count(), 3);
    }

    #[test]
    fn test_loaded_files() {
        let mut tracker = ProvenanceTracker::new();

        tracker.record_function(Path::new("a.kleis"), "func_a");
        tracker.record_function(Path::new("b.kleis"), "func_b");

        let files = tracker.loaded_files();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_source_lookup() {
        let mut tracker = ProvenanceTracker::new();
        let path = Path::new("myfile.kleis");

        tracker.record_function(path, "my_func");

        assert!(tracker.function_source("my_func").is_some());
        assert!(tracker.function_source("unknown").is_none());
    }

    #[test]
    fn test_prepare_unload() {
        let mut tracker = ProvenanceTracker::new();
        let path = Path::new("test.kleis");

        tracker.record_function(path, "foo");
        tracker.record_structure(path, "Bar");

        let to_unload = tracker.prepare_unload(path).unwrap();
        assert!(to_unload.functions.contains("foo"));
        assert!(to_unload.structures.contains("Bar"));
    }

    #[test]
    fn test_remove_file() {
        let mut tracker = ProvenanceTracker::new();
        let path = Path::new("test.kleis");

        tracker.record_function(path, "foo");
        assert!(tracker.function_source("foo").is_some());

        tracker.remove_file(path);
        assert!(tracker.function_source("foo").is_none());
        assert!(!tracker.is_file_loaded(path));
    }

    #[test]
    fn test_reset() {
        let mut tracker = ProvenanceTracker::new();

        tracker.record_function(Path::new("a.kleis"), "func_a");
        tracker.record_function(Path::new("b.kleis"), "func_b");

        tracker.reset();

        assert_eq!(tracker.file_count(), 0);
        assert_eq!(tracker.total_definition_count(), 0);
    }
}
