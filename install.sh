#!/usr/bin/env sh

set -e

REPO="LaneSun/jjit"
INSTALL_DIR=""

# Detect OS
OS=$(uname -s)
case "$OS" in
    Linux*)     OS_TYPE="linux";;
    Darwin*)    OS_TYPE="macos";;
    *)          echo "Unsupported OS: $OS"; exit 1;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)     ARCH_TYPE="x86_64";;
    amd64)      ARCH_TYPE="x86_64";;
    arm64)      ARCH_TYPE="aarch64";;
    aarch64)    ARCH_TYPE="aarch64";;
    *)          echo "Unsupported architecture: $ARCH"; exit 1;;
esac

# Build target string
if [ "$OS_TYPE" = "linux" ]; then
    TARGET="${ARCH_TYPE}-unknown-linux-gnu"
else
    TARGET="${ARCH_TYPE}-apple-darwin"
fi

# Determine install directory
if [ -w /usr/local/bin ] 2>/dev/null || [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Check if jjit is already installed
if command -v jjit >/dev/null 2>&1; then
    EXISTING_PATH=$(command -v jjit)
    echo "jjit is already installed at: $EXISTING_PATH"
    
    # Check version
    if jjit --version >/dev/null 2>&1; then
        EXISTING_VERSION=$(jjit --version)
        echo "Current version: $EXISTING_VERSION"
    fi
    
    printf "Do you want to reinstall/overwrite? [y/N] "
    read -r REPLY
    case "$REPLY" in
        y|Y|yes|YES) echo "Proceeding with installation...";;
        *) echo "Installation cancelled."; exit 0;;
    esac
fi

# Create temp directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Download URL
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/jjit-${TARGET}.tar.gz"

echo "Detected platform: ${OS_TYPE} ${ARCH_TYPE}"
echo "Downloading jjit from GitHub releases..."
echo "  URL: ${DOWNLOAD_URL}"

# Download
cd "$TMP_DIR"
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o jjit.tar.gz
elif command -v wget >/dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O jjit.tar.gz
else
    echo "Error: Neither curl nor wget is installed."
    exit 1
fi

# Extract
echo "Extracting..."
tar xzf jjit.tar.gz

# Check if binary exists
if [ ! -f "jjit" ]; then
    echo "Error: Expected binary 'jjit' not found in archive."
    exit 1
fi

# Install
if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ ! -w "$INSTALL_DIR" ]; then
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv jjit "$INSTALL_DIR/"
    sudo chmod +x "$INSTALL_DIR/jjit"
else
    echo "Installing to ${INSTALL_DIR}..."
    mv jjit "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/jjit"
fi

# Verify installation
if command -v jjit >/dev/null 2>&1; then
    echo ""
    echo "Successfully installed jjit!"
    jjit --version
else
    echo ""
    echo "jjit installed to: ${INSTALL_DIR}/jjit"
    
    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) 
            echo "But 'jjit' command not found. Please restart your terminal or run:"
            echo "  source ~/.bashrc"
            ;;
        *)
            echo ""
            echo "WARNING: ${INSTALL_DIR} is not in your PATH."
            echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            echo ""
            echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "Then run: source ~/.bashrc  (or ~/.zshrc)"
            ;;
    esac
fi

echo ""
echo "Next steps:"
echo "  1. Set up your API key:"
echo "     jjit config set api_key sk-your-api-key-here"
echo ""
echo "  2. Or use environment variable:"
echo "     export DEEPSEEK_API_KEY=sk-your-api-key-here"
echo ""
echo "  3. Try auto-commit:"
echo "     jjit commit"
