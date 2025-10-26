#!/bin/bash
# scripts/setup.sh - Setup environment

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔═══════════════════════════════════════════════════════╗"
echo "║     NEXUS ORCHESTRATOR - ENVIRONMENT SETUP            ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""

# Create directories
echo "📁 Creating directories..."
mkdir -p "$PROJECT_ROOT/config/cache"
mkdir -p "$PROJECT_ROOT/logs"

# Create placeholder files
echo "📝 Creating placeholder files..."

if [ ! -f "$PROJECT_ROOT/config/tokens.txt" ]; then
    cat > "$PROJECT_ROOT/config/tokens.txt" << 'EOF'
# GitHub Personal Access Tokens (one per line)
# Format: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
# Minimum scopes: repo, workflow, codespace

EOF
    echo "  ✅ config/tokens.txt"
fi

if [ ! -f "$PROJECT_ROOT/config/proxies.txt" ]; then
    cat > "$PROJECT_ROOT/config/proxies.txt" << 'EOF'
# Proxies (one per line, 1:1 mapping with tokens)
# Format: http://user:pass@ip:port

EOF
    echo "  ✅ config/proxies.txt"
fi

if [ ! -f "$PROJECT_ROOT/config/nodes.txt" ]; then
    cat > "$PROJECT_ROOT/config/nodes.txt" << 'EOF'
# Nexus Node IDs (one per line)

EOF
    echo "  ✅ config/nodes.txt"
fi

if [ ! -f "$PROJECT_ROOT/config/wallets.txt" ]; then
    cat > "$PROJECT_ROOT/config/wallets.txt" << 'EOF'
# Wallet Addresses (one per line, matching order with nodes.txt)
# Format: 0xAbCdEf1234567890...

EOF
    echo "  ✅ config/wallets.txt"
fi

if [ ! -f "$PROJECT_ROOT/config/setup.json" ]; then
    cat > "$PROJECT_ROOT/config/setup.json" << 'EOF'
{
  "main_repo_owner": "your_github_username",
  "main_repo_name": "nexus-runner",
  "billing_warning_threshold": 118.0,
  "billing_critical_threshold": 119.5,
  "workflow_check_interval_minutes": 30,
  "max_parallel_nodes": 20
}
EOF
    echo "  ✅ config/setup.json"
fi

echo ""
echo "╔═══════════════════════════════════════════════════════╗"
echo "║     SETUP COMPLETE                                    ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""
echo "📋 Next Steps:"
echo "  1. Edit config/tokens.txt - Add GitHub PAT tokens"
echo "  2. Edit config/proxies.txt - Add proxies (1:1 with tokens)"
echo "  3. Edit config/nodes.txt - Add Nexus node IDs"
echo "  4. Edit config/wallets.txt - Add wallet addresses"
echo "  5. Edit config/setup.json - Configure main repo"
echo "  6. Run: cargo run --release"
echo ""
