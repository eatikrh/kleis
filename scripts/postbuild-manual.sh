#!/usr/bin/env bash
# Post-build SEO fixes for mdBook output.
# Injects canonical URLs and per-page meta descriptions into static HTML.
# Run after: mdbook build docs/manual

set -euo pipefail

BOOK_DIR="${1:-docs/manual/book}"
SITE_BASE="https://kleis.io/docs/manual/book"

# Portable sed in-place: macOS requires -i '', GNU sed requires -i
sedi() {
    if sed --version 2>/dev/null | grep -q GNU; then
        sed -i "$@"
    else
        sed -i '' "$@"
    fi
}

if [ ! -d "$BOOK_DIR" ]; then
    echo "Error: book directory not found: $BOOK_DIR" >&2
    exit 1
fi

count=0

while IFS= read -r -d '' file; do
    rel="${file#"$BOOK_DIR"/}"

    # Canonical URL: normalize index.html and introduction.html (identical content) to directory form.
    # Strip .html suffix because Cloudflare Pages forces 308 redirects that remove it.
    # Canonical must match the URL Cloudflare actually serves, otherwise Google Search
    # Console reports redirect errors (known CF Pages issue since 2021, no server-side fix).
    if [[ "$rel" == "index.html" || "$rel" == "introduction.html" ]]; then
        canonical="${SITE_BASE}/"
    else
        canonical="${SITE_BASE}/${rel%.html}"
    fi

    # Extract page title for unique meta description
    title=$(sed -n 's/.*<title>\(.*\)<\/title>.*/\1/p' "$file" | head -1)
    short_title="${title% - The Kleis Manual}"

    # Per-page descriptions (150-160 chars for SEO)
    case "$rel" in
        index.html|introduction.html)
            desc="The Kleis Manual: learn formal verification, Hindley-Milner type inference, Z3 theorem proving, and universal knowledge production in one language.";;
        chapters/01-starting-out.html)
            desc="Get started with Kleis: install the language, write your first expressions, use the REPL, and understand basic types like Scalar, Bool, and String.";;
        chapters/03-functions.html)
            desc="Define and compose functions in Kleis with full type inference. Covers lambda expressions, higher-order functions, currying, and polymorphic signatures.";;
        chapters/06-let-bindings.html)
            desc="Use let-bindings in Kleis to define local variables, structure computations, and build complex expressions from simpler parts with type safety.";;
        chapters/08-conditionals.html)
            desc="Conditional expressions in Kleis: if-then-else with type-checked branches, Boolean logic, comparison operators, and Z3-verified guard conditions.";;
        chapters/13-applications.html)
            desc="Real-world Kleis applications: physics verification, business rules, authorization systems, cryptographic protocols, and domain-specific modeling.";;
        chapters/16-bit-vectors.html)
            desc="Bit-vector arithmetic in Kleis with Z3 backend: fixed-width integers, bitwise operations, overflow detection, and hardware verification examples.";;
        chapters/17-strings.html)
            desc="String operations in Kleis: concatenation, length, substring, regex matching, and Z3-backed verification of string properties and constraints.";;
        chapters/19-matrices.html)
            desc="Matrix algebra in Kleis: construction, arithmetic, determinants, eigenvalues, LAPACK integration, and type-safe dimensioned linear algebra.";;
        chapters/20-example-blocks.html)
            desc="Example blocks in Kleis: executable assertions, concrete evaluation, Z3 verification, and how to write self-testing mathematical specifications.";;
        chapters/31-equation-editor.html)
            desc="Kleis Equation Editor: visual WYSIWYG tool for building math expressions, matrices, tensors, and Egyptian hieroglyphs with live Typst rendering and Z3 verification.";;
        chapters/22-standard-library.html)
            desc="The Kleis standard library: built-in structures for groups, rings, fields, vector spaces, categories, and differential geometry with axiom verification.";;
        appendix/operators.html)
            desc="Complete Kleis operator reference: arithmetic, comparison, logical, set, and custom operators with precedence, associativity, and type signatures.";;
        *)
            if [ -z "$short_title" ] || [ "$short_title" = "$title" ]; then
                desc="The Kleis Manual: learn formal verification, Hindley-Milner type inference, Z3 theorem proving, and universal knowledge production in one language."
            else
                lower_title=$(echo "$short_title" | tr '[:upper:]' '[:lower:]')
                desc="${short_title} — Learn ${lower_title} in Kleis: formal verification with Z3, Hindley-Milner type inference, and universal mathematical specification."
            fi;;
    esac

    # Inject canonical link after the <meta charset> line
    if ! grep -q 'rel="canonical"' "$file"; then
        sedi "s|<meta charset=\"UTF-8\">|<meta charset=\"UTF-8\">\n        <link rel=\"canonical\" href=\"${canonical}\">|" "$file"
    fi

    # Replace generic meta description with page-specific one
    sedi "s|<meta name=\"description\" content=\"The official guide to the Kleis mathematical specification language\">|<meta name=\"description\" content=\"${desc}\">|" "$file"

    count=$((count + 1))
done < <(find "$BOOK_DIR" -name '*.html' -print0)

echo "SEO post-build: processed ${count} HTML files"
