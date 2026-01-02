//! Kleis CLI - Parse, verify, and evaluate .kleis files
//!
//! Usage:
//!   kleis <file.kleis>           Parse and load file, report results
//!   kleis --check <file.kleis>   Parse only, check for syntax errors
//!   kleis --help                 Show help
//!
//! Example:
//!   cargo run -- examples/security/sql_injection_detection.kleis

use kleis::evaluator::Evaluator;
use kleis::kleis_ast::{Program, TopLevel};
use kleis::kleis_parser::{parse_kleis_program, parse_kleis_program_with_file};
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help(&args[0]);
            ExitCode::SUCCESS
        }
        "--version" | "-V" => {
            println!("kleis {}", VERSION);
            ExitCode::SUCCESS
        }
        "--check" | "-c" => {
            if args.len() < 3 {
                eprintln!("❌ Error: --check requires a file path");
                print_usage(&args[0]);
                ExitCode::from(1)
            } else {
                check_file(&args[2])
            }
        }
        path if path.starts_with('-') => {
            eprintln!("❌ Unknown option: {}", path);
            print_usage(&args[0]);
            ExitCode::from(1)
        }
        path => run_file(path),
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} <file.kleis>", program);
    eprintln!("       {} --check <file.kleis>", program);
    eprintln!("       {} --help", program);
}

fn print_help(program: &str) {
    println!("🧮 Kleis v{} - Symbolic Mathematics Language", VERSION);
    println!();
    println!("USAGE:");
    println!("    {} <file.kleis>           Parse and load file", program);
    println!("    {} --check <file.kleis>   Check syntax only", program);
    println!("    {} --help                 Show this help", program);
    println!("    {} --version              Show version", program);
    println!();
    println!("EXAMPLES:");
    println!("    {} stdlib/text.kleis", program);
    println!(
        "    {} --check examples/security/sql_injection_detection.kleis",
        program
    );
    println!();
    println!("For interactive use, run: cargo run --bin repl");
}

/// Check file for syntax errors only (no evaluation)
fn check_file(path: &str) -> ExitCode {
    println!("📄 Checking {}...", path);

    let mut loaded_files: HashSet<PathBuf> = HashSet::new();
    let base_path = Path::new(path);

    match check_file_recursive(base_path, &mut loaded_files) {
        Ok(stats) => {
            println!();
            println!("✅ Syntax OK");
            println!();
            print_stats(&stats);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ {}", e);
            ExitCode::from(1)
        }
    }
}

/// Run file - parse, load, and report
fn run_file(path: &str) -> ExitCode {
    println!("📄 Loading {}...", path);

    let mut evaluator = Evaluator::new();
    let mut loaded_files: HashSet<PathBuf> = HashSet::new();
    let mut imported_paths: Vec<String> = Vec::new();
    let base_path = Path::new(path);

    match load_file_recursive(
        base_path,
        &mut evaluator,
        &mut loaded_files,
        &mut imported_paths,
    ) {
        Ok(stats) => {
            println!();
            println!("✅ Loaded successfully");
            println!();
            print_stats(&stats);

            // Show what's available
            let functions = evaluator.list_functions();
            if !functions.is_empty() {
                println!();
                println!("📚 Functions defined:");
                for name in functions.iter().take(10) {
                    println!("   • {}", name);
                }
                if functions.len() > 10 {
                    println!("   ... and {} more", functions.len() - 10);
                }
            }

            println!();
            println!(
                "💡 Tip: Use 'cargo run --bin repl' then ':load {}' for interactive use",
                path
            );

            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ {}", e);
            ExitCode::from(1)
        }
    }
}

/// Stats for reporting what was loaded
struct LoadStats {
    files: usize,
    functions: usize,
    structures: usize,
    data_types: usize,
    type_aliases: usize,
    imports: usize,
}

impl LoadStats {
    fn new() -> Self {
        LoadStats {
            files: 0,
            functions: 0,
            structures: 0,
            data_types: 0,
            type_aliases: 0,
            imports: 0,
        }
    }

    fn add(&mut self, other: &LoadStats) {
        self.files += other.files;
        self.functions += other.functions;
        self.structures += other.structures;
        self.data_types += other.data_types;
        self.type_aliases += other.type_aliases;
        self.imports += other.imports;
    }

    fn from_program(program: &Program) -> Self {
        let mut stats = LoadStats::new();
        stats.files = 1;
        stats.functions = program.functions().len();
        stats.structures = program.structures().len();
        stats.data_types = program.data_types().len();
        stats.type_aliases = program.type_aliases().len();
        stats.imports = program
            .items
            .iter()
            .filter(|item| matches!(item, TopLevel::Import(_)))
            .count();
        stats
    }
}

fn print_stats(stats: &LoadStats) {
    println!("📊 Summary:");
    println!("   Files:       {}", stats.files);
    println!("   Functions:   {}", stats.functions);
    println!("   Structures:  {}", stats.structures);
    println!("   Data types:  {}", stats.data_types);
    println!("   Type aliases:{}", stats.type_aliases);
    if stats.imports > 0 {
        println!("   Imports:     {}", stats.imports);
    }
}

