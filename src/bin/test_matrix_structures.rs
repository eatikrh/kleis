///! Test parsing matrices.kleis structure definitions
use kleis::kleis_ast::TopLevel;
use kleis::kleis_parser::KleisParser;
use std::fs;

fn main() {
    println!("=== Testing Matrix Structures Parser ===\n");

    let content =
        fs::read_to_string("stdlib/matrices.kleis").expect("Failed to read stdlib/matrices.kleis");

    println!("File size: {} bytes\n", content.len());

    match KleisParser::new(&content).parse_program() {
        Ok(program) => {
            println!("✅ Parsed successfully!\n");

            let structures: Vec<_> = program
                .items
                .iter()
                .filter_map(|item| match item {
                    TopLevel::StructureDef(def) => Some(def),
                    _ => None,
                })
                .collect();

            let implements: Vec<_> = program
                .items
                .iter()
                .filter_map(|item| match item {
                    TopLevel::ImplementsDef(def) => Some(def),
                    _ => None,
                })
                .collect();

            let type_aliases: Vec<_> = program
                .items
                .iter()
                .filter_map(|item| match item {
                    TopLevel::TypeAlias(def) => Some(def),
                    _ => None,
                })
                .collect();

            println!("📊 Summary:");
            println!("  Structures: {}", structures.len());
            println!("  Implements: {}", implements.len());
            println!("  Type aliases: {}", type_aliases.len());
            println!();

            // Show structure details
            for def in structures {
                println!("Structure: {}", def.name);
                println!("  Type parameters: [");
                for param in &def.type_params {
                    if let Some(kind) = &param.kind {
                        println!("    {}: {}", param.name, kind);
                    } else {
                        println!("    {}", param.name);
                    }
                }
                println!("  ]");
                println!("  Members: {}", def.members.len());

                // Count operations
                let ops = def
                    .members
                    .iter()
                    .filter(|m| matches!(m, kleis::kleis_ast::StructureMember::Operation { .. }))
                    .count();
                println!("    Operations: {}", ops);

                // Count axioms
                let axioms = def
                    .members
                    .iter()
                    .filter(|m| matches!(m, kleis::kleis_ast::StructureMember::Axiom { .. }))
                    .count();
                println!("    Axioms: {}", axioms);
                println!();
            }

            // Show implements details
            for def in implements {
                println!("Implements: {}({:?})", def.structure_name, def.type_args);
                println!("  Members: {}", def.members.len());
                println!();
            }

            // Show type aliases
            for alias in type_aliases {
                println!("Type alias: {} = {:?}", alias.name, alias.type_expr);
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e.message);
            eprintln!("   Position: {}", e.position);

            // Show context around error
            let lines: Vec<&str> = content.lines().collect();
            let mut char_count = 0;
            for (line_num, line) in lines.iter().enumerate() {
                char_count += line.len() + 1; // +1 for newline
                if char_count >= e.position {
                    eprintln!("\nContext:");
                    if line_num > 0 {
                        eprintln!("  {} | {}", line_num, lines[line_num - 1]);
                    }
                    eprintln!("  {} | {}", line_num + 1, line);
                    eprintln!("  {} | ^", line_num + 1);
                    if line_num + 1 < lines.len() {
                        eprintln!("  {} | {}", line_num + 2, lines[line_num + 1]);
                    }
                    break;
                }
            }

            std::process::exit(1);
        }
    }
}
