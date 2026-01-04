#!/bin/bash
# Documentation Validation Script
# Checks documentation for consistency and guideline compliance

set -e

# ============================================================================
# Colors
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ============================================================================
# State
# ============================================================================

ERRORS=0
WARNINGS=0

# ============================================================================
# Helper Functions
# ============================================================================

print_success() {
  echo -e "   ${GREEN}✓${NC} $1"
}

print_error() {
  echo -e "   ${RED}✗${NC} $1"
  ERRORS=$((ERRORS + 1))
}

print_warning() {
  echo -e "   ${YELLOW}⚠${NC} $1"
  WARNINGS=$((WARNINGS + 1))
}

normalize_path() {
  local path="$1"

  # Try readlink -f (available on most Linux systems)
  if readlink -f . &> /dev/null 2>&1; then
    readlink -f "$path" 2>/dev/null || echo "$path"
  # Python fallback for other systems (macOS, Windows Git Bash)
  elif command -v python3 &> /dev/null; then
    python3 -c "import os; print(os.path.normpath('$path'))" 2>/dev/null || echo "$path"
  else
    # Manual normalization as last resort
    while [[ "$path" == *"/../"* ]]; do
      path=$(echo "$path" | sed 's|[^/]\+/\.\./||')
    done
    echo "$path" | sed 's|^\./||'
  fi
}

# ============================================================================
# Check Functions
# ============================================================================

check_markdownlint() {
  echo "1. Markdownlint Check"

  if markdownlint . 2>&1 > /dev/null; then
    print_success "All markdown files pass linting"
  else
    print_error "Markdownlint errors found"
  fi

  echo ""
}

check_metadata_blocks() {
  echo "2. Metadata Block Check"

  local missing=0

  # Check docs/ files
  while IFS= read -r file; do
    # Skip files that don't need metadata
    if [[ "$file" == *"/CHANGELOG.md" ]] || \
       [[ "$file" == *"/LICENSE.md" ]]; then
      continue
    fi

    if ! grep -q "^- \*\*Document type\*\*:" "$file"; then
      print_error "Missing metadata: $file"
      missing=1
    fi
  done < <(find docs -name "*.md" -type f)

  # Check root markdown files (except README.md which is for external users)
  for root_file in AGENTS.md CONTRIBUTING.md; do
    if [ -f "$root_file" ]; then
      if ! grep -q "^- \*\*Document type\*\*:" "$root_file"; then
        print_error "Missing metadata: $root_file"
        missing=1
      fi
    fi
  done

  if [ "$missing" -eq 0 ]; then
    print_success "All documentation files have metadata blocks"
  fi

  echo ""
}

check_document_types() {
  echo "3. Document Type Validity"

  local invalid=0
  local valid_types="Reference|Explanation|How-to guide|Tutorial"

  # Check docs/ files
  while IFS= read -r file; do
    if grep -q "^- \*\*Document type\*\*:" "$file"; then
      local type_line=$(grep "^- \*\*Document type\*\*:" "$file" | head -1)

      if ! echo "$type_line" | grep -qE "($valid_types)"; then
        print_warning "Non-standard type in $file"
        echo "      $type_line"
        invalid=1
      fi
    fi
  done < <(find docs -name "*.md" -type f)

  # Check root files
  for root_file in AGENTS.md CONTRIBUTING.md; do
    if [ -f "$root_file" ]; then
      if grep -q "^- \*\*Document type\*\*:" "$root_file"; then
        local type_line=$(grep "^- \*\*Document type\*\*:" "$root_file" | head -1)

        if ! echo "$type_line" | grep -qE "($valid_types)"; then
          print_warning "Non-standard type in $root_file"
          echo "      $type_line"
          invalid=1
        fi
      fi
    fi
  done

  if [ "$invalid" -eq 0 ]; then
    print_success "All document types are valid"
  fi

  echo ""
}

check_file_links() {
  local file="$1"
  local file_dir=$(dirname "$file")

  while IFS= read -r link; do
    # Skip external links, anchors, and absolute paths
    if [[ "$link" =~ ^https?:// ]] || \
       [[ "$link" =~ ^# ]] || \
       [[ "$link" =~ ^/ ]]; then
      continue
    fi

    # Skip non-markdown links
    if [[ ! "$link" =~ \.md($|#) ]]; then
      continue
    fi

    # Remove anchor fragments
    local clean_link="${link%%#*}"

    # Resolve relative path from file's directory
    local target
    if [[ "$file_dir" == "." ]]; then
      target="$clean_link"
    else
      target="$file_dir/$clean_link"
    fi

    # Normalize path
    target=$(normalize_path "$target")

    # Check if target exists
    if [ ! -f "$target" ]; then
      print_error "Broken link in $file: $link -> $target"
    fi
  done < <(grep -oP '\]\(\K[^)]+' "$file" 2>/dev/null || true)
}

check_internal_links() {
  echo "4. Internal Link Check"

  local initial_errors=$ERRORS

  # Check docs/ markdown files
  while IFS= read -r file; do
    check_file_links "$file"
  done < <(find docs -name "*.md" -type f)

  # Check root markdown files
  for root_file in README.md AGENTS.md CONTRIBUTING.md; do
    if [ -f "$root_file" ]; then
      check_file_links "$root_file"
    fi
  done

  # If no new errors were added, print success
  if [ "$ERRORS" -eq "$initial_errors" ]; then
    print_success "No broken internal markdown links detected"
  fi

  echo ""
}

print_summary() {
  echo "==================================="

  if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed${NC}"
    exit 0
  elif [ "$ERRORS" -eq 0 ]; then
    echo -e "${YELLOW}⚠ $WARNINGS warning(s)${NC}"
    exit 0
  else
    echo -e "${RED}✗ $ERRORS error(s), $WARNINGS warning(s)${NC}"
    exit 1
  fi
}

# ============================================================================
# Main
# ============================================================================

main() {
  echo "=== Oxidris Documentation Validation ==="
  echo ""

  check_markdownlint
  check_metadata_blocks
  check_document_types
  check_internal_links

  print_summary
}

main