/// Check file for syntax errors (parse only, no evaluation)
fn check_file_recursive(
    path: &Path,
    loaded_files: &mut HashSet<PathBuf>,
) -> Result<LoadStats, String> {
    // Resolve to canonical path for circular import detection
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    // Check for circular imports
    if loaded_files.contains(&canonical) {
        return Ok(LoadStats::new());
    }
    loaded_files.insert(canonical.clone());

    // Read file contents
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("File error '{}': {}", path.display(), e))?;

    // Parse with canonicalized file path for VS Code debugging support
    // The `canonical` variable is already computed above for cycle detection
    let file_path_str = canonical.to_string_lossy().to_string();
    let program = parse_kleis_program_with_file(&contents, &file_path_str).map_err(|e| {
        format!(
            "Parse error in '{}':\n   {}",
            path.display(),
            format_parse_error(&e, &contents)
        )
    })?;

    let mut stats = LoadStats::from_program(&program);

    // Get the directory containing this file for resolving relative imports (use canonical path)
    let base_dir = canonical.parent().unwrap_or(Path::new("."));

    // Process imports (depth-first)
    for item in &program.items {
        if let TopLevel::Import(import_path) = item {
            let resolved_path = resolve_import_path(import_path, base_dir);
            match check_file_recursive(&resolved_path, loaded_files) {
                Ok(import_stats) => {
                    stats.add(&import_stats);
                }
                Err(e) => {
                    return Err(format!(
                        "Error in import '{}' from '{}':\n   {}",
                        import_path,
                        path.display(),
                        e
                    ));
                }
            }
        }
    }

    Ok(stats)
}

/// Load file and its imports into evaluator
fn load_file_recursive(
    path: &Path,
    evaluator: &mut Evaluator,
    loaded_files: &mut HashSet<PathBuf>,
    imported_paths: &mut Vec<String>,
) -> Result<LoadStats, String> {
    // Resolve to canonical path for circular import detection
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    // Check for circular imports
    if loaded_files.contains(&canonical) {
        return Ok(LoadStats::new());
    }
    loaded_files.insert(canonical.clone());

    // Read file contents
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("File error '{}': {}", path.display(), e))?;

    // Parse with canonicalized file path for VS Code debugging support
    let file_path_str = canonical.to_string_lossy().to_string();
    let program = parse_kleis_program_with_file(&contents, &file_path_str).map_err(|e| {
        format!(
            "Parse error in '{}':\n   {}",
            path.display(),
            format_parse_error(&e, &contents)
        )
    })?;

    let mut stats = LoadStats::from_program(&program);

    // Get the directory containing this file for resolving relative imports (use canonical path)
    let base_dir = canonical.parent().unwrap_or(Path::new("."));

    // Process imports first (depth-first)
    for item in &program.items {
        if let TopLevel::Import(import_path) = item {
            // Track this import path
            if !imported_paths.contains(import_path) {
                imported_paths.push(import_path.clone());
            }

            let resolved_path = resolve_import_path(import_path, base_dir);
            match load_file_recursive(&resolved_path, evaluator, loaded_files, imported_paths) {
                Ok(import_stats) => {
                    stats.add(&import_stats);
                }
                Err(e) => {
                    return Err(format!(
                        "Error loading import '{}' from '{}':\n   {}",
                        import_path,
                        path.display(),
                        e
                    ));
                }
            }
        }
    }

    // Load definitions into evaluator
    if let Err(e) = evaluator.load_program(&program) {
        return Err(format!(
            "Error loading definitions from '{}':\n   {}",
            path.display(),
            e
        ));
    }

    Ok(stats)
}

/// Resolve an import path relative to the base directory
///
/// For stdlib imports, checks KLEIS_ROOT environment variable first.
fn resolve_import_path(import_path: &str, base_dir: &Path) -> PathBuf {
    let import = Path::new(import_path);

    if import.is_absolute() {
        import.to_path_buf()
    } else if import_path.starts_with("stdlib/") {
        // Standard library: check KLEIS_ROOT first
        if let Ok(kleis_root) = std::env::var("KLEIS_ROOT") {
            let candidate = PathBuf::from(&kleis_root).join(import_path);
            if candidate.exists() {
                return candidate;
            }
        }
        // Fallback to relative path
        PathBuf::from(import_path)
    } else {
        // Relative import: resolve from the importing file's directory
        base_dir.join(import)
    }
}

/// Format a parse error with line number context
fn format_parse_error(error: &kleis::kleis_parser::KleisParseError, source: &str) -> String {
    let pos = error.position;
    let lines: Vec<&str> = source.lines().collect();

    // Find line number and column
    let mut line_num = 1;
    let mut col = 1;
    let mut char_count = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_len = line.len() + 1; // +1 for newline
        if char_count + line_len > pos {
            line_num = i + 1;
            col = pos - char_count + 1;
            break;
        }
        char_count += line_len;
    }

    let mut result = format!("Line {}, column {}: {}", line_num, col, error.message);

    // Show the offending line if available
    if line_num > 0 && line_num <= lines.len() {
        let line = lines[line_num - 1];
        result.push_str(&format!("\n\n   {} | {}", line_num, line));
        result.push_str(&format!(
            "\n   {} | {}^",
            " ".repeat(line_num.to_string().len()),
            " ".repeat(col.saturating_sub(1))
        ));
    }

    result
}
