#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::math_layout::compile_math_to_svg_with_ids;

fn main() {
    let markup = "a";
    match compile_math_to_svg_with_ids(markup, &[], &[]) {
        Ok(output) => {
            println!("SVG for 'a':");
            println!("{}", output.svg);
            println!("\nText tag count: {}", output.svg.matches("<text").count());
        }
        Err(e) => println!("Error: {}", e),
    }
}
