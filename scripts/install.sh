#!/bin/bash
# scripts/install.sh - Install Nexus CLI (used in workflow)

set -euo pipefail

echo "🔧 Installing Nexus CLI..."

# Download official installer
INSTALLER_URL="https://cli.nexus.xyz/install.sh"

if ! command -v curl &> /dev/null; then
    echo "❌ curl not found, trying wget..."
    if ! command -v wget &> /dev/null; then
        echo "❌ Neither curl nor wget found!"
        exit 1
    fi
    wget -O /tmp/nexus_install.sh "$INSTALLER_URL"
else
    curl -fsSL "$INSTALLER_URL" -o /tmp/nexus_install.sh
fi

chmod +x /tmp/nexus_install.sh

# Run installer non-interactively
NONINTERACTIVE=1 /tmp/nexus_install.sh

rm -f /tmp/nexus_install.sh

# Verify installation
if command -v nexus-cli &> /dev/null; then
    echo "✅ Nexus CLI installed successfully"
    nexus-cli --version || echo "Version: latest"
else
    echo "❌ Installation failed"
    exit 1
fi
