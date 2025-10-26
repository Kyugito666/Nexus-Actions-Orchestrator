#!/bin/bash
# scripts/cleanup.sh - Cleanup script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔═══════════════════════════════════════════════════════╗"
echo "║     NEXUS ORCHESTRATOR - CLEANUP                      ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""

read -p "⚠️  This will delete all cache files. Continue? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo "🧹 Cleaning cache directory..."
rm -rf "$PROJECT_ROOT/config/cache/"*
echo "  ✅ Cache cleaned"

echo "📝 Cleaning logs..."
rm -f "$PROJECT_ROOT/logs/"*.log
echo "  ✅ Logs cleaned"

echo ""
echo "✅ Cleanup complete!"
