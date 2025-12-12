#!/bin/bash
# Check all markdown links in the repository

echo "🔍 Checking all markdown links in repository..."
echo ""

REPO_ROOT="/Users/eatik_1/Documents/git/cee/kleis"
cd "$REPO_ROOT"

BROKEN_LINKS=0
TOTAL_LINKS=0

# Find all .md files (excluding node_modules and target)
find . -name "*.md" \
  -not -path "*/node_modules/*" \
  -not -path "*/target/*" \
  -not -path "*/.git/*" \
  | sort | while read -r mdfile; do
  
  # Extract markdown links: [text](path)
  # Only check relative paths (not http://, https://, etc.)
  grep -oE '\[([^\]]+)\]\(([^)]+)\)' "$mdfile" 2>/dev/null | \
    sed -E 's/\[([^\]]+)\]\(([^)]+)\)/\2/' | \
    grep -v '^http' | \
    grep -v '^#' | \
    while read -r link; do
    
    TOTAL_LINKS=$((TOTAL_LINKS + 1))
    
    # Get directory of the markdown file
    MD_DIR=$(dirname "$mdfile")
    
    # Resolve relative path
    if [[ "$link" == /* ]]; then
      # Absolute path from repo root
      FULL_PATH="$REPO_ROOT$link"
    else
      # Relative path from markdown file location
      FULL_PATH="$MD_DIR/$link"
    fi
    
    # Normalize path (remove ..)
    FULL_PATH=$(cd "$(dirname "$FULL_PATH")" 2>/dev/null && pwd)/$(basename "$FULL_PATH") 2>/dev/null
    
    # Check if file exists
    if [ ! -e "$FULL_PATH" ] && [ ! -d "$FULL_PATH" ]; then
      echo "❌ BROKEN: $mdfile"
      echo "   Link: $link"
      echo "   Resolved: $FULL_PATH"
      echo ""
      BROKEN_LINKS=$((BROKEN_LINKS + 1))
    fi
  done
done

echo ""
echo "📊 Summary:"
echo "Total links checked: $TOTAL_LINKS"
echo "Broken links: $BROKEN_LINKS"

if [ $BROKEN_LINKS -eq 0 ]; then
  echo "✅ All links are valid!"
  exit 0
else
  echo "⚠️  Found broken links - please fix them"
  exit 1
fi

