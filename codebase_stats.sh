#!/bin/bash

echo "═══════════════════════════════════════════════════════"
echo "          CODEBASE ANALYSIS"
echo "═══════════════════════════════════════════════════════"
echo ""

# Check if cloc is installed
if ! command -v cloc &> /dev/null; then
    echo "❌ Error: cloc is not installed"
    echo "Install with: brew install cloc (macOS) or apt-get install cloc (Linux)"
    exit 1
fi

# Check if tree is installed
if ! command -v tree &> /dev/null; then
    echo "⚠️  Warning: tree is not installed"
    echo "Install with: brew install tree (macOS) or apt-get install tree (Linux)"
    TREE_AVAILABLE=false
else
    TREE_AVAILABLE=true
fi

echo "📊 Lines of Code Analysis (via cloc)"
echo "───────────────────────────────────────────────────────"
echo ""

# Run cloc - it respects .gitignore by default
cloc . --exclude-dir=target

echo ""
echo "═══════════════════════════════════════════════════════"
echo ""

if [ "$TREE_AVAILABLE" = true ]; then
    echo "📁 Directory Tree Structure"
    echo "───────────────────────────────────────────────────────"
    echo ""

    # Use tree with gitignore support
    # -I flag for additional patterns if needed
    if [ -f .gitignore ]; then
        tree -a -I '.git|target' --gitignore
    else
        tree -a -I '.git|target'
    fi
else
    echo "📁 Directory Tree Structure (using find)"
    echo "───────────────────────────────────────────────────────"
    echo ""
    find . -not -path '*/\.git/*' -not -path '*/target/*' -print | sed 's|[^/]*/| |g'
fi

echo ""
echo "═══════════════════════════════════════════════════════"
