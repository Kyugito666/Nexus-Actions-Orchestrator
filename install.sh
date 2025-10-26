#!/bin/bash
# install.sh - One-command installation

set -euo pipefail

echo "╔═══════════════════════════════════════════════════════╗"
echo "║     NEXUS GITHUB ORCHESTRATOR - INSTALLER             ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found!"
    echo ""
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "✅ Rust found: $(rustc --version)"

# Install libsodium
echo ""
echo "📦 Installing libsodium..."

if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y libsodium-dev pkg-config build-essential
elif command -v brew &> /dev/null; then
    brew install libsodium pkg-config
elif command -v pacman &> /dev/null; then
    sudo pacman -S libsodium pkg-config base-devel
else
    echo "⚠️  Please install libsodium manually"
fi

# Install gh CLI
echo ""
echo "📦 Installing GitHub CLI..."

if ! command -v gh &> /dev/null; then
    if command -v apt-get &> /dev/null; then
        curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
        sudo apt-get update
        sudo apt-get install -y gh
    elif command -v brew &> /dev/null; then
        brew install gh
    fi
fi

echo "✅ GitHub CLI: $(gh --version | head -n1)"

# Setup config
echo ""
echo "📁 Setting up configuration..."
bash scripts/setup.sh

# Build project
echo ""
echo "🔨 Building project..."
cargo build --release

echo ""
echo "╔═══════════════════════════════════════════════════════╗"
echo "║     INSTALLATION COMPLETE                             ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  1. Edit config files (tokens.txt, proxies.txt, etc)"
echo "  2. Run: ./start.sh"
echo ""
