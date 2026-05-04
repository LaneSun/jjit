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

# Function to get latest version from GitHub API
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$api_url" 2>/dev/null | grep -o '"tag_name": *"v[^"]*"' | sed 's/.*"v\([^"]*\)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$api_url" 2>/dev/null | grep -o '"tag_name": *"v[^"]*"' | sed 's/.*"v\([^"]*\)".*/\1/'
    fi
}

# Function to compare versions (returns 0 if v1 < v2)
version_lt() {
    # Simple version comparison: split by dots and compare numerically
    if [ "$1" = "$2" ]; then
        return 1  # Equal is not less than
    fi
    printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -n1 | grep -qx "$1"
}

# Check if jjit is already installed
SHOULD_INSTALL=1
if command -v jjit >/dev/null 2>&1; then
    EXISTING_PATH=$(command -v jjit)
    echo "jjit is already installed at: $EXISTING_PATH"
    
    # Get installed version
    INSTALLED_VERSION=""
    if jjit --version >/dev/null 2>&1; then
        INSTALLED_VERSION=$(jjit --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -n1)
        echo "Installed version: v${INSTALLED_VERSION}"
    fi
    
    # Get latest version
    LATEST_VERSION=$(get_latest_version)
    if [ -n "$LATEST_VERSION" ]; then
        echo "Latest version: v${LATEST_VERSION}"
    fi
    
    # Determine if we should auto-install
    if [ -n "$INSTALLED_VERSION" ] && [ -n "$LATEST_VERSION" ]; then
        if version_lt "$INSTALLED_VERSION" "$LATEST_VERSION"; then
            echo "Newer version available. Auto-updating..."
            SHOULD_INSTALL=1
        elif [ "$INSTALLED_VERSION" = "$LATEST_VERSION" ]; then
            echo "Already at the latest version."
            # Only prompt when running interactively (not piped)
            if [ -t 0 ] || [ -c /dev/tty ]; then
                printf "Reinstall anyway? [y/N] "
                if read -r REPLY < /dev/tty 2>/dev/null; then
                    case "$REPLY" in
                        y|Y|yes|YES) SHOULD_INSTALL=1;;
                        *) SHOULD_INSTALL=0;;
                    esac
                else
                    SHOULD_INSTALL=0
                fi
            else
                echo "Use: curl ... | sh -s -- --force  to force reinstall"
                SHOULD_INSTALL=0
            fi
        else
            echo "Installed version is newer than latest release."
            SHOULD_INSTALL=0
        fi
    else
        # Can't determine versions, prompt for reinstall
        if [ -t 0 ] || [ -c /dev/tty ]; then
            printf "Do you want to reinstall/overwrite? [y/N] "
            if read -r REPLY < /dev/tty 2>/dev/null; then
                case "$REPLY" in
                    y|Y|yes|YES) SHOULD_INSTALL=1;;
                    *) SHOULD_INSTALL=0;;
                esac
            else
                SHOULD_INSTALL=0
            fi
        else
            echo "Use: curl ... | sh -s -- --force  to force reinstall"
            SHOULD_INSTALL=0
        fi
    fi
fi

if [ "$SHOULD_INSTALL" -eq 0 ]; then
    echo "Installation cancelled."
    exit 0
fi

# Handle --force flag
if [ "$1" = "--force" ] || [ "$1" = "-f" ]; then
    echo "Force install requested."
fi

# Create temp directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Download URL
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/jjit-${TARGET}.tar.gz"

echo ""
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
echo ""
echo "Successfully installed jjit!"
"${INSTALL_DIR}/jjit" --version || true

# Check if install dir is in PATH
if ! command -v jjit >/dev/null 2>&1; then
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) 
            echo ""
            echo "Note: jjit is installed but 'jjit' command not found in current session."
            echo "Please restart your terminal or run: source ~/.bashrc"
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
echo "  1. Set up your API key (globally):"
echo "     jjit config set api_key sk-your-api-key-here --global"
echo ""
echo "  2. Or use environment variable:"
echo "     export DEEPSEEK_API_KEY=sk-your-api-key-here"
echo ""
echo "  3. Try auto-commit:"
echo "     jjit commit"
