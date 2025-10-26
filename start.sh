#!/bin/bash
# start.sh - Linux/macOS launcher

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================"
echo "  NEXUS GITHUB ORCHESTRATOR v2.0"
echo "  Multi-Account GitHub Actions Runner"
echo "========================================"
echo ""

# Check Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: Rust/Cargo not found!"
    echo ""
    echo "Install from: https://rustup.rs/"
    echo ""
    exit 1
fi

# Check config
if [ ! -f "config/tokens.txt" ]; then
    echo "⚠️ WARNING: config/tokens.txt not found!"
    echo ""
    echo "Run: bash scripts/setup.sh first"
    echo ""
    exit 1
fi

echo "🚀 Starting orchestrator..."
echo ""

cargo run --release

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ ERROR: Orchestrator exited with error"
    read -p "Press Enter to exit..."
fi
```

### 37. `config/.gitkeep`
```
# Keep config directory in git
