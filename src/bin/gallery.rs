use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::Command;

fn main() {
    // Collect samples from the renderer
    let samples = kleis::render::collect_samples_for_gallery();

    // Write LaTeX document
    let tex_path = "tmp_gallery.tex";
    let pdf_path = "tmp_gallery.pdf";
    let mut f = BufWriter::new(File::create(tex_path).expect("create tmp_gallery.tex"));
    writeln!(f, "{}",
        "\\documentclass[11pt]{article}\n\\usepackage{amsmath,amssymb,bm}\n\\usepackage[margin=1in]{geometry}\n\\title{Kleis Render Gallery}\n\\date{}\n\\begin{document}\n\\maketitle\n").unwrap();

    for (title, latex) in samples {
        writeln!(f, "\\subsection*{{{}}}", title).unwrap();
        writeln!(f, "\\[ {} \\]", latex).unwrap();
    }
    writeln!(f, "{}", "\\end{document}\n").unwrap();
    drop(f);

    // Try to compile with pdflatex if available
    let pdflatex = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(tex_path)
        .output();

    match pdflatex {
        Ok(out) => {
            if !out.status.success() {
                eprintln!(
                    "pdflatex failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
                    out.status,
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
            } else {
                println!("Gallery PDF generated at: {}", pdf_path);
            }
        }
        Err(err) => {
            eprintln!(
                "Could not run pdflatex ({}). You can compile {} manually.",
                err, tex_path
            );
        }
    }
}
