///! Kleis Documentation Generator
///!
///! Generates beautiful documentation from .kleis files
///!
///! Usage:
///!   cargo run --bin kleis_doc stdlib/minimal_prelude.kleis
///!   cargo run --bin kleis_doc stdlib/matrices.kleis --format html > matrices.html
use kleis::kleis_ast::{ImplementsDef, StructureDef, StructureMember, TopLevel};
use kleis::kleis_parser::KleisParser;
use kleis::render::{build_default_context, render_expression, RenderTarget};
use std::env;
use std::fs;

#[derive(Debug)]
enum OutputFormat {
    Markdown,
    Html,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <kleis-file> [--format markdown|html]", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} stdlib/minimal_prelude.kleis", args[0]);
        eprintln!(
            "  {} stdlib/matrices.kleis --format html > doc.html",
            args[0]
        );
        std::process::exit(1);
    }

    let file_path = &args[1];
    let mut format = OutputFormat::Markdown;

    // Parse format flag
    if args.len() >= 4 && args[2] == "--format" {
        format = match args[3].as_str() {
            "html" => OutputFormat::Html,
            "markdown" | "md" => OutputFormat::Markdown,
            other => {
                eprintln!("Unknown format: {}. Using markdown.", other);
                OutputFormat::Markdown
            }
        };
    }

    // Read file
    let content = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Error reading file {}: {}", file_path, e);
        std::process::exit(1);
    });

    // Parse
    let mut parser = KleisParser::new(&content);
    let program = parser.parse_program().unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    // Generate documentation
    match format {
        OutputFormat::Markdown => generate_markdown(&program, file_path),
        OutputFormat::Html => generate_html(&program, file_path),
    }
}

fn generate_markdown(program: &kleis::kleis_ast::Program, file_path: &str) {
    let ctx = build_default_context();

    // Extract filename for title
    let filename = file_path.split('/').last().unwrap_or("Kleis Documentation");

    println!("# {}\n", filename);
    println!("*Auto-generated documentation from Kleis source*\n");
    println!("---\n");

    let mut structure_count = 0;
    let mut implements_count = 0;

    for item in &program.items {
        match item {
            TopLevel::StructureDef(s) => {
                structure_count += 1;
                render_structure_markdown(s, &ctx);
            }
            TopLevel::ImplementsDef(i) => {
                implements_count += 1;
                render_implements_markdown(i, &ctx);
            }
            TopLevel::DataDef(d) => {
                println!("### Data Type: `{}`\n", d.name);
                if !d.type_params.is_empty() {
                    print!("**Type Parameters:** ");
                    for (i, param) in d.type_params.iter().enumerate() {
                        if i > 0 {
                            print!(", ");
                        }
                        print!("`{}`", param.name);
                        if let Some(ref kind) = param.kind {
                            print!(" : `{}`", kind);
                        }
                    }
                    println!("\n");
                }
                println!("**Variants:**");
                for variant in &d.variants {
                    println!("- `{}`", variant.name);
                }
                println!();
            }
            TopLevel::FunctionDef(f) => {
                println!("### Function: `{}`\n", f.name);
                println!();
            }
            _ => {}
        }
    }

    println!("\n---\n");
    println!(
        "*Summary: {} structures, {} implementations*\n",
        structure_count, implements_count
    );
}

