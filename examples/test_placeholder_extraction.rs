#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test placeholder extraction regex

fn main() {
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"600\" height=\"200\" viewBox=\"0 0 600 200\">\n  <rect width=\"600\" height=\"200\" fill=\"#fafafa\"/>\n  <text x=\"50\" y=\"80\" font-family=\"Latin Modern Math, Times New Roman, serif\" font-size=\"32\" fill=\"black\">(⟨⟨PH0⟩⟩)/(⟨⟨PH1⟩⟩)</text>\n  <text x=\"20\" y=\"180\" font-size=\"14\" fill=\"gray\">Mock rendering - Typst library integration pending</text>\n</svg>";

    println!("SVG content:");
    println!("{}", svg);
    println!();

    println!("Looking for markers...");
    let marker_pattern = regex::Regex::new(r"⟨⟨PH(\d+)⟩⟩").unwrap();

    let matches: Vec<_> = marker_pattern.captures_iter(svg).collect();
    println!("Found {} matches", matches.len());

    for (i, cap) in matches.iter().enumerate() {
        if let Some(id_match) = cap.get(1) {
            println!("  Match {}: ID = {}", i + 1, id_match.as_str());
            println!("    Full match: {}", cap.get(0).unwrap().as_str());
        }
    }

    // Also try searching for the literal string
    println!();
    println!("Searching for literal '⟨⟨PH0⟩⟩':");
    if svg.contains("⟨⟨PH0⟩⟩") {
        println!("  ✅ Found!");
        let pos = svg.find("⟨⟨PH0⟩⟩").unwrap();
        println!("  Position: {}", pos);
    } else {
        println!("  ❌ Not found!");
    }

    // Print bytes to check encoding
    println!();
    println!("Checking bytes of marker:");
    let marker = "⟨⟨PH0⟩⟩";
    println!("  String: {}", marker);
    println!("  Bytes: {:?}", marker.as_bytes());
    println!("  Chars: {:?}", marker.chars().collect::<Vec<_>>());
}
