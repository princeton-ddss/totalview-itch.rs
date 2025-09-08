#!/bin/sh
# Installation script for totalview-itch.rs NASDAQ ITCH parser
# Usage: curl -sSL https://raw.githubusercontent.com/USER/totalview-itch.rs/main/install.sh | sh

set -e

# Configuration
REPO="princeton-ddss/totalview-itch.rs"
ASSET_PREFIX="totalview-itch-rs"
BINARY_NAME="tvi"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          log_error "Unsupported OS: $(uname -s)" && exit 1 ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) log_error "Unsupported architecture: $(uname -m)" && exit 1 ;;
    esac
    
    if [[ "$os" == "windows" ]]; then
        echo "${arch}-pc-windows-msvc"
    elif [[ "$os" == "macos" ]]; then
        echo "${arch}-apple-darwin"
    else
        echo "${arch}-unknown-linux-gnu"
    fi
}

# Get latest release version
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install binary
install_binary() {
    local version="$1"
    local platform="$2"
    
    # Determine file extension based on platform
    local file_extension
    if [ "${platform#*windows}" != "$platform" ]; then
        file_extension="zip"
    else
        file_extension="tar.gz"
    fi
    
    local download_url="https://github.com/${REPO}/releases/download/${version}/${ASSET_PREFIX}-${platform}.${file_extension}"
    local temp_dir=$(mktemp -d)
    
    log_info "Downloading ${ASSET_PREFIX} ${version} for ${platform}..."
    
    # Download the release
    if ! curl -sL "$download_url" -o "${temp_dir}/${ASSET_PREFIX}.${file_extension}"; then
        log_error "Failed to download from ${download_url}"
        log_info "Available releases: https://github.com/${REPO}/releases"
        exit 1
    fi
    
    # Extract the binary
    log_info "Extracting binary..."
    if [ "$file_extension" = "zip" ]; then
        unzip -q "${temp_dir}/${ASSET_PREFIX}.zip" -d "$temp_dir"
    else
        tar -xzf "${temp_dir}/${ASSET_PREFIX}.tar.gz" -C "$temp_dir"
    fi
    
    # Make executable
    chmod +x "${temp_dir}/${BINARY_NAME}"
    
    # Install to destination
    if [ -w "$INSTALL_DIR" ]; then
        mv "${temp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
    else
        log_info "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "${temp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_success "${BINARY_NAME} installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Verify installation
# verify_installation() {
#     if command -v "$BINARY_NAME" >/dev/null 2>&1; then
#         local installed_version
#         installed_version=$("$BINARY_NAME" --version 2>/dev/null | head -1 || echo "unknown")
#         log_success "Installation verified: ${installed_version}"
#         log_info "Try running: ${BINARY_NAME} --help"
#     else
#         log_warning "${BINARY_NAME} not found in PATH"
#         log_info "You may need to add ${INSTALL_DIR} to your PATH or restart your shell"
#     fi
# }

# Main installation flow
main() {
    echo "totalview-itch.rs installer"
    echo "==========================="
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    log_info "Detected platform: ${platform}"
    
    # Check dependencies
    local required_commands="curl"
    if [ "${platform#*windows}" != "$platform" ]; then
        required_commands="$required_commands unzip"
    else
        required_commands="$required_commands tar"
    fi
    
    for cmd in $required_commands; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            log_error "Required command not found: $cmd"
            exit 1
        fi
    done
    
    # Get latest version
    local version
    version=$(get_latest_version)
    if [ -z "$version" ]; then
        log_error "Could not determine latest version"
        exit 1
    fi
    log_info "Latest version: ${version}"
    
    # Install
    install_binary "$version" "$platform"
    
    # Verify
    # verify_installation
    
    echo
    log_success "Installation complete! 🎉"
    echo
    echo "📖 Documentation: https://github.com/${REPO}"
    echo "🐛 Issues: https://github.com/${REPO}/issues"
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Environment variables:"
        echo "  INSTALL_DIR    Installation directory (default: /usr/local/bin)"
        echo
        echo "Examples:"
        echo "  $0                          # Install to /usr/local/bin"
        echo "  INSTALL_DIR=~/.local/bin $0 # Install to ~/.local/bin"
        exit 0
        ;;
    --version|-v)
        get_latest_version
        exit 0
        ;;
esac

# Run main function
main "$@"