fn render_structure_markdown(structure: &StructureDef, ctx: &kleis::render::GlyphContext) {
    println!("## Structure: `{}`\n", structure.name);

    // Type parameters
    if !structure.type_params.is_empty() {
        print!("**Type Parameters:** ");
        for (i, param) in structure.type_params.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("`{}`", param.name);
            if let Some(ref kind) = param.kind {
                print!(" : `{}`", kind);
            }
        }
        println!("\n");
    }

    // Extends clause
    if let Some(ref extends) = structure.extends_clause {
        println!("**Extends:** `{}`\n", format_type_expr(extends));
    }

    // Over clause
    if let Some(ref over) = structure.over_clause {
        println!("**Over:** `{}`\n", format_type_expr(over));
    }

    // Members
    let mut operations = vec![];
    let mut elements = vec![];
    let mut axioms = vec![];
    let mut nested = vec![];

    for member in &structure.members {
        match member {
            StructureMember::Operation {
                name,
                type_signature,
            } => operations.push((name.clone(), format_type_expr(type_signature))),
            StructureMember::Field { name, type_expr } => {
                elements.push((name.clone(), format_type_expr(type_expr)))
            }
            StructureMember::Axiom { name, proposition } => {
                let latex = render_expression(proposition, ctx, &RenderTarget::LaTeX);
                let unicode = render_expression(proposition, ctx, &RenderTarget::Unicode);
                axioms.push((name.clone(), latex, unicode));
            }
            StructureMember::NestedStructure { name, .. } => nested.push(name.clone()),
            _ => {}
        }
    }

    // Operations
    if !operations.is_empty() {
        println!("### Operations\n");
        for (name, sig) in operations {
            println!("- `{}` : `{}`", name, sig);
        }
        println!();
    }

    // Elements
    if !elements.is_empty() {
        println!("### Elements\n");
        for (name, ty) in elements {
            println!("- `{}` : `{}`", name, ty);
        }
        println!();
    }

    // Axioms
    if !axioms.is_empty() {
        println!("### Axioms\n");
        for (name, latex, unicode) in axioms {
            println!("**{}**", name);
            println!("- LaTeX: `{}`", latex);
            println!("- Unicode: {}", unicode);
            println!();
        }
    }

    // Nested structures
    if !nested.is_empty() {
        println!("### Nested Structures\n");
        for name in nested {
            println!("- `{}`", name);
        }
        println!();
    }

    println!("---\n");
}

fn render_implements_markdown(implements: &ImplementsDef, _ctx: &kleis::render::GlyphContext) {
    print!("## Implementation: `{}`", implements.structure_name);

    if !implements.type_args.is_empty() {
        print!("(");
        for (i, arg) in implements.type_args.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", format_type_expr(arg));
        }
        print!(")");
    }
    println!("\n");

    // Where clause
    if let Some(ref where_clause) = implements.where_clause {
        println!("**Where:**");
        for constraint in where_clause {
            let type_args_str = constraint
                .type_args
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            println!("- `{}`({})", constraint.structure_name, type_args_str);
        }
        println!();
    }

    // Members
    println!("### Implementations\n");
    for member in &implements.members {
        match member {
            kleis::kleis_ast::ImplMember::Operation { name, .. } => {
                println!("- Operation: `{}`", name);
            }
            kleis::kleis_ast::ImplMember::Element { name, .. } => {
                println!("- Element: `{}`", name);
            }
        }
    }

    println!("\n---\n");
}

