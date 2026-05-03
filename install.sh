#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CARGO_DIR="$HOME/.cargo/bin"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Functions
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        print_error "$1 is not installed. Please install it first."
        exit 1
    fi
    print_success "$1 is installed"
}

detect_best_install_dir() {
    # Prefer ~/.cargo/bin if it exists and is in PATH
    if [[ -d "$CARGO_DIR" ]] && [[ ":$PATH:" == *":$CARGO_DIR:"* ]]; then
        INSTALL_DIR="$CARGO_DIR"
        print_info "Using cargo bin directory: $INSTALL_DIR"
    elif [[ -d "$HOME/.local/bin" ]] && [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
        INSTALL_DIR="$HOME/.local/bin"
        print_info "Using ~/.local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
        print_warning "~/.local/bin not in PATH. Will install there and show instructions."
    fi
}

build_project() {
    print_info "Building jjit in release mode..."
    cd "$REPO_DIR"
    
    if ! cargo build --release 2>&1; then
        print_error "Build failed"
        exit 1
    fi
    
    print_success "Build successful"
}

install_binary() {
    print_info "Installing jjit to $INSTALL_DIR..."
    
    # Create directory if needed
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary
    cp "$REPO_DIR/target/release/jjit" "$INSTALL_DIR/jjit"
    chmod +x "$INSTALL_DIR/jjit"
    
    print_success "jjit installed to $INSTALL_DIR/jjit"
}

check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add the following to your shell configuration file:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "For bash: ~/.bashrc"
        echo "For zsh: ~/.zshrc"
        echo "For fish: ~/.config/fish/config.fish"
    else
        print_success "$INSTALL_DIR is already in PATH"
    fi
}

verify_installation() {
    print_info "Verifying installation..."
    
    if command -v jjit &> /dev/null; then
        local version
        version=$(jjit --version 2>&1 || echo "unknown")
        print_success "jjit is available: $version"
        
        echo ""
        echo "Quick start:"
        echo "  jjit config set api_key <your-deepseek-api-key>"
        echo "  jjit commit              # Auto-generate commit message"
        echo "  jjit goto \"initial\"      # Checkout specific commit"
        echo "  jjit pack \"all\"          # Squash commits"
        echo ""
        echo "For more information: jjit --help"
    else
        print_error "jjit is not in PATH. Please check your installation."
        exit 1
    fi
}

uninstall() {
    print_info "Uninstalling jjit..."
    
    if [[ -f "$INSTALL_DIR/jjit" ]]; then
        rm "$INSTALL_DIR/jjit"
        print_success "Removed $INSTALL_DIR/jjit"
    else
        print_warning "jjit not found in $INSTALL_DIR"
    fi
    
    # Also check cargo bin
    if [[ -f "$CARGO_DIR/jjit" ]]; then
        rm "$CARGO_DIR/jjit"
        print_success "Removed $CARGO_DIR/jjit"
    fi
}

# Help
show_help() {
    cat << EOF
jjit Installer

Usage: $0 [OPTIONS] [COMMAND]

Commands:
    install     Install jjit (default)
    uninstall   Remove jjit
    help        Show this help message

Options:
    --dir DIR   Install to specific directory (default: auto-detect)
    --help      Show this help message

Examples:
    $0                          # Install to default location
    $0 --dir /usr/local/bin     # Install to specific directory
    $0 uninstall                # Remove jjit
EOF
}

# Main
main() {
    local command="install"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            install|uninstall|help)
                command="$1"
                shift
                ;;
            --dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    echo "╔════════════════════════════════════╗"
    echo "║      jjit Installer v0.1.0        ║"
    echo "╚════════════════════════════════════╝"
    echo ""
    
    case "$command" in
        install)
            print_info "Checking dependencies..."
            check_dependency "cargo"
            check_dependency "jj"
            
            if [[ -z "${INSTALL_DIR+x}" ]]; then
                detect_best_install_dir
            fi
            
            build_project
            install_binary
            check_path
            verify_installation
            ;;
        uninstall)
            detect_best_install_dir
            uninstall
            ;;
        help)
            show_help
            ;;
    esac
}

main "$@"
