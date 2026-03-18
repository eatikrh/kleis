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

    # Canonical URL: normalize index.html and introduction.html (identical content) to directory form
    if [[ "$rel" == "index.html" || "$rel" == "introduction.html" ]]; then
        canonical="${SITE_BASE}/"
    else
        canonical="${SITE_BASE}/${rel}"
    fi

    # Extract page title for unique meta description
    title=$(sed -n 's/.*<title>\(.*\)<\/title>.*/\1/p' "$file" | head -1)
    short_title="${title% - The Kleis Manual}"

    if [ -z "$short_title" ] || [ "$short_title" = "$title" ]; then
        desc="The official guide to the Kleis mathematical specification language"
    else
        desc="${short_title} — Kleis language guide: formal verification, type inference, and Z3 theorem proving."
    fi

    # Inject canonical link after the <meta charset> line
    if ! grep -q 'rel="canonical"' "$file"; then
        sedi "s|<meta charset=\"UTF-8\">|<meta charset=\"UTF-8\">\n        <link rel=\"canonical\" href=\"${canonical}\">|" "$file"
    fi

    # Replace generic meta description with page-specific one
    sedi "s|<meta name=\"description\" content=\"The official guide to the Kleis mathematical specification language\">|<meta name=\"description\" content=\"${desc}\">|" "$file"

    count=$((count + 1))
done < <(find "$BOOK_DIR" -name '*.html' -print0)

echo "SEO post-build: processed ${count} HTML files"