fn generate_html(program: &kleis::kleis_ast::Program, file_path: &str) {
    let ctx = build_default_context();

    let filename = file_path.split('/').last().unwrap_or("Kleis Documentation");

    println!("<!DOCTYPE html>");
    println!("<html lang=\"en\">");
    println!("<head>");
    println!("  <meta charset=\"UTF-8\">");
    println!("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
    println!("  <title>{}</title>", filename);
    println!("  <script src=\"https://polyfill.io/v3/polyfill.min.js?features=es6\"></script>");
    println!(
        "  <script id=\"MathJax-script\" async src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\"></script>"
    );
    println!("  <style>");
    println!(
        "    body {{ font-family: system-ui, -apple-system, sans-serif; max-width: 900px; margin: 40px auto; padding: 20px; line-height: 1.6; }}"
    );
    println!(
        "    h1 {{ color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }}"
    );
    println!(
        "    h2 {{ color: #34495e; margin-top: 40px; border-bottom: 2px solid #95a5a6; padding-bottom: 8px; }}"
    );
    println!("    h3 {{ color: #7f8c8d; margin-top: 30px; }}");
    println!(
        "    code {{ background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: 'Courier New', monospace; }}"
    );
    println!(
        "    .axiom {{ background: #e8f4f8; padding: 15px; margin: 10px 0; border-left: 4px solid #3498db; border-radius: 4px; }}"
    );
    println!("    .axiom-name {{ font-weight: bold; color: #2980b9; margin-bottom: 8px; }}");
    println!("    .math-display {{ margin: 10px 0; overflow-x: auto; }}");
    println!(
        "    .structure {{ background: #f9f9f9; padding: 20px; margin: 20px 0; border-radius: 8px; }}"
    );
    println!(
        "    .badge {{ display: inline-block; background: #3498db; color: white; padding: 3px 8px; border-radius: 3px; font-size: 0.85em; margin: 2px; }}"
    );
    println!(
        "    .summary {{ background: #ecf0f1; padding: 15px; border-radius: 5px; margin-top: 30px; }}"
    );
    println!("  </style>");
    println!("</head>");
    println!("<body>");

    println!("<h1>{}</h1>", filename);
    println!("<p><em>Auto-generated documentation from Kleis source</em></p>");

    let mut structure_count = 0;
    let mut implements_count = 0;

    for item in &program.items {
        match item {
            TopLevel::StructureDef(s) => {
                structure_count += 1;
                render_structure_html(s, &ctx);
            }
            TopLevel::ImplementsDef(i) => {
                implements_count += 1;
                render_implements_html(i, &ctx);
            }
            _ => {}
        }
    }

    println!("<div class=\"summary\">");
    println!(
        "<strong>Summary:</strong> {} structures, {} implementations",
        structure_count, implements_count
    );
    println!("</div>");

    println!("</body>");
    println!("</html>");
}

fn render_structure_html(structure: &StructureDef, ctx: &kleis::render::GlyphContext) {
    println!("<div class=\"structure\">");
    println!("<h2>Structure: <code>{}</code></h2>", structure.name);

    if let Some(ref extends) = structure.extends_clause {
        println!(
            "<p><span class=\"badge\">Extends</span> <code>{}</code></p>",
            escape_html(&format_type_expr(extends))
        );
    }

    if let Some(ref over) = structure.over_clause {
        println!(
            "<p><span class=\"badge\">Over</span> <code>{}</code></p>",
            escape_html(&format_type_expr(over))
        );
    }

    // Collect members
    let mut operations = vec![];
    let mut elements = vec![];
    let mut axioms = vec![];

    for member in &structure.members {
        match member {
            StructureMember::Operation {
                name,
                type_signature,
            } => operations.push((name.clone(), format_type_expr(type_signature))),
            StructureMember::Field { name, type_expr } => {
                elements.push((name.clone(), format_type_expr(type_expr)))
            }
            StructureMember::Axiom { name, proposition } => {
                let latex = render_expression(proposition, ctx, &RenderTarget::LaTeX);
                let unicode = render_expression(proposition, ctx, &RenderTarget::Unicode);
                axioms.push((name.clone(), latex, unicode));
            }
            _ => {}
        }
    }

    if !operations.is_empty() {
        println!("<h3>Operations</h3>");
        println!("<ul>");
        for (name, sig) in operations {
            println!(
                "<li><code>{}</code> : <code>{}</code></li>",
                escape_html(&name),
                escape_html(&sig)
            );
        }
        println!("</ul>");
    }

    if !elements.is_empty() {
        println!("<h3>Elements</h3>");
        println!("<ul>");
        for (name, ty) in elements {
            println!(
                "<li><code>{}</code> : <code>{}</code></li>",
                escape_html(&name),
                escape_html(&ty)
            );
        }
        println!("</ul>");
    }

    if !axioms.is_empty() {
        println!("<h3>Axioms</h3>");
        for (name, latex, _unicode) in axioms {
            println!("<div class=\"axiom\">");
            println!("<div class=\"axiom-name\">{}</div>", escape_html(&name));
            println!("<div class=\"math-display\">\\[{}\\]</div>", latex);
            println!("</div>");
        }
    }

    println!("</div>");
}

fn render_implements_html(implements: &ImplementsDef, _ctx: &kleis::render::GlyphContext) {
    println!("<div class=\"structure\">");
    print!("<h2>Implementation: <code>{}", implements.structure_name);

    if !implements.type_args.is_empty() {
        print!("(");
        for (i, arg) in implements.type_args.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", escape_html(&format_type_expr(arg)));
        }
        print!(")");
    }
    println!("</code></h2>");

    if let Some(ref where_clause) = implements.where_clause {
        println!("<p><span class=\"badge\">Where</span>");
        for constraint in where_clause {
            let type_args_str = constraint
                .type_args
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                " <code>{}({})</code>",
                escape_html(&constraint.structure_name),
                escape_html(&type_args_str)
            );
        }
        println!("</p>");
    }

    println!("</div>");
}

fn format_type_expr(type_expr: &kleis::kleis_ast::TypeExpr) -> String {
    match type_expr {
        kleis::kleis_ast::TypeExpr::Named(name) => name.clone(),
        kleis::kleis_ast::TypeExpr::Parametric(name, args) => {
            let args_str = args
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, args_str)
        }
        kleis::kleis_ast::TypeExpr::Function(from, to) => {
            format!("{} → {}", format_type_expr(from), format_type_expr(to))
        }
        kleis::kleis_ast::TypeExpr::Var(name) => name.clone(),
        _ => "?".to_string(),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
